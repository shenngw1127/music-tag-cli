use std::path::{Path, PathBuf};

use anyhow::Error;
use titlecase::titlecase;

use crate::model::{ConvEnProfile, MyTag, TEXT_TAGS};
use crate::op::{get_file_iterator, get_tags_from_args, get_where, string_to_option};
use crate::op::{Action, WalkAction, WriteAction, WriteTextAction};
use crate::op::tag_impl::ReadWriteTag;
use crate::where_clause::WhereClause;

pub struct ConvEnAction {
    it: Box<dyn Iterator<Item=PathBuf>>,
    dry_run: bool,
    tags: Vec<MyTag>,
    where_clause: Option<WhereClause>,
    profile: ConvEnProfile,
}

impl ConvEnAction {
    pub fn new<P>(dir: P,
                  dry_run: bool,
                  tags: &[MyTag],
                  where_string: &Option<String>,
                  profile: &ConvEnProfile) -> Result<Self, Error>
        where P: AsRef<Path>
    {
        let it = get_file_iterator(dir.as_ref())?;
        let tags = get_tags_from_args(tags, &TEXT_TAGS)?;
        let where_clause = get_where(where_string)?;
        Ok(Self {
            it,
            dry_run,
            tags,
            where_clause,
            profile: profile.clone(),
        })
    }
}

impl Action for ConvEnAction {
    fn do_any(&mut self) -> Result<(), Error> {
        self.do_all()
    }
}

impl WalkAction for ConvEnAction {
    fn get_iterator(&mut self) -> &mut dyn Iterator<Item=PathBuf> {
        &mut self.it
    }

    fn do_one_file(&mut self, path: &Path) -> Result<bool, Error> {
        self.do_one_file_write(path)
    }

    fn get_where(&self) -> &Option<WhereClause> {
        &self.where_clause
    }

    fn get_tags(&self) -> &Vec<MyTag> {
        &self.tags
    }
}

impl WriteAction for ConvEnAction {
    fn is_dry_run(&self) -> bool {
        self.dry_run
    }

    fn set_tags_some(&self, t: &mut dyn ReadWriteTag) -> Result<bool, Error> {
        self.set_tags_some_impl(t)
    }
}

impl WriteTextAction for ConvEnAction {
    fn get_new_text(&self, current: &Option<String>) -> Option<String> {
        if let Some(curr) = current {
            let new_v = match &self.profile {
                ConvEnProfile::Lowercase => curr.to_lowercase(),
                ConvEnProfile::Uppercase => curr.to_uppercase(),
                ConvEnProfile::Titlecase => titlecase(curr),
            };

            string_to_option(new_v, curr)
        } else {
            None
        }
    }
}

