use std::path::{Path, PathBuf};

use anyhow::{anyhow, Error};
use log::debug;
use regex::{Regex, RegexBuilder};

use crate::model::{MyTag, TEXT_TAGS};
use crate::op::{Action, get_file_iterator, get_tags_from_args, get_where, WalkAction, WriteAction, WriteTextAction};
use crate::op::tag_impl::ReadWriteTag;
use crate::where_clause::WhereClause;

pub struct ModTextRegexAction {
    it: Box<dyn Iterator<Item=PathBuf>>,
    dry_run: bool,
    tags: Vec<MyTag>,
    where_clause: Option<WhereClause>,
    re: Regex,
    to: String,
}

impl ModTextRegexAction {
    pub fn new<P>(dir: P,
                  dry_run: bool,
                  tags: &[MyTag],
                  where_string: &Option<String>,
                  from: &str,
                  ignore_case: bool,
                  to: &str) -> Result<Self, Error>
        where P: AsRef<Path>
    {
        let re = get_regex(from, ignore_case)?;
        let it = get_file_iterator(dir.as_ref())?;
        let tags = get_tags_from_args(tags, &TEXT_TAGS)?;
        let where_clause = get_where(where_string)?;
        Ok(Self {
            it,
            dry_run,
            tags,
            where_clause,
            re,
            to: to.to_owned(),
        })
    }
}

impl Action for ModTextRegexAction {
    fn do_any(&mut self) -> Result<(), Error> {
        self.do_all()
    }
}

impl WalkAction for ModTextRegexAction {
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

impl WriteAction for ModTextRegexAction {
    fn is_dry_run(&self) -> bool {
        self.dry_run
    }

    fn set_tags_some(&self, t: &mut dyn ReadWriteTag) -> Result<bool, Error> {
        self.set_tags_some_impl(t)
    }
}

impl WriteTextAction for ModTextRegexAction {
    fn get_new_text(&self, current: &Option<String>) -> Option<String> {
        if let Some(curr) = current {
            let new_v = (&self.re).replace_all(curr, &self.to);
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

fn get_regex(from: &str, ignore_case: bool) -> Result<Regex, Error> {
    debug!("from: {}", from);
    RegexBuilder::new(from)
        .case_insensitive(ignore_case)
        .unicode(true)
        .build()
        .map_err(|e| { anyhow!(e) })
}

#[cfg(test)]
mod test {
    use super::get_regex;

    #[test]
    fn test_get_value_re() {
        let re = &get_regex(r"[A-Za-z]", false).unwrap();
        assert_eq!(re.replace_all("Hello World!", "x"), "xxxxx xxxxx!");

        let re = &get_regex("Hello", true).unwrap();
        assert_eq!(re.replace_all("Hello World hello!", "x"), "x World x!");

        let re = &get_regex(r"久石譲", false).unwrap();
        assert_eq!(re.replace_all("久石譲", "久石 譲"), "久石 譲");
        assert_eq!(
            re.replace_all("久石譲; 久石譲; New Japan Philharmonic Orchestra",
                           "久石 譲"),
            "久石 譲; 久石 譲; New Japan Philharmonic Orchestra");

        let re = &get_regex("\\s*([^;]*);", false).unwrap();
        assert_eq!(
            re.replace_all("久石譲; 久石譲; New Japan Philharmonic Orchestra",
                           ";${1}"),
            ";久石譲;久石譲 New Japan Philharmonic Orchestra");
    }
}