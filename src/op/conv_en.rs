use anyhow::Error;
use std::path::Path;
use titlecase::titlecase;

use crate::model::{ConvEnProfile, MyTag, TEXT_TAGS};
use crate::op::{Action, get_where, string_to_option, WalkAction, WriteAction, WriteTextAction};
use crate::op::tag_impl::TagImpl;
use crate::where_clause::WhereClause;

pub struct ConvEnAction<'a> {
    dir: &'a Path,
    dry_run: bool,
    tags: &'a Vec<MyTag>,
    where_clause: Option<WhereClause>,
    profile: &'a ConvEnProfile,
}

impl<'a> ConvEnAction<'a> {
    pub fn new(dir: &'a Path,
               dry_run: bool,
               tags: &'a Vec<MyTag>,
               where_string: &Option<String>,
               profile: &'a ConvEnProfile) -> Result<Self, Error> {
        let where_clause = get_where(where_string)?;
        Ok(ConvEnAction {
            dir,
            dry_run,
            tags: if !tags.is_empty() {
                tags
            } else {
                &TEXT_TAGS
            },
            where_clause,
            profile,
        })
    }
}

impl Action for ConvEnAction<'_> {
    fn do_dir(&self) -> Result<(), Error> {
        self.do_dir_walk()
    }

    fn do_file(&self) -> Result<(), Error> {
        self.do_file_impl()
    }

    fn op_name(&self) -> &'static str {
        "conv-en"
    }

    fn get_path(&self) -> &Path {
        self.dir
    }

    fn get_tags(&self) -> &Vec<MyTag> {
        self.tags
    }
}

impl WalkAction for ConvEnAction<'_> {
    fn do_one_file(&self, path: &Path) -> Result<(), Error> {
        self.do_one_file_write(path)
    }
}

impl WriteAction for ConvEnAction<'_> {
    fn is_dry_run(&self) -> bool {
        self.dry_run
    }

    fn set_tags_some(&self, t: &mut TagImpl) -> Result<(), Error> {
        self.set_tags_some_impl(t)
    }

    fn get_where(&self) -> &Option<WhereClause> {
        &self.where_clause
    }
}

impl WriteTextAction for ConvEnAction<'_> {
    fn get_new_text(&self, current: &Option<String>) -> Option<String> {
        if let Some(curr) = current {
            let new_v = match self.profile {
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

