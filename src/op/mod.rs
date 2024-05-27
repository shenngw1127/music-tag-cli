extern crate lazy_static;

use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;
use std::{fs, iter};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Error};
use chrono::NaiveDate;
use itertools::Itertools;
use log::{debug, error, warn};
use walkdir::WalkDir;

use crate::model::{ConstValue, ModifyMode, MyTag};
use crate::op::tag_impl::ReadWriteTag;
use crate::where_clause::WhereClause;

pub use self::conv_en::ConvEnAction;
pub use self::conv_utf8::ConvUtf8Action;
pub use self::conv_zh::ConvZhAction;
pub use self::exp::ExpAction;
pub use self::imp::ImpAction;
pub use self::mod_num::ModNumAction;
pub use self::mod_text_const::ModTextConstAction;
pub use self::mod_text_regex::ModTextRegexAction;
pub use self::set_const::SetConstAction;
pub use self::set_name::{SetNameAction, get_tags_from_template};
pub use self::set_seq::SetSeqAction;
pub use self::ren::RenAction;
pub use self::view::ViewAction;

pub use self::tag_impl::ReadTag;

use self::tag_impl::{is_available_suffix, TagImpl};

mod conv_en;
mod conv_utf8;
mod conv_zh;
mod exp;
mod imp;
mod mod_num;
mod mod_text_const;
mod mod_text_regex;
mod set_const;
mod set_name;
mod set_seq;
mod ren;
mod view;

mod tag_impl;

#[derive(Debug)]
enum MyValue {
    Text(String),
    Num(u32, String),
    None,
}

#[derive(Debug)]
struct MyValues<'a> {
    raw: Option<HashMap<&'a MyTag, MyValue>>,
    properties: Option<BTreeMap<String, Vec<String>>>,
}

