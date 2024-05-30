use std::borrow::Cow;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use anyhow::Error;
use log::{error, warn};

use crate::model::{EMPTY_TAGS, ModifyMode, MyTag};
use crate::op::{check_numeric_date_tags_must_be_overwrite, get_dir_iterator, get_tags_from_args, MAX_NUMBER, sorted_filtered_files};
use crate::op::{Action, SeqAction};
use crate::op::SeqWriteAction;
use crate::op::tag_impl::{ReadTag, ReadWriteTag, WriteTag};
use crate::util::numeric::decimal_to_padding_string;

pub struct SetSeqAction {
    it: Box<dyn Iterator<Item=PathBuf>>,
    file: Rc<Option<PathBuf>>,
    dry_run: bool,
    tags: Vec<MyTag>,
    seq_start: u32,
    seq_step: u32,
    seq_padding: usize,
    hyphen: String,
    modify_mode: ModifyMode,
}

impl SetSeqAction {
    pub fn new<P>(dir: P,
                  dry_run: bool,
                  tags: &[MyTag],
                  seq_start: u32,
                  seq_step: u32,
                  seq_padding: usize,
                  hyphen: &str,
                  modify_mode: &ModifyMode) -> Result<Self, Error>
        where P: AsRef<Path>
    {
        let (it, file) = get_dir_iterator(dir.as_ref())?;
        let tags = get_tags_from_args(tags, &EMPTY_TAGS)?;
        Self::check(&tags, modify_mode).map(|_| {
            Self {
                it,
                file: Rc::new(file),
                dry_run,
                tags,
                seq_start,
                seq_step,
                seq_padding,
                hyphen: hyphen.to_owned(),
                modify_mode: modify_mode.clone(),
            }
        })
    }

    fn check(tags: &Vec<MyTag>,
             modify_mode: &ModifyMode) -> Result<(), Error> {
        check_numeric_date_tags_must_be_overwrite(tags, modify_mode)?;
        Ok(())
    }

    fn get_next_seq(&self, seed: &Option<u32>) -> Option<(String, u32)> {
        get_next_value(self.seq_start, self.seq_step, self.seq_padding, seed)
    }

    fn set_text_tag<T>(&self, t: &mut T, tag: &MyTag, from_seq: &str) -> bool
        where T: ReadTag + WriteTag + ?Sized
    {
        match &self.modify_mode {
            ModifyMode::Overwrite => {
                let new_value = from_seq;
                t.write_text_tag(tag, new_value);
            }
            _ => {
                let current = t.get_text_tag(tag);
                let new_value = self.get_new_value(&current, from_seq);
                t.write_text_tag(tag, &new_value);
            }
        }
        true
    }

    fn get_new_value<'b>(&self, current: &Option<String>, from_seq: &'b str) -> Cow<'b, str> {
        match self.modify_mode {
            ModifyMode::Append =>
                Cow::Owned(String::new() + &current.clone().unwrap_or_default()
                    + &self.hyphen + from_seq),
            ModifyMode::Insert =>
                Cow::Owned(String::new() + from_seq + &self.hyphen
                    + &current.clone().unwrap_or_default()),
            ModifyMode::Overwrite => Cow::Borrowed(from_seq)
        }
    }

    fn set_numeric_tag(&self, t: &mut dyn ReadWriteTag, tag: &MyTag, from_seq: &str) -> bool {
        let new_value = from_seq.parse::<u32>().unwrap();
        t.write_numeric_tag(tag, new_value, self.seq_padding);
        true
    }
}

impl SeqAction for SetSeqAction {
    fn do_next(&mut self) -> Option<Result<(), Error>> {
        let it = self.get_iterator();
        if let Some(ref dir_path) = it.next() {
            if let Some(file_path) = Rc::clone(&self.file).as_ref() {
                if let Some((value, _)) = self.get_next_seq(&None) {
                    let _ = self.do_one_file(file_path, &Some(&value))
                        .map_err(|e| error!("Error: {}", e));
                } else {
                    error!("Sequence index out of boundary. last is {:?}.", None::<u32>);
                }
                return Some(Ok(()));
            } else {
                match sorted_filtered_files(dir_path) {
                    Ok(file_list) => {
                        let mut seed: Option<u32> = None;

                        for path in file_list {
                            if let Some((value, next)) = self.get_next_seq(&seed) {
                                self.do_one_file(&path, &Some(&value))
                                    .map_or_else(|e| error!("Error: {}", e),
                                                 |_| seed = Some(next));
                            } else {
                                error!("Sequence index out of boundary. last is {:?}.", seed);
                                break;
                            }
                        }
                    }
                    Err(e) => { error!("Error: {}.", e); }
                }
            }
            return Some(Ok(()));
        }
        None
    }

    fn get_iterator(&mut self) -> &mut dyn Iterator<Item=PathBuf> {
        &mut self.it
    }

    fn do_one_file(&mut self, path: &Path, seq: &Option<&str>) -> Result<(), Error> {
        self.do_one_file_write(path, seq)
    }

    fn tags(&self) -> &Vec<MyTag> {
        &self.tags
    }
}

impl Action for SetSeqAction {
    fn do_any(&mut self) -> Result<(), Error> {
        self.do_all()
    }
}

impl SeqWriteAction for SetSeqAction {
    fn is_dry_run(&self) -> bool {
        self.dry_run
    }

    fn set_tags(&self,
                t: &mut dyn ReadWriteTag,
                out: &Option<&str>) -> Result<(), Error> {
        if self.tags.is_empty() {
            return Ok(());
        }

        let mut any_changed = false;
        let out = out.unwrap();
        for tag in &self.tags {
            let changed = match tag {
                MyTag::Title
                | MyTag::Artist
                | MyTag::AlbumTitle
                | MyTag::Genre
                | MyTag::Comment
                | MyTag::AlbumArtist
                | MyTag::Composer
                | MyTag::Copyright
                | MyTag::Lyrics => self.set_text_tag(t, tag, out),

                MyTag::Year
                | MyTag::TrackNumber
                | MyTag::TrackTotal
                | MyTag::DiscNumber
                | MyTag::DiscTotal => self.set_numeric_tag(t, tag, out),

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

fn get_next_value(seq_start: u32,
                  seq_step: u32,
                  seq_padding: usize,
                  seed: &Option<u32>) -> Option<(String, u32)> {
    if let Some(v) = seed {
        let new_value = *v + seq_step;
        if new_value <= MAX_NUMBER {
            Some((decimal_to_padding_string(new_value, seq_padding), new_value))
        } else {
            None
        }
    } else {
        Some((decimal_to_padding_string(seq_start, seq_padding), seq_start))
    }
}

#[cfg(test)]
mod test {
    use super::get_next_value;

    #[test]
    fn test_get_next_value() {
        assert_eq!(get_next_value(1, 1, 2, &None),
                   Some(("01".to_string(), 1)));
        assert_eq!(get_next_value(1, 1, 2, &Some(1)),
                   Some(("02".to_string(), 2)));
        assert_eq!(get_next_value(1, 1, 2, &Some(10)),
                   Some(("11".to_string(), 11)));
    }
}