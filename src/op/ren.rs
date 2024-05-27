use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Error};
use log::info;

use crate::model::MyTag;
use crate::op::{get_file_iterator, get_tags_from_template, get_where};
use crate::op::{Action, ReadAction, WalkAction};
use crate::where_clause::WhereClause;

pub struct RenAction {
    it: Box<dyn Iterator<Item=PathBuf>>,
    tags: Vec<MyTag>,
    dry_run: bool,
    where_clause: Option<WhereClause>,
    with_properties: bool,
    template: String,
    // for cache
    empty_value: String,
}

impl RenAction {
    pub fn new<P>(dir: P,
                  dry_run: bool,
                  where_string: &Option<String>,
                  template: &str) -> Result<Self, Error>
        where P: AsRef<Path>
    {
        let it = get_file_iterator(dir.as_ref())?;
        let tags = get_tags_from_template(template)?;
        let where_clause = get_where(where_string)?;
        let empty_value = get_empty_value(&tags, template);
        Ok(Self {
            it,
            tags,
            dry_run,
            where_clause,
            with_properties: false,
            template: template.to_owned(),
            empty_value,
        })
    }
}

impl Action for RenAction {
    fn do_any(&mut self) -> Result<(), Error> {
        self.do_all()
    }
}

impl WalkAction for RenAction {
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

fn get_new_path<P>(path: P,
                   stem: &str) -> Option<PathBuf>
    where P: AsRef<Path>
{
    let old_path = path.as_ref();
    let mut i: u16 = 0;
    loop {
        let mut filename = stem.to_owned();
        if i > 0 {
            filename.push_str(&format!("({})", i));
        }
        if let Some(ext) = old_path.extension() {
            filename.push('.');
            filename.push_str(&ext.to_string_lossy());
        }
        let mut new_path = PathBuf::from(old_path);
        new_path.set_file_name(&filename);

        if !new_path.exists() {
            break Some(new_path);
        } else {
            if i == u16::MAX {
                break None;
            }
            i = i + 1;
        }
    }
}

impl ReadAction for RenAction {
    fn with_properties(&self) -> bool {
        self.with_properties
    }

    fn get_content(&self, path: &Path) -> Result<Option<String>, Error> {
        let v = self.read_tags(path)?;
        if !v.is_empty_value() {
            let mut result = self.template.clone();
            for tag in self.get_tags() {
                let tag_name = &tag.to_string();
                result = result.replace(&format!("${{{}}}", tag_name),
                                        v.get_text(tag).unwrap_or_default());
            }
            if !result.eq(&self.empty_value)
                && !result.eq(&self.template)
                && !result.is_empty() {
                Ok(Some(result))
            } else {
                Err(anyhow!("File {:?} NOT contains any value for tags in --template \"{}\".",
                    path, &self.template))
            }
        } else {
            Err(anyhow!("Not found any tag in file {:?}", path))
        }
    }

    fn do_output(&mut self, path: &Path, content: &str) -> Result<bool, Error> {
        if let Some(ref new_path) = get_new_path(path, &content) {
            if !self.dry_run {
                return fs::rename(path, new_path)
                    .map(|_| {
                        info!("Rename file {:?} to {:?}", path, new_path);
                        true
                    })
                    .map_err(|e| anyhow!("Rename file {:?} to {:?} failed. (error: {:?})",
                        path, new_path, e));
            } else {
                info!("Rename file {:?} to {:?}", path, new_path);
                Ok(true)
            }
        } else {
            Ok(false)
        }
    }
}

fn get_empty_value(tags: &[MyTag], template: &str) -> String {
    let mut result = template.to_owned();
    for tag in tags {
        let tag_name = &tag.to_string();
        result = result.replace(&format!("${{{}}}", tag_name), "");
    }
    result
}