impl MyValues<'_> {
    fn is_empty_value(&self) -> bool {
        if let Some(r) = &self.raw {
            r.is_empty()
        } else {
            true
        }
    }

    fn is_empty_properties(&self) -> bool {
        if let Some(p) = &self.properties {
            p.is_empty()
        } else {
            true
        }
    }
    fn get_text(&self, tag: &MyTag) -> Option<&str> {
        if let Some(r) = &self.raw {
            if let Some(v) = r.get(tag) {
                match v {
                    MyValue::Text(s) => Some(s),
                    MyValue::Num(_, s) => Some(s),
                    MyValue::None => None,
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    fn get_num(&self, tag: &MyTag) -> Option<u32> {
        if let Some(r) = &self.raw {
            if let Some(v) = r.get(tag) {
                match v {
                    MyValue::Num(u, _) => Some(*u),
                    _ => None
                }
            } else {
                None
            }
        } else {
            None
        }
    }

    fn get_prop_keys(&self) -> Option<Vec<&String>> {
        if let Some(p) = &self.properties {
            Some(p.keys().clone().collect::<Vec<&String>>())
        } else {
            None
        }
    }

    fn get_prop(&self, key: &str) -> Option<&Vec<String>> {
        if let Some(p) = &self.properties {
            p.get(key)
        } else {
            None
        }
    }
}

pub trait Action {
    fn do_any(&mut self) -> Result<(), Error>;
}

trait WalkAction: Action {
    fn do_all(&mut self) -> Result<(), Error> {
        while let Some(res) = self.do_next() {
            res.err()
                .map(|e| error!("Error: {}", e));
        }

        Ok(())
    }

    fn do_next(&mut self) -> Option<Result<bool, Error>> {
        let it = self.get_iterator();
        if let Some(ref path) = it.next() {
            return Some(self.do_one_file(path));
        }
        None
    }

    fn get_iterator(&mut self) -> &mut dyn Iterator<Item=PathBuf>;

    fn do_one_file(&mut self, path: &Path) -> Result<bool, Error>;

    fn get_where(&self) -> &Option<WhereClause>;

    fn check_where(&self, t: &dyn ReadTag) -> Result<bool, Error> {
        check_where(self.get_where(), t)
    }

    fn get_tags(&self) -> &Vec<MyTag>;
}

fn check_where(where_clause: &Option<WhereClause>, t: &dyn ReadTag) -> Result<bool, Error> {
    if let Some(where_clause) = where_clause {
        match where_clause.check(t) {
            Some(t) => Ok(t),
            None => Err(anyhow!("Some error in where clause.")),
        }
    } else {
        // None: don't check, equals check ok
        Ok(true)
    }
}

trait SeqAction: Action {
    fn do_all(&mut self) -> Result<(), Error> {
        while let Some(res) = self.do_next() {
            res.err()
                .map(|e| error!("Error: {}", e));
        }

        Ok(())
    }

    fn do_next(&mut self) -> Option<Result<(), Error>>; // TODO default impl

    fn get_iterator(&mut self) -> &mut dyn Iterator<Item=PathBuf>;

    fn do_one_file(&mut self, path: &Path, seq: &Option<&str>) -> Result<(), Error>;

    fn get_tags(&self) -> &Vec<MyTag>;
}

fn sorted_filtered_files<P>(path: P) -> Result<Vec<PathBuf>, Error>
    where P: AsRef<Path>
{
    let mut paths: Vec<_> = fs::read_dir(path)?
        .into_iter()
        .flatten()
        .map(|e| e.path())
        .filter(|p|
            p.is_file() && is_available_suffix(&p.to_string_lossy()))
        .collect();
    paths.sort();
    Ok(paths)
}

trait ReadAction: WalkAction {
    fn with_properties(&self) -> bool;

    fn do_one_file_read(&mut self, path: &Path) -> Result<bool, Error> {
        if let Some(content) = self.get_content(path)? {
            debug!("content: {}", &content);
            self.do_output(path, &content)
        } else {
            Ok(false)
        }
    }

    fn get_content(&self, path: &Path) -> Result<Option<String>, Error>;

    fn do_output(&mut self, path: &Path, content: &str) -> Result<bool, Error>;

    fn read_tags(&self, path: &Path) -> Result<MyValues, Error> {
        let tag_impl = TagImpl::new(&path, true)?;
        self.get_tags_some(&tag_impl)
    }

    fn get_tags_some(&self, t: &dyn ReadTag) -> Result<MyValues, Error> {
        let tags = self.get_tags();
        if tags.is_empty() {
            return Ok(MyValues { raw: None, properties: None });
        }

        if !self.check_where(t)? {
            return Ok(MyValues { raw: None, properties: None });
        }

        let mut map: HashMap<&MyTag, MyValue> = HashMap::with_capacity(tags.len());
        for tag in tags {
            let v = get_tag(t, tag);
            map.insert(tag, v);
        }

        let properties = if self.with_properties() {
            get_properties(t)
        } else {
            None
        };

        Ok(MyValues { raw: Some(map), properties })
    }
}

trait WriteAction: WalkAction {
    fn is_dry_run(&self) -> bool;

    fn do_one_file_write(&self, path: &Path) -> Result<bool, Error> {
        let mut tag_impl = TagImpl::new(&path, self.is_dry_run())?;
        self.set_tags_some(&mut tag_impl)
    }

    fn set_tags_some(&self, t: &mut dyn ReadWriteTag) -> Result<bool, Error>;
}

trait WriteTextAction: WriteAction {
    fn set_text_tag(&self, t: &mut dyn ReadWriteTag, tag: &MyTag) -> bool {
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

    fn set_tags_some_impl(&self, t: &mut dyn ReadWriteTag) -> Result<bool, Error> {
        if self.get_tags().is_empty() {
            return Ok(false);
        }

        if !self.check_where(t.as_dyn_read_tag_mut())? {
            return Ok(false);
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
            t.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

trait WriteNumAction: WriteAction {
    fn set_tags_some_impl(&self, t: &mut dyn ReadWriteTag) -> Result<bool, Error> {
        if self.get_tags().is_empty() {
            return Ok(false);
        }

        if !self.check_where(t.as_dyn_read_tag_mut())? {
            return Ok(false);
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
            t.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn set_numeric_tag(&self, t: &mut dyn ReadWriteTag, tag: &MyTag) -> bool {
        let current = t.get_numeric_tag(tag);
        let new_value = self.get_new_numeric(&current);

        if let Some(new_v) = new_value {
            if numeric_is_valid(tag, new_v) {
                t.write_numeric_tag(tag, new_v, self.get_padding());
                true
            } else {
                warn!("Numeric value {} exceed the boundary.", new_v);
                false
            }
        } else {
            false
        }
    }

    fn get_padding(&self) -> usize;

    fn get_new_numeric(&self, current: &Option<u32>) -> Option<u32>;
}

trait WriteAllAction: WriteAction {
    fn set_tags_some_impl(&self, t: &mut dyn ReadWriteTag) -> Result<bool, Error> {
        if self.get_tags().is_empty() {
            return Ok(false);
        }

        if !self.check_where(t.as_dyn_read_tag_mut())? {
            return Ok(false);
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
            t.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    fn set_text_tag(&self, t: &mut dyn ReadWriteTag, tag: &MyTag) -> bool;

    fn set_numeric_tag(&self, t: &mut dyn ReadWriteTag, tag: &MyTag) -> bool;

    fn set_date_tag(&self, t: &mut dyn ReadWriteTag, tag: &MyTag) -> bool;
}

trait SeqWriteAction: SeqAction {
    fn is_dry_run(&self) -> bool;

    fn do_one_file_write(&self, path: &Path, seq: &Option<&str>) -> Result<(), Error> {
        let mut tag_impl = TagImpl::new(&path, self.is_dry_run())?;
        self.set_tags_some(&mut tag_impl, seq)
    }

    fn set_tags_some(&self,
                     t: &mut dyn ReadWriteTag,
                     other: &Option<&str>) -> Result<(), Error>;
}

fn get_tags_from_args(tags: &[MyTag], default: &[MyTag]) -> Result<Vec<MyTag>, Error> {
    let res = if !tags.is_empty() {
        tags.into_iter().unique()
            .map(|t| t.clone())
            .collect::<Vec<_>>()
    } else {
        default.to_vec()
    };

    if !res.is_empty() {
        Ok(res)
    } else {
        Err(anyhow!("You must specified one tag at least for \"--tags\" parameter!"))
    }
}

fn get_where(where_string: &Option<String>) -> Result<Option<WhereClause>, Error> {
    if let Some(where_string) = where_string {
        WhereClause::new(where_string)
            .map(|t| Some(t))
            .map_err(|s| anyhow!("{}", &s))
    } else {
        Ok(None)
    }
}

fn get_file_iterator<P>(dir: P) -> Result<Box<dyn Iterator<Item=PathBuf>>, Error>
    where P: AsRef<Path>
{
    let path = dir.as_ref();
    if path.is_dir() {
        debug!("dir: {:?}", path);
        Ok(Box::new(
            WalkDir::new(path)
                .follow_links(false)
                .into_iter()
                .flatten()
                .filter_map(|e| {
                    if let Some(m) = e.metadata().ok() {
                        Some((e, m))
                    } else {
                        None
                    }
                })
                .filter(
                    |(e, m)|
                        m.is_file() && is_available_suffix(&e.path().to_string_lossy()))
                .map(|(e, _)| e.into_path())
        ))
    } else if path.is_file() {
        debug!("file: {:?}", path);
        Ok(Box::new(iter::once(PathBuf::from(path))))
    } else {
        Err(anyhow!("Could NOT perform action on path: {:?}. Please check the path.", path))
    }
}

fn get_dir_iterator<P>(dir: P) -> Result<(Box<dyn Iterator<Item=PathBuf>>, Option<PathBuf>), Error>
    where P: AsRef<Path>
{
    let path = dir.as_ref();
    if path.is_dir() {
        debug!("dir: {:?}", path);
        Ok(
            (Box::new(
                WalkDir::new(path)
                    .follow_links(false)
                    .into_iter()
                    .flatten()
                    .filter_map(|e| {
                        if let Some(m) = e.metadata().ok() {
                            Some((e, m))
                        } else {
                            None
                        }
                    })
                    .filter(|(_, m)| m.is_dir())
                    .map(|(e, _)| e.into_path())),
             None)
        )
    } else if path.is_file() {
        debug!("file: {:?}", path);
        let file_path = PathBuf::from(path);
        let parent_path = PathBuf::from(file_path.parent().unwrap());
        Ok(
            (Box::new(iter::once(parent_path)),
             Some(file_path))
        )
    } else {
        Err(anyhow!("Could NOT perform action on path: {:?}. Please check the path.", path))
    }
}

// TODO: 改为macro
fn string_to_option(new_value: String, value: &str) -> Option<String> {
    if !new_value.eq(value) {
        Some(new_value)
    } else {
        None
    }
}

fn get_tag(t: &dyn ReadTag, tag: &MyTag) -> MyValue {
    if tag.is_text() || tag.is_date() {
        if let Some(s) = t.get_text_tag(tag) {
            return MyValue::Text(s);
        }
        MyValue::None
    } else if tag.is_numeric() {
        if let Some(s) = t.get_numeric_tag_string(tag) {
            if let Some(u) = s.parse::<u32>().ok() {
                return MyValue::Num(u, s);
            }
        }
        MyValue::None
    } else {
        error!("Unknown tag type for tag: {}", tag);
        MyValue::None
    }
}

fn get_properties<T: ReadTag + ?Sized>(t: &T) -> Option<BTreeMap<String, Vec<String>>> {
    t.get_property_keys()
        .map_or_else(
            |e| {
                error!("{:?}", e);
                None
            },
            |keys| {
                Some(keys.iter()
                    .map(|k| {
                        if let Some(v) = t.get_property(k).ok() {
                            (k.clone(), v)
                        } else {
                            (k.clone(), vec![])
                        }
                    })
                    .collect::<BTreeMap<String, Vec<String>>>())
            },
        )
}

const MIN_PADDING: usize = 1;
const MAX_PADDING: usize = 9;
const MIN_NUMBER: u32 = 0;
const MIN_NATURAL_NUMBER: u32 = 1;
const MAX_NUMBER: u32 = u16::MAX as u32;

fn check_numeric_date_tags_must_be_overwrite(tags: &[MyTag],
                                             mode: &ModifyMode) -> Result<(), Error> {
    if exist_numeric_or_date(tags) && mode != &ModifyMode::Overwrite {
        Err(anyhow!("numeric or date constant value must work with \"--modify-mode overwrite\"."))
    } else {
        Ok(())
    }
}

fn exist_numeric_or_date(tags: &[MyTag]) -> bool {
    tags.iter()
        .find(|e| { e.is_numeric() || e.is_date() })
        .is_some()
}

fn check_tags_type_value_type(tags: &[MyTag],
                              value: &ConstValue) -> Result<(), Error> {
    match value {
        ConstValue::Date { .. } =>
            if tags.iter()
                .find(|tag| !tag.is_date()).is_some() {
                Err(anyhow!("date constant value only work with date tags."))
            } else {
                Ok(())
            }
        ConstValue::Num { .. } =>
            if tags.iter()
                .find(|tag| !tag.is_numeric()).is_some() {
                Err(anyhow!("numeric constant value only work with numeric tags."))
            } else {
                Ok(())
            }
        ConstValue::Text { .. } =>
            if tags.iter()
                .find(|tag| !tag.is_text()).is_some() {
                Err(anyhow!("text constant value only work with text tags."))
            } else {
                Ok(())
            }
    }
}

fn check_value_is_ok(tags: &[MyTag], value: &ConstValue) -> Result<(), Error> {
    match value {
        ConstValue::Num { value, padding } => {
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
        ConstValue::Date { value, format } => {
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

fn check_file_not_exists<P>(path: P) -> Result<(), Error>
    where P: AsRef<Path>
{
    let path = path.as_ref();
    if path.exists() {
        return Err(anyhow!("{:?} exists, could NOT process.", path));
    }
    Ok(())
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