use anyhow::Error;
use std::borrow::Cow;
use std::path::Path;

use crate::args::ConstValueArgs;
use crate::model::{ModifyMode, MyTag, SetWhen};
use crate::op::{check_numeric_date_tags_must_be_overwrite, check_tags_not_empty, check_tags_type_value_type, check_value_is_ok, Action, WalkAction, WriteAllAction};
use crate::op::WriteAction;
use crate::op::tag_impl::{ReadTag, TagImpl, WriteTag};

pub struct SetConstAction<'a> {
    dir: &'a Path,
    dry_run: bool,
    tags: &'a Vec<MyTag>,
    value: &'a ConstValueArgs,
    set_when: &'a SetWhen,
    modify_mode: &'a ModifyMode,
    // cache
    text_value: String,
}

impl<'a> SetConstAction<'a> {
    pub fn new(dir: &'a Path,
               dry_run: bool,
               tags: &'a Vec<MyTag>,
               value: &'a ConstValueArgs,
               set_when: &'a SetWhen,
               modify_mode: &'a ModifyMode) -> Result<SetConstAction<'a>, Error> {
        Self::check(tags, modify_mode, value).map(|_|
            SetConstAction {
                dir,
                dry_run,
                tags,
                value,
                set_when,
                modify_mode,
                text_value: value.get_text_value(),
            })
    }

    fn check(tags: &Vec<MyTag>,
             modify_mode: &ModifyMode,
             value: &ConstValueArgs) -> Result<(), Error> {
        check_tags_not_empty(tags)?;
        check_numeric_date_tags_must_be_overwrite(tags, modify_mode)?;
        check_tags_type_value_type(tags, value)?;
        check_value_is_ok(tags, value)?;
        Ok(())
    }

    fn set_numeric_tag_real(&self, t: &mut TagImpl, tag: &MyTag) -> bool {
        if let Some((value, padding)) = self.get_numeric_value() {
            t.write_numeric_tag(tag, value, padding);
            true
        } else {
            false
        }
    }

    fn set_date_tag_real(&self, t: &mut TagImpl, tag: &MyTag) -> bool {
        if let Some((value, _format)) = self.get_date_value() {
            t.write_text_tag(tag, value);
            true
        } else {
            false
        }
    }

    fn get_new_text(&self, current: &Option<String>) -> Cow<str> {
        if current.is_none() {
            Cow::Borrowed(self.get_text_value())
        } else {
            match self.modify_mode {
                ModifyMode::Append => {
                    Cow::Owned(String::new() + &current.clone().unwrap() + &self.get_text_value())
                }
                ModifyMode::Insert => {
                    Cow::Owned(String::new() + &self.get_text_value() + &current.clone().unwrap())
                }
                ModifyMode::Overwrite => {
                    Cow::Borrowed(self.get_text_value())
                }
            }
        }
    }

    fn get_text_value(&self) -> &str {
        &self.text_value
    }

    fn get_numeric_value(&self) -> Option<(u32, usize)> {
        match self.value {
            ConstValueArgs::Num { value, padding } => Some((*value, *padding)),
            _ => None,
        }
    }

    fn get_date_value(&self) -> Option<(&str, &str)> {
        match self.value {
            ConstValueArgs::Date { value, format } => Some((value, format)),
            _ => None,
        }
    }
}

impl Action for SetConstAction<'_> {
    fn do_dir(&self) -> Result<(), Error> {
        self.do_dir_walk()
    }

    fn do_file(&self) -> Result<(), Error> {
        self.do_file_impl()
    }

    fn op_name(&self) -> &'static str {
        "set-const"
    }

    fn get_path(&self) -> &Path {
        self.dir
    }

    fn get_tags(&self) -> &Vec<MyTag> {
        self.tags
    }
}

impl WalkAction for SetConstAction<'_> {
    fn do_one_file(&self, path: &Path) -> Result<(), Error> {
        self.do_one_file_write(path)
    }
}

