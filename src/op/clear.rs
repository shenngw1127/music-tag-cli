use std::path::{Path, PathBuf};

use anyhow::Error;

use crate::model::{EMPTY_TAGS, MyTag};
use crate::op::{get_file_iterator, get_tags_from_args, get_where};
use crate::op::{Action, WalkAction, WriteAction, WriteAllAction};
use crate::op::tag_impl::ReadWriteTag;
use crate::where_clause::WhereClause;

pub struct ClearAction {
    it: Box<dyn Iterator<Item=PathBuf>>,
    dry_run: bool,
    tags: Vec<MyTag>,
    where_clause: Option<WhereClause>,
}

impl ClearAction {
    pub fn new<P>(dir: P,
                  dry_run: bool,
                  tags: &[MyTag],
                  where_string: &Option<String>) -> Result<Self, Error>
        where P: AsRef<Path>
    {
        let it = get_file_iterator(dir.as_ref())?;
        let tags = get_tags_from_args(tags, &EMPTY_TAGS)?;
        let where_clause = get_where(where_string)?;
        Ok(
            Self {
                it,
                dry_run,
                tags,
                where_clause,
            })
    }
}

impl Action for ClearAction {
    fn do_any(&mut self) -> Result<(), Error> {
        self.do_all()
    }
}

impl WalkAction for ClearAction {
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

impl WriteAction for ClearAction {
    fn is_dry_run(&self) -> bool {
        self.dry_run
    }

    fn write_tags(&self, t: &mut dyn ReadWriteTag) -> Result<bool, Error> {
        self.write_tags_impl(t)
    }
}

impl WriteAllAction for ClearAction {
    fn set_text_tag(&self, t: &mut dyn ReadWriteTag, tag: &MyTag) -> bool {
        t.clear_tag(tag);
        true
    }

    fn set_numeric_tag(&self, t: &mut dyn ReadWriteTag, tag: &MyTag) -> bool {
        t.clear_tag(tag);
        true
    }

    fn set_date_tag(&self, t: &mut dyn ReadWriteTag, tag: &MyTag) -> bool {
        t.clear_tag(tag);
        true
    }
}

