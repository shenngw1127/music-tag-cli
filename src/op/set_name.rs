use std::borrow::Cow;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Error};
use fancy_regex::{Regex, RegexBuilder};
use itertools::Itertools;
use lazy_static::lazy_static;
use log::debug;

use crate::model::{DEFAULT_PADDING, MyTag, SetWhen};
use crate::op::{get_file_iterator, get_where};
use crate::op::{Action, WalkAction, WriteAction, WriteAllAction};
use crate::op::tag_impl::ReadWriteTag;
use crate::where_clause::WhereClause;

pub struct SetNameAction {
    it: Box<dyn Iterator<Item=PathBuf>>,
    tags: Vec<MyTag>,
    dry_run: bool,
    set_when: SetWhen,
    where_clause: Option<WhereClause>,
    regex: Regex,
}

impl SetNameAction {
    pub fn new<P>(dir: P,
                  dry_run: bool,
                  set_when: &SetWhen,
                  where_string: &Option<String>,
                  template: &str) -> Result<Self, Error>
        where P: AsRef<Path>
    {
        let regex = get_regex(template)?;
        let it = get_file_iterator(dir.as_ref())?;
        let tags = get_tags_from_template(template)?;
        let where_clause = get_where(where_string)?;
        Ok(Self {
            it,
            tags,
            dry_run,
            set_when: set_when.clone(),
            where_clause,
            regex,
        })
    }

    fn set_text_tag_real(&self, t: &mut dyn ReadWriteTag, tag: &MyTag) -> bool {
        if let Some(new_value) = self.get_new_text(t.get_path(), tag) {
            t.write_text_tag(tag, &new_value);
            true
        } else {
            false
        }
    }

    fn set_numeric_tag_real(&self, t: &mut dyn ReadWriteTag, tag: &MyTag) -> bool {
        if let Some(new_text) = self.get_new_text(t.get_path(), tag) {
            new_text.parse::<u32>()
                .map_or_else(|_| false,
                             |value| {
                                 t.write_numeric_tag(tag, value, DEFAULT_PADDING);
                                 true
                             })
        } else {
            false
        }
    }

    fn get_new_text<P>(&self,
                       filename: P,
                       tag: &MyTag) -> Option<String>
        where P: AsRef<Path>
    {
        let key = tag.to_string().replace("-", "");
        debug!("key: {}", key);

        let path = filename.as_ref();
        let file_stem = path.file_stem().map_or_else(
            || Cow::Borrowed(""),
            |t| t.to_string_lossy(),
        );

        let mut all_captures = self.regex.captures_iter(&file_stem);
        if let Some(first) = all_captures.next() {
            first.map_or_else(
                |_| None,
                |c| c.name(&key).map_or_else(
                    || None,
                    |t| Some(t.as_str().to_owned()),
                ),
            )
        } else {
            None
        }
    }
}

impl Action for SetNameAction {
    fn do_any(&mut self) -> Result<(), Error> {
        self.do_all()
    }
}

impl WalkAction for SetNameAction {
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

impl WriteAction for SetNameAction {
    fn is_dry_run(&self) -> bool {
        self.dry_run
    }

    fn write_tags(&self, t: &mut dyn ReadWriteTag) -> Result<bool, Error> {
        self.write_tags_impl(t)
    }
}

impl WriteAllAction for SetNameAction {
    fn set_text_tag(&self, t: &mut dyn ReadWriteTag, tag: &MyTag) -> bool {
        match &self.set_when {
            SetWhen::Always => {
                self.set_text_tag_real(t, tag)
            }
            set_when => {
                let current = t.get_text_tag(tag);
                if should_write_text(&current, set_when) {
                    self.set_text_tag_real(t, tag)
                } else {
                    false
                }
            }
        }
    }

    fn set_numeric_tag(&self, t: &mut dyn ReadWriteTag, tag: &MyTag) -> bool {
        match &self.set_when {
            SetWhen::Always => {
                self.set_numeric_tag_real(t, tag)
            }
            set_when => {
                let current = t.get_numeric_tag(tag);
                if should_write_numeric(&current, set_when) {
                    self.set_numeric_tag_real(t, tag)
                } else {
                    false
                }
            }
        }
    }

    fn set_date_tag(&self, t: &mut dyn ReadWriteTag, tag: &MyTag) -> bool {
        self.set_text_tag_real(t, tag)
    }
}

impl SetNameAction {}

fn should_write_text(current: &Option<String>, set_when: &SetWhen) -> bool {
    if let Some(curr) = current {
        if !curr.is_empty() {
            set_when != &SetWhen::OnlyEmpty
        } else {
            set_when != &SetWhen::OnlyNotEmpty
        }
    } else {
        set_when != &SetWhen::OnlyNotEmpty
    }
}

fn should_write_numeric(current: &Option<u32>, set_when: &SetWhen) -> bool {
    if current.is_some() {
        set_when != &SetWhen::OnlyEmpty
    } else {
        set_when != &SetWhen::OnlyNotEmpty
    }
}

fn get_regex(input: &str) -> Result<Regex, Error> {
    let regex_str = &get_regex_string(input)?;
    debug!("regex_str: {}", regex_str);
    if !regex_str.is_empty() {
        RegexBuilder::new(regex_str).build()
            .map_err(|e| anyhow!("Build regex failed. input: {}, (error: {:?})", input, e))
    } else {
        Err(anyhow!("Build regex failed. input: {} not contains any tag!", input))
    }
}

#[derive(Debug, PartialEq)]
enum Token {
    Chars(String),
    Tags(String),
}

impl Token {
    fn to_tag(&self) -> Option<&'static MyTag> {
        match self {
            Token::Tags(value) => MyTag::from_str(value).ok(),
            _ => None,
        }
    }
}

