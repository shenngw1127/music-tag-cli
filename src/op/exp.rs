use std::collections::HashMap;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Error};
use log::{debug, error};

use crate::model::{ALL_TAGS, FilenameExistPolicy, MyTag};
use crate::op::{check_where, get_file_iterator, get_new_path, get_properties, get_tags_from_args,
                get_tags_value, get_where};
use crate::op::{Action, MyValue, MyValues, ReadAction, ReadTag, WalkAction};
use crate::op::tag_impl::TagImpl;
use crate::where_clause::WhereClause;

const BUFFER_SIZE: usize = 4 * 1024;

pub struct ExpAction {
    it: Box<dyn Iterator<Item=PathBuf>>,
    writer: Box<dyn Write>,
    with_properties: bool,
    tags: Vec<MyTag>,
    where_clause: Option<WhereClause>,
    // state
    is_first: bool,
}

impl ExpAction {
    pub fn new<P: AsRef<Path>>(dir: P,
                               tags: &[MyTag],
                               where_string: &Option<String>,
                               with_properties: bool,
                               output_file: P,
                               filename_exist_policy: FilenameExistPolicy) -> Result<Self, Error> {
        let it = get_file_iterator(dir.as_ref())?;
        let tags = get_tags_from_args(tags, &ALL_TAGS)?;
        let where_clause = get_where(where_string)?;
        let writer = get_file_writer(output_file, filename_exist_policy)?;
        Ok(Self {
            it,
            writer,
            with_properties,
            tags,
            where_clause,
            is_first: true,
        })
    }

    #[inline]
    fn do_start(&mut self) -> Result<(), Error> {
        write!(self.writer, "[")?;
        Ok(())
    }

    #[inline]
    fn do_end(&mut self) -> Result<(), Error> {
        write!(self.writer, "]")?;
        Ok(())
    }

    #[inline]
    fn do_sep(&mut self) -> Result<(), Error> {
        writeln!(self.writer, ",")?;
        Ok(())
    }
}

fn get_values<'a, P>(path: P,
                     tags: &'a Vec<MyTag>,
                     where_clause: &Option<WhereClause>,
                     with_properties: bool) -> Result<MyValues<'a>, Error>
    where P: AsRef<Path>
{
    let tag_impl = TagImpl::new(&path, true)?;
    read_tags(&tag_impl, tags, where_clause, with_properties)
}

fn read_tags<'a>(t: &dyn ReadTag,
                 tags: &'a Vec<MyTag>,
                 where_clause: &Option<WhereClause>,
                 with_properties: bool) -> Result<MyValues<'a>, Error> {
    if tags.is_empty() {
        return Ok(MyValues { raw: None, properties: None });
    }

    if !check_where(where_clause, t.as_dyn_read_tag())? {
        return Ok(MyValues { raw: None, properties: None });
    }

    let mut map: HashMap<&MyTag, MyValue> = HashMap::with_capacity(tags.len());
    for tag in tags.iter() {
        let v = get_tags_value(t, tag);
        map.insert(tag, v);
    }

    let properties = if with_properties {
        get_properties(t)
    } else {
        None
    };

    Ok(MyValues { raw: Some(map), properties })
}

fn get_file_writer<P>(path: P,
                      filename_exist_policy: FilenameExistPolicy) -> Result<Box<dyn Write>, Error>
    where P: AsRef<Path> {
    let path = path.as_ref();
    if let Some(path) = get_new_path(path, filename_exist_policy) {
        let f = File::create(path)?;
        let writer = BufWriter::with_capacity(BUFFER_SIZE, f);
        Ok(Box::new(writer))
    } else {
        return Err(anyhow!("{:?} exists, could NOT process. filename_exist_policy: {:?}",
            path, filename_exist_policy));
    }
}

impl Action for ExpAction {
    fn do_any(&mut self) -> Result<(), Error> {
        self.do_all()
    }
}

impl WalkAction for ExpAction {
    fn do_all(&mut self) -> Result<(), Error> {
        self.do_start()?;

        while let Some(res) = self.do_next() {
            match res {
                Ok(last_result) => {
                    if last_result {
                        if self.is_first {
                            self.is_first = false;
                        }
                    }
                }
                Err(e) => {
                    error!("Error: {}", e);
                }
            }
        }

        self.do_end()?;

        Ok(())
    }

    fn get_iterator(&mut self) -> &mut dyn Iterator<Item=PathBuf> {
        &mut self.it
    }

    fn do_one_file(&mut self, path: &Path) -> Result<bool, Error> {
        self.do_one_file_read(path)
    }

