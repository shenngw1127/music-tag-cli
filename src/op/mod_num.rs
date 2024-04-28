use anyhow::Error;
use log::warn;
use std::path::Path;

use crate::model::{CalcMethod, MyTag};
use crate::op::{check_tags_not_empty, Action, MAX_NUMBER, WalkAction, WriteAction, WriteNumAction, get_where};
use crate::op::tag_impl::TagImpl;
use crate::where_clause::WhereClause;

pub struct ModNumAction<'a> {
    dir: &'a Path,
    dry_run: bool,
    tags: &'a Vec<MyTag>,
    where_clause: Option<WhereClause>,
    calc_method: &'a CalcMethod,
    operand: &'a u32,
    padding: &'a usize,
}

impl<'a> ModNumAction<'a> {
    pub fn new(dir: &'a Path,
               dry_run: bool,
               tags: &'a Vec<MyTag>,
               where_string: &Option<String>,
               calc_method: &'a CalcMethod,
               operand: &'a u32,
               padding: &'a usize) -> Result<Self, Error> {
        let where_clause = get_where(where_string)?;
        Self::check(tags)
            .map(|_| {
                ModNumAction {
                    dir,
                    dry_run,
                    tags,
                    where_clause,
                    calc_method,
                    operand,
                    padding,
                }
            })
    }

    fn check(tags: &Vec<MyTag>) -> Result<(), Error> {
        check_tags_not_empty(tags)
    }
}

impl Action for ModNumAction<'_> {
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

impl WalkAction for ModNumAction<'_> {
    fn do_one_file(&self, path: &Path) -> Result<(), Error> {
        self.do_one_file_write(path)
    }
}

impl WriteAction for ModNumAction<'_> {
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

impl WriteNumAction for ModNumAction<'_> {
    fn get_padding(&self) -> &usize {
        self.padding
    }

    fn get_new_numeric(&self, current: &Option<u32>) -> Option<u32> {
        if let Some(curr) = current {
            let new_v = match self.calc_method {
                CalcMethod::Increase => *curr + *self.operand,
                CalcMethod::Decrease => *curr - *self.operand,
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
