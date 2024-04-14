extern crate lazy_static;

use anyhow::{anyhow, Error};
use chrono::NaiveDate;
use log::{debug, error, warn};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::{DirEntry, WalkDir};

use crate::args::ConstValueArgs;
use crate::model::{ModifyMode, MyTag};
use self::tag_impl::{is_available_suffix, ReadTag, TagImpl, WriteTag, WriteTagFile};

pub use self::conv_en::ConvEnAction;
pub use self::conv_zh::ConvZhAction;
pub use self::conv_utf8::ConvUtf8Action;
pub use self::mod_num::ModNumAction;
pub use self::mod_text_const::ModTextConstAction;
pub use self::mod_text_regex::ModTextRegexAction;
pub use self::set_const::SetConstAction;
pub use self::set_seq::SetSeqAction;
pub use self::view::ViewAction;

mod conv_en;
mod conv_utf8;
mod conv_zh;
mod mod_num;
mod mod_text_const;
mod mod_text_regex;
mod set_const;
mod set_seq;
mod view;

mod tag_impl;

pub trait Action {
    fn do_any(&self) -> Result<(), Error> {
        let path = self.get_path();
        if path.is_dir() {
            self.do_dir()
        } else if path.is_file() {
            self.do_file()
        } else {
            err_could_not_perform_action_on_path(self.op_name(), self.get_path())
        }
    }

    fn do_dir(&self) -> Result<(), Error>;

    fn do_file(&self) -> Result<(), Error>;

    fn op_name(&self) -> &'static str;

    fn get_path(&self) -> &Path;

    fn get_tags(&self) -> &Vec<MyTag>;
}

trait WalkAction: Action {
    fn do_dir_walk(&self) -> Result<(), Error> {
        let dir = self.get_path();
        debug!("dir: {:?}", dir);

        for entry in WalkDir::new(dir)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok()) {
            let meta = entry.metadata();
            if meta.is_ok() && meta.unwrap().is_file() {
                let path = entry.path();
                if is_available_suffix(&path.to_string_lossy()) {
                    self.do_one_file(path)
                        .err()
                        .map(|e| error!("Error: {}", e));
                }
            }
        }

        Ok(())
    }

    fn do_one_file(&self, path: &Path) -> Result<(), Error>;

    fn do_file_impl(&self) -> Result<(), Error> {
        match check_input_path(self.get_path()) {
            Ok(_) => (),
            Err(error) => return Err(error)
        }

        self.do_one_file(self.get_path())
    }
}

