use std::path::{Path, PathBuf};

use anyhow::Error;
use log::warn;

use crate::model::{CalcMethod, EMPTY_TAGS, MyTag};
use crate::op::{get_file_iterator, get_tags_from_args, get_where, MAX_NUMBER};
use crate::op::{Action, WalkAction, WriteAction, WriteNumAction};
use crate::op::tag_impl::ReadWriteTag;
use crate::where_clause::WhereClause;

pub struct ModNumAction {
    it: Box<dyn Iterator<Item=PathBuf>>,
    dry_run: bool,
    tags: Vec<MyTag>,
    where_clause: Option<WhereClause>,
    calc_method: CalcMethod,
    operand: u32,
    padding: usize,
}

impl ModNumAction {
    pub fn new<P>(dir: P,
                  dry_run: bool,
                  tags: &[MyTag],
                  where_string: &Option<String>,
                  calc_method: &CalcMethod,
                  operand: u32,
                  padding: usize) -> Result<Self, Error>
        where P: AsRef<Path>
    {
        let it = get_file_iterator(dir.as_ref())?;
        let where_clause = get_where(where_string)?;
        let tags = get_tags_from_args(tags, &EMPTY_TAGS)?;
        Ok(Self {
            it,
            dry_run,
            tags,
            where_clause,
            calc_method: calc_method.clone(),
            operand,
            padding,
        })
    }
}

impl Action for ModNumAction {
    fn do_any(&mut self) -> Result<(), Error> {
        self.do_all()
    }
}

impl WalkAction for ModNumAction {
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

impl WriteAction for ModNumAction {
    fn is_dry_run(&self) -> bool {
        self.dry_run
    }

    fn set_tags_some(&self, t: &mut dyn ReadWriteTag) -> Result<bool, Error> {
        self.set_tags_some_impl(t)
    }
}

impl WriteNumAction for ModNumAction {
    fn get_padding(&self) -> usize {
        self.padding
    }

    fn get_new_numeric(&self, current: &Option<u32>) -> Option<u32> {
        if let Some(curr) = current {
            let new_v = match &self.calc_method {
                CalcMethod::Increase => *curr + self.operand,
                CalcMethod::Decrease => *curr - self.operand,
            };
            if new_v < MAX_NUMBER {
                Some(new_v)
            } else {
                warn!("Numeric value {} exceed the boundary.", new_v);
                None
            }
        } else {
            None
        }
    }
}
