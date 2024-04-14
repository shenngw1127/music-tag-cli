use anyhow::{anyhow, Error};
use regex::{Regex, RegexBuilder};
use std::borrow::Cow;
use std::path::Path;

use crate::model::{MyTag, TEXT_TAGS};
use crate::op::tag_impl::TagImpl;
use crate::op::{Action, WalkAction, WriteAction, WriteTextAction};

pub struct ModTextRegexAction<'a> {
    dir: &'a Path,
    dry_run: bool,
    tags: &'a Vec<MyTag>,
    re: Regex,
    to: &'a str,
}

impl<'a> ModTextRegexAction<'a> {
    pub fn new(dir: &'a Path,
               dry_run: bool,
               tags: &'a Vec<MyTag>,
               from: &'a str,
               ignore_case: &'a bool,
               to: &'a str) -> Result<ModTextRegexAction<'a>, Error> {
        let re = Self::get_regex(from, ignore_case)?;
        Ok(ModTextRegexAction {
            dir,
            dry_run,
            tags: if !tags.is_empty() {
                tags
            } else {
                &TEXT_TAGS
            },
            re,
            to,
        })
    }

    fn get_regex(from: &str, ignore_case: &bool) -> Result<Regex, Error> {
        RegexBuilder::new(from)
            .case_insensitive(*ignore_case)
            .unicode(true)
            .build()
            .map_err(|e| { anyhow!(e) })
    }
}

impl Action for ModTextRegexAction<'_> {
    fn do_dir(&self) -> Result<(), Error> {
        self.do_dir_walk()
    }

    fn do_file(&self) -> Result<(), Error> {
        self.do_file_impl()
    }

    fn op_name(&self) -> &'static str {
        "mod-text-regex"
    }

    fn get_path(&self) -> &Path {
        self.dir
    }

    fn get_tags(&self) -> &Vec<MyTag> {
        self.tags
    }
}

impl WalkAction for ModTextRegexAction<'_> {
    fn do_one_file(&self, path: &Path) -> Result<(), Error> {
        self.do_one_file_write(path)
    }
}

impl WriteAction for ModTextRegexAction<'_> {
    fn is_dry_run(&self) -> bool {
        self.dry_run
    }

    fn set_tags_some(&self, t: &mut TagImpl) -> Result<(), Error> {
        self.set_tags_some_impl(t)
    }
}

impl WriteTextAction for ModTextRegexAction<'_> {
    fn get_new_text(&self, current: &Option<String>) -> Option<String> {
        if let Some(curr) = current {
            let new_v = get_value_re(curr, &self.re, self.to);
            if !new_v.eq(curr) {
                Some(new_v.to_string())
            } else {
                None
            }
        } else {
            None
        }
    }
}

fn get_value_re<'a, 'b>(original: &'b str, re: &'a Regex, to: &'a str) -> Cow<'b, str> {
    re.replace(original, to)
}
