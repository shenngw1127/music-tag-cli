use std::io::{stdout, Write};
use std::path::{Path, PathBuf};

use anyhow::Error;

use crate::model::{ALL_TAGS, MyTag};
use crate::op::{Action, get_file_iterator, get_tags_from_args, get_where, MyValues, ReadAction, WalkAction};
use crate::where_clause::WhereClause;

pub struct ViewAction {
    it: Box<dyn Iterator<Item=PathBuf>>,
    tags: Vec<MyTag>,
    where_clause: Option<WhereClause>,
    with_properties: bool,
}

impl ViewAction {
    pub fn new<P>(dir: P,
                  tags: &[MyTag],
                  where_string: &Option<String>,
                  with_properties: bool) -> Result<Self, Error>
        where P: AsRef<Path>
    {
        let it = get_file_iterator(dir.as_ref())?;
        let tags = get_tags_from_args(tags, &ALL_TAGS)?;
        let where_clause = get_where(where_string)?;
        Ok(Self {
            it,
            tags,
            where_clause,
            with_properties,
        })
    }
}

impl Action for ViewAction {
    fn do_any(&mut self) -> Result<(), Error> {
        self.do_all()
    }
}

impl WalkAction for ViewAction {
    fn get_iterator(&mut self) -> &mut dyn Iterator<Item=PathBuf> {
        &mut self.it
    }

    fn do_one_file(&mut self, path: &Path) -> Result<bool, Error> {
        self.do_one_file_read(path)
    }

    fn get_where(&self) -> &Option<WhereClause> {
        &self.where_clause
    }

    fn get_tags(&self) -> &Vec<MyTag> {
        &self.tags
    }
}

impl ReadAction for ViewAction {
    fn with_properties(&self) -> bool {
        self.with_properties
    }

    fn get_content(&self, path: &Path) -> Result<Option<String>, Error> {
        let v = self.read_tags(path)?;

        let mut w = Vec::new();
        let success = output_text(&mut w, self.get_tags(), &v, path)?;
        if success {
            let s = String::from_utf8(w)?;
            Ok(Some(s))
        } else {
            Ok(None)
        }
    }

    fn do_output(&mut self, _path: &Path, content: &str) -> Result<bool, Error> {
        let stdout = stdout();
        let mut writer = stdout.lock();
        write!(writer, "{}", content)?;
        Ok(true)
    }
}

fn output_text<W, P>(writer: &mut W,
                     tags: &Vec<MyTag>,
                     v: &MyValues,
                     path: P) -> Result<bool, Error>
    where W: Write,
          P: AsRef<Path>
{
    if !v.is_empty_value() {
        writeln!(writer, "-- TAGS for {:?} --", path.as_ref())?;
        for tag in tags {
            let tag_name = tag.to_string();
            writeln!(writer, "{} {} - {}",
                     &tag_name,
                     " ".repeat(get_space_count(&tag_name)),
                     &v.get_text(tag).unwrap_or_default())?;
        }

        if !v.is_empty_properties() {
            let keys = v.get_prop_keys().unwrap();
            let len = keys.len();
            if len == 1 {
                writeln!(writer, "-- PROPERTIES for {:?} {} key --", path.as_ref(), len)?;
            } else {
                writeln!(writer, "-- PROPERTIES for {:?} {} keys --", path.as_ref(), len)?;
            }

            for key in keys {
                if let Some(values) = v.get_prop(key) {
                    writeln!(writer, "{}: {:?}", key, values)?;
                } else {
                    writeln!(writer, "{}:", key)?;
                }
            }
        }

        Ok(true)
    } else {
        Ok(false)
    }
}

const MAX_WIDTH: usize = 16 - 2;

fn get_space_count(name: &str) -> usize {
    let len = name.len();
    if MAX_WIDTH > len {
        MAX_WIDTH - len
    } else {
        0
    }
}