trait SeqAction: Action {
    fn do_dir_seq(&self) -> Result<(), Error> {
        let dir = self.get_path();
        debug!("dir: {:?}", dir);

        'out: for dir_entry in WalkDir::new(&dir.to_str().unwrap())
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok()) {
            let meta = dir_entry.metadata();
            if meta.is_ok() && meta.unwrap().is_dir() {
                let file_list = sorted_files(&dir_entry)?;
                let mut seed: Option<u32> = None;

                for path in file_list {
                    if is_available_suffix(&path.to_string_lossy()) {
                        if let Some((value, next)) = self.get_next_seq(&seed) {
                            self.do_one_file(&path, &Some(&value))
                                .map_or_else(|e| error!("Error: {}", e),
                                             |_| seed = Some(next));
                        } else {
                            error!("Sequence index out of boundary. last is {:?}.", seed);
                            continue 'out;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn do_one_file(&self, path: &Path, seq: &Option<&str>) -> Result<(), Error>;
    fn get_next_seq(&self, seed: &Option<u32>) -> Option<(String, u32)>;

    fn do_file_impl(&self) -> Result<(), Error> {
        match check_input_path(self.get_path()) {
            Ok(_) => (),
            Err(error) => return Err(error)
        }

        self.do_one_file(self.get_path(), &None)
    }
}

fn sorted_files(dir_entry: &DirEntry) -> Result<Vec<PathBuf>, Error> {
    let mut paths: Vec<_> = fs::read_dir(dir_entry.path())?
        .into_iter()
        .filter(|r| r.is_ok())
        .map(|r| r.unwrap().path())
        .filter(|r| r.is_file())
        .collect();
    paths.sort();
    Ok(paths)
}

trait ReadAction: WalkAction {
    fn get_tags_some(&self, t: &TagImpl);

    fn do_one_file_read(&self, path: &Path) -> Result<(), Error> {
        let tag_impl = TagImpl::new(path, true)?;
        self.get_tags_some(&tag_impl);
        Ok(())
    }
}

trait WriteAction: WalkAction {
    fn is_dry_run(&self) -> bool;

    fn do_one_file_write(&self, path: &Path) -> Result<(), Error> {
        let mut tag_impl = TagImpl::new(path, self.is_dry_run())?;
        self.set_tags_some(&mut tag_impl)
    }

    fn set_tags_some(&self, t: &mut TagImpl) -> Result<(), Error>;
}

trait WriteTextAction: WriteAction {
    fn set_text_tag(&self, t: &mut TagImpl, tag: &MyTag) -> bool {
        let current = &t.get_text_tag(tag);
        let new_value = self.get_new_text(current);

        if let Some(new_v) = &new_value {
            t.write_text_tag(tag, new_v);
            true
        } else {
            false
        }
    }

    fn get_new_text(&self, current: &Option<String>) -> Option<String>;

    fn set_tags_some_impl(&self, t: &mut TagImpl) -> Result<(), Error> {
        if self.get_tags().is_empty() {
            return Ok(());
        }

        let mut any_changed = false;
        for tag in self.get_tags() {
            let changed = match tag {
                MyTag::Title => self.set_text_tag(t, &MyTag::Title),
                MyTag::Artist => self.set_text_tag(t, &MyTag::Artist),
                MyTag::AlbumTitle => self.set_text_tag(t, &MyTag::AlbumTitle),
                MyTag::Genre => self.set_text_tag(t, &MyTag::Genre),
                MyTag::Comment => self.set_text_tag(t, &MyTag::Comment),
                MyTag::AlbumArtist => self.set_text_tag(t, &MyTag::AlbumArtist),
                MyTag::Composer => self.set_text_tag(t, &MyTag::Composer),
                MyTag::Copyright => self.set_text_tag(t, &MyTag::Copyright),

                _ => false,
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

trait WriteNumAction: WriteAction {
    fn set_tags_some_impl(&self, t: &mut TagImpl) -> Result<(), Error> {
        if self.get_tags().is_empty() {
            return Ok(());
        }

        let mut any_changed = false;
        for tag in self.get_tags() {
            let changed = match tag {
                MyTag::Year => self.set_numeric_tag(t, &MyTag::Year),
                MyTag::TrackNumber => self.set_numeric_tag(t, &MyTag::TrackNumber),
                MyTag::TrackTotal => self.set_numeric_tag(t, &MyTag::TrackTotal),
                MyTag::DiscNumber => self.set_numeric_tag(t, &MyTag::DiscNumber),
                MyTag::DiscTotal => self.set_numeric_tag(t, &MyTag::DiscTotal),

                _ => false,
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

    fn set_numeric_tag(&self, t: &mut TagImpl, tag: &MyTag) -> bool {
        let current = t.get_numeric_tag(tag);
        let new_value = self.get_new_numeric(&current);

        if let Some(new_v) = new_value {
            if numeric_is_valid(tag, new_v) {
                t.write_numeric_tag(tag, new_v, *self.get_padding());
                true
            } else {
                warn!("Numeric value {} exceed the boundary.", new_v);
                false
            }
        } else {
            false
        }
    }

    fn get_padding(&self) -> &usize;

    fn get_new_numeric(&self, current: &Option<u32>) -> Option<u32>;
}

trait WriteAllAction: WriteAction {
    fn set_tags_some_impl(&self, t: &mut TagImpl) -> Result<(), Error> {
        if self.get_tags().is_empty() {
            return Ok(());
        }

        let mut any_changed = false;
        for tag in self.get_tags() {
            let changed = match tag {
                MyTag::Title => self.set_text_tag(t, &MyTag::Title),
                MyTag::Artist => self.set_text_tag(t, &MyTag::Artist),
                MyTag::AlbumTitle => self.set_text_tag(t, &MyTag::AlbumTitle),
                MyTag::Genre => self.set_text_tag(t, &MyTag::Genre),
                MyTag::Comment => self.set_text_tag(t, &MyTag::Comment),
                MyTag::AlbumArtist => self.set_text_tag(t, &MyTag::AlbumArtist),
                MyTag::Composer => self.set_text_tag(t, &MyTag::Composer),
                MyTag::Copyright => self.set_text_tag(t, &MyTag::Copyright),

                MyTag::Year => self.set_numeric_tag(t, &MyTag::Year),
                MyTag::TrackNumber => self.set_numeric_tag(t, &MyTag::TrackNumber),
                MyTag::TrackTotal => self.set_numeric_tag(t, &MyTag::TrackTotal),
                MyTag::DiscNumber => self.set_numeric_tag(t, &MyTag::DiscNumber),
                MyTag::DiscTotal => self.set_numeric_tag(t, &MyTag::DiscTotal),

                MyTag::Date => self.set_date_tag(t, &MyTag::Date),
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

    fn set_text_tag(&self, t: &mut TagImpl, tag: &MyTag) -> bool;

    fn set_numeric_tag(&self, t: &mut TagImpl, tag: &MyTag) -> bool;

    fn set_date_tag(&self, t: &mut TagImpl, tag: &MyTag) -> bool;
}

trait SeqWriteAction: SeqAction {
    fn is_dry_run(&self) -> bool;

    fn do_one_file_write(&self, path: &Path, seq: &Option<&str>) -> Result<(), Error> {
        let mut tag_impl = TagImpl::new(path, self.is_dry_run())?;
        self.set_tags_some(&mut tag_impl, seq)
    }

    fn set_tags_some(&self,
                     t: &mut TagImpl,
                     other: &Option<&str>) -> Result<(), Error>;
}

fn check_input_path(input_path: &Path) -> Result<(), Error> {
    if !input_path.is_file() {
        Err(anyhow!("{input_path:?} is not file!"))
    } else {
        Ok(())
    }
}

fn err_could_not_perform_action_on_path(op_name: &str, path: &Path) -> Result<(), Error> {
    Err(anyhow!("Could NOT perform action {} on path: {:?}. Please check the path.",
        op_name, path))
}

const MIN_PADDING: usize = 1;
const MAX_PADDING: usize = 9;
const MIN_NUMBER: u32 = 0;
const MIN_NATURAL_NUMBER: u32 = 1;
const MAX_NUMBER: u32 = u16::MAX as u32;

fn check_tags_not_empty(tags: &Vec<MyTag>) -> Result<(), Error> {
    if tags.is_empty() {
        Err(anyhow!("You must specified one tag at least for \"--tags\" parameter!"))
    } else {
        Ok(())
    }
}

fn check_numeric_date_tags_must_be_overwrite(tags: &Vec<MyTag>,
                                             mode: &ModifyMode) -> Result<(), Error> {
    if exist_numeric_or_date(tags) && mode != &ModifyMode::Overwrite {
        Err(anyhow!("numeric or date constant value must work with \"--modify-mode overwrite\"."))
    } else {
        Ok(())
    }
}

fn exist_numeric_or_date(tags: &Vec<MyTag>) -> bool {
    tags.iter()
        .find(|e| { e.is_numeric() || e.is_date() })
        .is_some()
}

fn check_tags_type_value_type(tags: &Vec<MyTag>,
                              value: &ConstValueArgs) -> Result<(), Error> {
    match value {
        ConstValueArgs::Date { .. } =>
            if tags.iter()
                .find(|tag| !tag.is_date()).is_some() {
                Err(anyhow!("date constant value only work with date tags."))
            } else {
                Ok(())
            }
        ConstValueArgs::Num { .. } =>
            if tags.iter()
                .find(|tag| !tag.is_numeric()).is_some() {
                Err(anyhow!("numeric constant value only work with numeric tags."))
            } else {
                Ok(())
            }
        ConstValueArgs::Text { .. } =>
            if tags.iter()
                .find(|tag| !tag.is_text()).is_some() {
                Err(anyhow!("text constant value only work with text tags."))
            } else {
                Ok(())
            }
    }
}

fn check_value_is_ok(tags: &Vec<MyTag>, value: &ConstValueArgs) -> Result<(), Error> {
    match value {
        ConstValueArgs::Num { value, padding } => {
            if *value < MIN_NUMBER || *value > MAX_NUMBER {
                err_param_exceed_boundary("value")
            } else if *padding < MIN_PADDING || *padding > MAX_PADDING {
                err_param_exceed_boundary("padding")
            } else if (*value < MIN_NATURAL_NUMBER)
                && (tags.contains(&MyTag::TrackNumber)
                || tags.contains(&MyTag::TrackTotal)
                || tags.contains(&MyTag::DiscNumber)
                || tags.contains(&MyTag::DiscTotal)) {
                Err(anyhow!("Parameter: value must great or equal {}, when use tags: {:?}",
                    MIN_NATURAL_NUMBER, tags))
            } else {
                Ok(())
            }
        }
        ConstValueArgs::Date { value, format } => {
            let d = NaiveDate::parse_from_str(value, &format);
            if d.is_err() {
                err_param_exceed_boundary_err("value", anyhow!(d.err().unwrap()))
            } else {
                Ok(())
            }
        }
        _ => Ok(())
    }
}

fn err_param_exceed_boundary(param_name: &str) -> Result<(), Error> {
    Err(anyhow!("Parameter: {} exceed the boundary.", param_name))
}

fn err_param_exceed_boundary_err(param_name: &str, err: Error) -> Result<(), Error> {
    Err(anyhow!("Parameter: {} exceed the boundary. (error: {:?})", param_name, err))
}

fn numeric_is_valid(tag: &MyTag, numeric_value: u32) -> bool {
    match tag {
        MyTag::Year => numeric_value <= MAX_NUMBER,
        MyTag::TrackNumber => MIN_NATURAL_NUMBER <= numeric_value && numeric_value <= MAX_NUMBER,
        MyTag::TrackTotal => MIN_NATURAL_NUMBER <= numeric_value && numeric_value <= MAX_NUMBER,
        MyTag::DiscNumber => MIN_NATURAL_NUMBER <= numeric_value && numeric_value <= MAX_NUMBER,
        MyTag::DiscTotal => MIN_NATURAL_NUMBER <= numeric_value && numeric_value <= MAX_NUMBER,
        _ => false
    }
}