    fn get_where(&self) -> &Option<WhereClause> {
        &self.where_clause
    }

    fn tags(&self) -> &Vec<MyTag> {
        &self.tags
    }
}

impl ReadAction for ExpAction {
    fn with_properties(&self) -> bool {
        self.with_properties
    }

    fn do_one_file_read(&mut self, path: &Path) -> Result<bool, Error> {
        let v = get_values(path, &self.tags, &self.where_clause, self.with_properties)?;
        match self.get_content(path, &v)? {
            Some(content) => {
                debug!("content: {}", &content);
                if !self.is_first {
                    self.do_sep()?;
                }
                self.do_output(path, &content)
            }
            None => Ok(false),
        }
    }

    fn get_content(&self, path: &Path, v: &MyValues) -> Result<Option<String>, Error> {
        get_json(&v, path, &self.tags)
    }

    fn do_output(&mut self, _path: &Path, content: &str) -> Result<bool, Error> {
        let writer = &mut self.writer;
        write!(writer, "{}", content)?;
        Ok(true)
    }
}

fn get_json<P>(v: &MyValues,
               path: P,
               tags: &Vec<MyTag>) -> Result<Option<String>, Error>
    where P: AsRef<Path>,
{
    let mut w = Vec::new();
    let success = output_json(&mut w, v, path, tags)?;
    if success {
        let s = String::from_utf8(w)?;
        Ok(Some(s))
    } else {
        Ok(None)
    }
}

fn output_json<W, P>(writer: &mut W,
                     v: &MyValues,
                     path: P,
                     tags: &Vec<MyTag>) -> Result<bool, Error>
    where W: Write,
          P: AsRef<Path>,
{
    if v.is_empty_value() {
        return Ok(false);
    }

    writeln!(writer, "{{")?;

    writeln!(writer, "  \"{}\": {:?},", "path", path.as_ref())?;
    {
        output_json_tags(writer, tags, &v)?;
    }

    if !v.is_empty_properties() {
        writeln!(writer, ",")?;
    } else {
        writeln!(writer)?;
    }

    if !v.is_empty_properties() {
        let keys = v.get_prop_keys().unwrap();
        output_json_props(writer, v, keys)?;
    }

    write!(writer, "}}")?;
    Ok(true)
}

fn output_json_props<W>(writer: &mut W,
                        v: &MyValues,
                        keys: Vec<&String>) -> Result<(), Error>
    where W: Write
{
    writeln!(writer, "  \"props\": {{")?;
    let mut it = keys.iter().peekable();
    while let Some(key) = it.next() {
        if let Some(values) = v.get_prop(key) {
            write!(writer, "    \"{}\": {:?}", key, values)?;
        }

        if it.peek().is_some() {
            writeln!(writer, ",")?;
        } else {
            writeln!(writer)?;
        }
    }
    writeln!(writer, "  }}")?;
    Ok(())
}

fn output_json_tags<W>(writer: &mut W,
                       tags: &Vec<MyTag>,
                       v: &&MyValues) -> Result<(), Error>
    where W: Write
{
    writeln!(writer, "  \"tags\": {{")?;
    let mut it = tags.iter().peekable();
    while let Some(tag) = it.next() {
        let tag_name = tag.to_string();
        if tag.is_text() || tag.is_date() {
            if let Some(s) = v.get_text(tag) {
                write!(writer, "    \"{}\": \"{}\"", &tag_name, &escape(s))?;
            } else {
                write!(writer, "    \"{}\": null", &tag_name)?;
            }
        } else if tag.is_numeric() {
            if let Some(n) = v.get_num(tag) {
                write!(writer, "    \"{}\": {}", &tag_name, n)?;
            } else {
                write!(writer, "    \"{}\": null", &tag_name)?;
            }
        } else {
            // do_nothing
        }

        if it.peek().is_some() {
            writeln!(writer, ",")?;
        } else {
            writeln!(writer)?;
        }
    }
    write!(writer, "  }}")?;
    Ok(())
}

fn escape(src: &str) -> String {
    let mut escaped = String::with_capacity(src.len());
    for c in src.chars() {
        match c {
            '\x08' => escaped += "\\b",
            '\x0c' => escaped += "\\f",
            '\n' => escaped += "\\n",
            '\r' => escaped += "\\r",
            '\t' => escaped += "\\t",
            '"' => escaped += "\\\"",
            '\\' => escaped += "\\\\",
            c => escaped.push(c),
        }
    }
    escaped
}
