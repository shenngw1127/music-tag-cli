use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Error};
use log::{error, info};

use crate::model::{FilenameExistPolicy, MyTag};
use crate::op::{get_file_iterator, get_new_path, get_tags_from_template, get_where};
use crate::op::{Action, MyValues, ReadAction, WalkAction};
use crate::util::path::combine_path;
use crate::where_clause::WhereClause;

pub struct RenAction {
    it: Box<dyn Iterator<Item=PathBuf>>,
    tags: Vec<MyTag>,
    dry_run: bool,
    where_clause: Option<WhereClause>,
    with_properties: bool,
    template: String,
    filename_exist_policy: FilenameExistPolicy,
    // for cache
    empty_value: String,
}

impl RenAction {
    pub fn new<P>(dir: P,
                  dry_run: bool,
                  where_string: &Option<String>,
                  template: &str,
                  filename_exist_policy: FilenameExistPolicy) -> Result<Self, Error>
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
            filename_exist_policy,
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

    fn tags(&self) -> &Vec<MyTag> {
        &self.tags
    }
}

impl ReadAction for RenAction {
    fn with_properties(&self) -> bool {
        self.with_properties
    }

    fn get_content(&self, path: &Path, v: &MyValues) -> Result<Option<String>, Error> {
        if !v.is_empty_value() {
            let mut result = self.template.clone();
            for tag in self.tags() {
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
        if let Some(p) = combine_path(path, content) {
            if path.eq(&p) {
                return Ok(true);
            }

            if let Some(ref new_path) = get_new_path(&p, self.filename_exist_policy) {
                return if !self.dry_run {
                    ren_real(self.filename_exist_policy, path, new_path)
                } else {
                    ren_dry_run(self.filename_exist_policy, path, new_path)
                };
            }
        }

        error!(
            "Could NOT get the new filename! path: {:?}, content: {}, filename_exist_policy: {:?}",
            path, & content, self.filename_exist_policy);
        Ok(false)
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

fn ren_real<P1, P2>(filename_exist_policy: FilenameExistPolicy,
                    path: P1,
                    new_path: P2) -> Result<bool, Error>
    where P1: AsRef<Path>,
          P2: AsRef<Path> {
    let path = path.as_ref();
    let new_path = new_path.as_ref();
    match filename_exist_policy {
        FilenameExistPolicy::Overwrite => {
            if new_path.exists() && new_path.is_file() {
                let _ = fs::remove_file(new_path);
            }
        }
        _ => {}
    }
    fs::rename(path, new_path).map_or_else(
        |e| {
            Err(anyhow!("Rename file {:?} to {:?} failed. (error: {:?})",
                            path, new_path, e))
        },
        |_| {
            info!("Rename file {:?} to {:?}", path, new_path);
            Ok(true)
        })
}

fn ren_dry_run<P1, P2>(filename_exist_policy: FilenameExistPolicy,
                       path: P1,
                       new_path: P2) -> Result<bool, Error>
    where P1: AsRef<Path>,
          P2: AsRef<Path> {
    let path = path.as_ref();
    let new_path = new_path.as_ref();
    if !new_path.exists() {
        info!("Rename file {:?} to {:?}", path, new_path);
        Ok(true)
    } else {
        match filename_exist_policy {
            FilenameExistPolicy::Overwrite if new_path.exists() && new_path.is_file() => {
                info!("Rename file {:?} to {:?}", path, new_path);
                Ok(true)
            }
            _ => Err(anyhow!("Rename file {:?} to {:?} failed! File already exists.",
                path, new_path))
        }
    }
}