impl WriteAction for SetConstAction<'_> {
    fn is_dry_run(&self) -> bool {
        self.dry_run
    }

    fn set_tags_some(&self, t: &mut TagImpl) -> Result<(), Error> {
        self.set_tags_some_impl(t)
    }
}

impl WriteAllAction for SetConstAction<'_> {
    fn set_text_tag(&self, t: &mut TagImpl, tag: &MyTag) -> bool {
        if self.set_when == &SetWhen::Always && self.modify_mode == &ModifyMode::Overwrite {
            let new_value = self.get_text_value();
            t.write_text_tag(tag, new_value);
            true
        } else {
            let current = t.get_text_tag(tag);
            if should_write_text(&current, self.set_when) {
                let new_value = self.get_new_text(&current);
                t.write_text_tag(tag, &new_value);
                true
            } else {
                false
            }
        }
    }

    fn set_numeric_tag(&self, t: &mut TagImpl, tag: &MyTag) -> bool {
        if self.set_when == &SetWhen::Always {
            self.set_numeric_tag_real(t, tag)
        } else {
            let current = t.get_numeric_tag(tag);
            if should_write_numeric(&current, &self.set_when) {
                self.set_numeric_tag_real(t, tag)
            } else {
                false
            }
        }
    }

    fn set_date_tag(&self, t: &mut TagImpl, tag: &MyTag) -> bool {
        if self.set_when == &SetWhen::Always {
            self.set_date_tag_real(t, tag)
        } else {
            let current = t.get_text_tag(tag);
            if should_write_text(&current, &self.set_when) {
                self.set_date_tag_real(t, tag)
            } else {
                false
            }
        }
    }
}

fn should_write_text(current: &Option<String>, set_when: &SetWhen) -> bool {
    if let Some(curr) = current {
        if !curr.is_empty() {
            set_when != &SetWhen::OnlyEmpty
        } else {
            set_when != &SetWhen::OnlyNotEmpty
        }
    } else {
        set_when != &SetWhen::OnlyNotEmpty
    }
}

fn should_write_numeric(current: &Option<u32>, set_when: &SetWhen) -> bool {
    if current.is_some() {
        set_when != &SetWhen::OnlyEmpty
    } else {
        set_when != &SetWhen::OnlyNotEmpty
    }
}

#[cfg(test)]
mod test {
    use crate::model::SetWhen;
    use super::{should_write_numeric, should_write_text};

    #[test]
    fn test_should_write_text() {
        assert!(should_write_text(&None, &SetWhen::Always));
        assert!(should_write_text(&Some("".to_owned()), &SetWhen::Always));
        assert!(should_write_text(&Some("abc".to_owned()), &SetWhen::Always));

        assert!(should_write_text(&None, &SetWhen::OnlyEmpty));
        assert!(should_write_text(&Some("".to_owned()), &SetWhen::OnlyEmpty));
        assert_eq!(false, should_write_text(&Some("abc".to_owned()), &SetWhen::OnlyEmpty));

        assert_eq!(false, should_write_text(&None, &SetWhen::OnlyNotEmpty));
        assert_eq!(false, should_write_text(&Some("".to_owned()), &SetWhen::OnlyNotEmpty));
        assert!(should_write_text(&Some("abc".to_owned()), &SetWhen::OnlyNotEmpty));
    }

    #[test]
    fn test_should_write_numeric() {
        assert!(should_write_numeric(&None, &SetWhen::Always));
        assert!(should_write_numeric(&Some(2), &SetWhen::Always));

        assert!(should_write_numeric(&None, &SetWhen::OnlyEmpty));
        assert_eq!(false, should_write_numeric(&Some(2), &SetWhen::OnlyEmpty));

        assert_eq!(false, should_write_numeric(&None, &SetWhen::OnlyNotEmpty));
        assert!(should_write_numeric(&Some(2), &SetWhen::OnlyNotEmpty));
    }
}