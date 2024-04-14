use anyhow::Error;
use log::warn;
use std::borrow::Cow;
use std::path::Path;

use crate::args::Sequence;
use crate::model::{ModifyMode, MyTag};
use crate::op::{check_numeric_date_tags_must_be_overwrite, check_tags_not_empty, Action, MAX_NUMBER, SeqAction};
use crate::op::SeqWriteAction;
use crate::op::tag_impl::{ReadTag, TagImpl, WriteTag, WriteTagFile};
use crate::util::numeric::decimal_to_padding_string;

pub struct SetSeqAction<'a> {
    dir: &'a Path,
    dry_run: bool,
    tags: &'a Vec<MyTag>,
    value: &'a Sequence,
    hyphen: &'a str,
    modify_mode: &'a ModifyMode,
}

impl<'a> SetSeqAction<'a> {
    pub fn new(dir: &'a Path,
               dry_run: bool,
               tags: &'a Vec<MyTag>,
               value: &'a Sequence,
               hyphen: &'a str,
               modify_mode: &'a ModifyMode) -> Result<SetSeqAction<'a>, Error> {
        Self::check(tags, modify_mode)
            .map(|_| {
                SetSeqAction {
                    dir,
                    dry_run,
                    tags,
                    value,
                    hyphen,
                    modify_mode,
                }
            })
    }

    fn check(tags: &Vec<MyTag>,
             modify_mode: &ModifyMode) -> Result<(), Error> {
        check_tags_not_empty(tags)?;
        check_numeric_date_tags_must_be_overwrite(tags, modify_mode)?;
        Ok(())
    }

    fn set_text_tag3<T>(&self, t: &mut T, tag: &MyTag, from_seq: &str) -> bool
        where T: ReadTag + WriteTag + ?Sized {
        if self.modify_mode == &ModifyMode::Overwrite {
            let new_value = from_seq;
            t.write_text_tag(tag, new_value);
        } else {
            let current = t.get_text_tag(tag);
            let new_value = self.get_new_value(&current, from_seq);
            t.write_text_tag(tag, &new_value);
        }
        true
    }

    fn get_new_value<'b>(&self, current: &Option<String>, from_seq: &'b str) -> Cow<'b, str> {
        match self.modify_mode {
            ModifyMode::Append =>
                Cow::Owned(String::new() + &current.clone().unwrap_or_default()
                    + self.hyphen + from_seq),
            ModifyMode::Insert =>
                Cow::Owned(String::new() + from_seq + self.hyphen
                    + &current.clone().unwrap_or_default()),
            ModifyMode::Overwrite => Cow::Borrowed(from_seq)
        }
    }

    fn set_numeric_tag(&self, t: &mut TagImpl, tag: &MyTag, from_seq: &str) -> bool {
        let new_value = from_seq.parse::<u32>().unwrap();
        t.write_numeric_tag(tag, new_value, self.value.padding);
        true
    }
}

impl SeqAction for SetSeqAction<'_> {
    fn do_one_file(&self, path: &Path, seq: &Option<&str>) -> Result<(), Error> {
        self.do_one_file_write(path, seq)
    }

    fn get_next_seq(&self, seed: &Option<u32>) -> Option<(String, u32)> {
        get_next_value(self.value, seed)
    }
}

impl Action for SetSeqAction<'_> {
    fn do_dir(&self) -> Result<(), Error> {
        self.do_dir_seq()
    }

    fn do_file(&self) -> Result<(), Error> {
        self.do_file_impl()
    }

    fn op_name(&self) -> &'static str {
        "set-seq"
    }

    fn get_path(&self) -> &Path {
        self.dir
    }

    fn get_tags(&self) -> &Vec<MyTag> {
        self.tags
    }
}

impl SeqWriteAction for SetSeqAction<'_> {
    fn is_dry_run(&self) -> bool {
        self.dry_run
    }

    fn set_tags_some(&self,
                     t: &mut TagImpl,
                     out: &Option<&str>) -> Result<(), Error> {
        if self.tags.is_empty() {
            return Ok(());
        }

        let mut any_changed = false;
        let out = out.unwrap();
        for tag in self.tags {
            let changed = match tag {
                MyTag::Title => self.set_text_tag3(t, &MyTag::Title, out),
                MyTag::Artist => self.set_text_tag3(t, &MyTag::Artist, out),
                MyTag::AlbumTitle => self.set_text_tag3(t, &MyTag::AlbumTitle, out),
                MyTag::Genre => self.set_text_tag3(t, &MyTag::Genre, out),
                MyTag::Comment => self.set_text_tag3(t, &MyTag::Comment, out),
                MyTag::AlbumArtist => self.set_text_tag3(t, &MyTag::AlbumArtist, out),
                MyTag::Composer => self.set_text_tag3(t, &MyTag::Composer, out),
                MyTag::Copyright => self.set_text_tag3(t, &MyTag::Copyright, out),

                MyTag::Year => self.set_numeric_tag(t, &MyTag::Year, out),
                MyTag::TrackNumber => self.set_numeric_tag(t, &MyTag::TrackNumber, out),
                MyTag::TrackTotal => self.set_numeric_tag(t, &MyTag::TrackTotal, out),
                MyTag::DiscNumber => self.set_numeric_tag(t, &MyTag::DiscNumber, out),
                MyTag::DiscTotal => self.set_numeric_tag(t, &MyTag::DiscTotal, out),

                MyTag::Date => {
                    warn!("Unable set tag {} as Sequence, Ignore it.", tag);
                    false
                }
            };
            if !any_changed {
                any_changed = changed;
            }
        }

        if any_changed {
            t.save()
        } else {
            Ok(())
        }
    }
}

fn get_next_value(value: &Sequence, seed: &Option<u32>) -> Option<(String, u32)> {
    let (start, step, padding) = (value.start, value.step, value.padding);

    if let Some(v) = seed {
        let new_value = *v + step;
        if new_value <= MAX_NUMBER {
            Some((decimal_to_padding_string(new_value, padding), new_value))
        } else {
            None
        }
    } else {
        Some((decimal_to_padding_string(start, padding), start))
    }
}

#[cfg(test)]
mod test {
    use crate::args::Sequence;
    use super::get_next_value;

    #[test]
    fn test_get_next_value() {
        let seq = Sequence { start: 1, step: 1, padding: 2 };
        assert_eq!(get_next_value(&seq, &None), Some(("01".to_string(), 1)));
        assert_eq!(get_next_value(&seq, &Some(1)), Some(("02".to_string(), 2)));
        assert_eq!(get_next_value(&seq, &Some(10)), Some(("11".to_string(), 11)));
    }
}