use std::borrow::Cow;
use std::path::{Path, PathBuf};

use anyhow::Error;

use crate::model::{ConstValue, EMPTY_TAGS, ModifyMode, MyTag, SetWhen};
use crate::op::{check_numeric_date_tags_must_be_overwrite, check_tags_type_value_type, check_value_is_ok, get_file_iterator, get_tags_from_args, get_where};
use crate::op::{Action, WalkAction, WriteAction, WriteAllAction};
use crate::op::tag_impl::ReadWriteTag;
use crate::where_clause::WhereClause;

pub struct SetConstAction {
    it: Box<dyn Iterator<Item=PathBuf>>,
    dry_run: bool,
    tags: Vec<MyTag>,
    where_clause: Option<WhereClause>,
    value: ConstValue,
    set_when: SetWhen,
    modify_mode: ModifyMode,
    // cache
    text_value: String,
}

impl SetConstAction {
    pub fn new<P>(dir: P,
                  dry_run: bool,
                  tags: &[MyTag],
                  where_string: &Option<String>,
                  value: ConstValue,
                  set_when: &SetWhen,
                  modify_mode: &ModifyMode) -> Result<Self, Error>
        where P: AsRef<Path>
    {
        let it = get_file_iterator(dir.as_ref())?;
        let tags = get_tags_from_args(tags, &EMPTY_TAGS)?;
        let where_clause = get_where(where_string)?;
        let text_value =  (&value.get_text_value()).to_owned();
        Self::check(&tags, modify_mode, &value).map(|_|
            Self {
                it,
                dry_run,
                tags,
                where_clause,
                value,
                set_when: set_when.clone(),
                modify_mode: modify_mode.clone(),
                text_value,
            })
    }

    fn check(tags: &[MyTag],
             modify_mode: &ModifyMode,
             value: &ConstValue) -> Result<(), Error> {
        check_numeric_date_tags_must_be_overwrite(tags, modify_mode)?;
        check_tags_type_value_type(tags, value)?;
        check_value_is_ok(tags, value)?;
        Ok(())
    }

    fn set_numeric_tag_real(&self, t: &mut dyn ReadWriteTag, tag: &MyTag) -> bool {
        if let Some((value, padding)) = self.get_numeric_value() {
            t.write_numeric_tag(tag, value, padding);
            true
        } else {
            false
        }
    }

    fn set_date_tag_real(&self, t: &mut dyn ReadWriteTag, tag: &MyTag) -> bool {
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
        match &self.value {
            ConstValue::Num { value, padding } => Some((*value, *padding)),
            _ => None,
        }
    }

    fn get_date_value(&self) -> Option<(&str, &str)> {
        match &self.value {
            ConstValue::Date { value, format } => Some((value, format)),
            _ => None,
        }
    }
}

impl Action for SetConstAction {
    fn do_any(&mut self) -> Result<(), Error> {
        self.do_all()
    }
}

impl WalkAction for SetConstAction {
    fn get_iterator(&mut self) -> &mut dyn Iterator<Item=PathBuf> {
        &mut self.it
    }

    fn do_one_file(&mut self, path: &Path) -> Result<bool, Error> {
        self.do_one_file_write(path)
    }

    fn get_where(&self) -> &Option<WhereClause> {
        &self.where_clause
    }

    fn tags(&self) -> &Vec<MyTag> {
        &self.tags
    }
}

impl WriteAction for SetConstAction {
    fn is_dry_run(&self) -> bool {
        self.dry_run
    }

    fn write_tags(&self, t: &mut dyn ReadWriteTag) -> Result<bool, Error> {
        self.write_tags_impl(t)
    }
}

impl WriteAllAction for SetConstAction {
    fn set_text_tag(&self, t: &mut dyn ReadWriteTag, tag: &MyTag) -> bool {
        if &self.set_when == &SetWhen::Always && &self.modify_mode == &ModifyMode::Overwrite {
            let new_value = self.get_text_value();
            t.write_text_tag(tag, new_value);
            true
        } else {
            let current = t.get_text_tag(tag);
            if should_write_text(&current, &self.set_when) {
                let new_value = self.get_new_text(&current);
                t.write_text_tag(tag, &new_value);
                true
            } else {
                false
            }
        }
    }

    fn set_numeric_tag(&self, t: &mut dyn ReadWriteTag, tag: &MyTag) -> bool {
        match &self.set_when {
            SetWhen::Always => {
                self.set_numeric_tag_real(t, tag)
            }
            set_when => {
                let current = t.get_numeric_tag(tag);
                if should_write_numeric(&current, set_when) {
                    self.set_numeric_tag_real(t, tag)
                } else {
                    false
                }
            }
        }
    }

    fn set_date_tag(&self, t: &mut dyn ReadWriteTag, tag: &MyTag) -> bool {
        match &self.set_when {
            SetWhen::Always => {
                self.set_date_tag_real(t, tag)
            }
            set_when => {
                let current = t.get_text_tag(tag);
                if should_write_text(&current, set_when) {
                    self.set_date_tag_real(t, tag)
                } else {
                    false
                }
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