pub fn get_tags_from_template(input: &str) -> Result<Vec<MyTag>, Error> {
    let res = get_tokens(input).map_or_else(
        |_| vec![],
        |t| t.iter()
            .map(|t| t.to_tag())
            .filter(|t| t.is_some())
            .map(|t| *t.unwrap())
            .unique()
            .collect());

    if !res.is_empty() {
        Ok(res)
    } else {
        Err(anyhow!("You must specified one tag at least in \"--template\" parameter!"))
    }
}

fn get_regex_string(input: &str) -> Result<String, Error> {
    let tokens = get_tokens(&input)?;
    debug!("tokens: {:?}", tokens);
    Ok(to_string(&tokens))
}

lazy_static! {
    static ref REGEX_TEMPLATE: Regex = RegexBuilder::new(r"(\$\{[\w-]+\})").build().unwrap();
}

fn get_tokens(input: &str) -> Result<Vec<Token>, Error> {
    let mut tokens: Vec<Token> = vec![];
    let mut start: usize = 0;
    for o_match in REGEX_TEMPLATE.captures_iter(input)
        .map(|c| c.unwrap().get(1)) {
        if let Some(m) = o_match {
            let cap_start = m.start();
            debug!("start: {}, cap_start: {}", start, cap_start);
            if start < cap_start {
                tokens.push(Token::Chars((&input[start..cap_start]).to_owned()));
            }
            start = m.end();
            let s = m.as_str();
            let value = &s[2..s.len() - 1];
            if MyTag::from_str(value).is_err() {
                return Err(anyhow!("{} is NOT a tag.", value));
            } else {
                tokens.push(Token::Tags(value.to_owned()));
            }
        }
    }

    Ok(tokens)
}

fn to_string(input: &Vec<Token>) -> String {
    let mut result = String::new();
    for e in input {
        match e {
            Token::Tags(s) => {
                result.push_str(&format!("(?<{}>.*)", &s.replace("-", "")));
            }
            Token::Chars(s) => {
                let escaped = escape(s);
                result.push_str(&format!("(?={}){}", &escaped, &escaped));
            }
        }
    }
    result
}

fn escape(input: &str) -> String {
    let mut result = String::new();
    for ch in input.chars() {
        if ch == '.' || ch == '?'
            || ch == '+' || ch == '*'
            || ch == '^' || ch == '$'
            || ch == '(' || ch == ')'
            || ch == '{' || ch == '}'
            || ch == '[' || ch == ']'
            || ch == '|' || ch == '\\' {
            result.push('\\');
        }
        result.push(ch);
    }
    result
}

#[cfg(test)]
mod test {
    use fancy_regex::RegexBuilder;

    use super::{get_regex_string, get_tokens};
    use super::Token::{Chars, Tags};

    #[test]
    fn test_get_tokens() {
        assert_eq!(get_tokens("${track-number} - ${title}").unwrap(),
                   vec![Tags("track-number".to_owned()),
                        Chars(" - ".to_owned()),
                        Tags("title".to_owned())]);
    }

    #[test]
    fn test_get_regex_string() {
        assert_eq!(get_regex_string("${title}").unwrap(), "(?<title>.*)".to_owned());
        assert_eq!(get_regex_string("${track-number} - ${title}").unwrap(),
                   "(?<tracknumber>.*)(?= - ) - (?<title>.*)".to_owned());

        assert!(get_regex_string("abc").unwrap().is_empty());
        assert!(get_regex_string("${unknown-tag}").is_err());
    }

    #[test]
    fn test_extract() {
        let re = RegexBuilder::new("(?<tracknumber>.*)(?= - ) - (?<title>.*)")
            .build()
            .unwrap();

        for cs in re.captures_iter("123 - test is test") {
            let cs = cs.unwrap();
            assert_eq!(cs.name("tracknumber").unwrap().as_str(), "123");
            assert_eq!(cs.name("title").unwrap().as_str(), "test is test");
        }

        for cs in re.captures_iter("123 - 东方之珠") {
            let cs = cs.unwrap();
            assert_eq!(cs.name("tracknumber").unwrap().as_str(), "123");
            assert_eq!(cs.name("title").unwrap().as_str(), "东方之珠");
        }
    }
}
