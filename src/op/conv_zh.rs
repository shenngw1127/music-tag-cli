use std::env;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Error};
use opencc_rust::{DefaultConfig, generate_static_dictionary, OpenCC};

use crate::model::{MyTag, TEXT_TAGS};
use crate::model::ConvZhProfile;
use crate::op::{get_file_iterator, get_tags_from_args, get_where, string_to_option, WriteTextForCurrentAction};
use crate::op::{Action, WalkAction, WriteAction, WriteTextAction};
use crate::op::tag_impl::ReadWriteTag;
use crate::where_clause::WhereClause;

pub struct ConvZhAction {
    it: Box<dyn Iterator<Item=PathBuf>>,
    dry_run: bool,
    tags: Vec<MyTag>,
    where_clause: Option<WhereClause>,
    open_cc: OpenCC,
}

impl ConvZhAction {
    pub fn new<P>(dir: P,
                  dry_run: bool,
                  tags: &[MyTag],
                  where_string: &Option<String>,
                  profile: &ConvZhProfile) -> Result<Self, Error>
        where P: AsRef<Path>
    {
        let open_cc = init_open_cc(profile)?;
        let it = get_file_iterator(dir.as_ref())?;
        let tags = get_tags_from_args(tags, &TEXT_TAGS)?;
        let where_clause = get_where(where_string)?;
        Ok(Self {
            it,
            dry_run,
            tags,
            where_clause,
            open_cc,
        })
    }
}

impl Action for ConvZhAction {
    fn do_any(&mut self) -> Result<(), Error> {
        self.do_all()
    }
}

impl WalkAction for ConvZhAction {
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

impl WriteAction for ConvZhAction {
    fn is_dry_run(&self) -> bool {
        self.dry_run
    }

    fn write_tags(&self, t: &mut dyn ReadWriteTag) -> Result<bool, Error> {
        self.write_tags_impl(t)
    }
}

impl WriteTextAction for ConvZhAction {
    fn set_text_tag(&self, t: &mut dyn ReadWriteTag, tag: &MyTag) -> bool {
        self.set_text_tag_impl(t, tag)
    }
}

impl WriteTextForCurrentAction for ConvZhAction {
    fn get_new_text(&self, current: &Option<String>) -> Option<String> {
        if let Some(curr) = current {
            let new_v = self.open_cc.convert(curr);
            string_to_option(new_v, curr)
        } else {
            None
        }
    }
}

impl From<ConvZhProfile> for DefaultConfig {
    fn from(value: ConvZhProfile) -> Self {
        match value {
            ConvZhProfile::HK2S => DefaultConfig::HK2S,
            ConvZhProfile::HK2T => DefaultConfig::HK2T,
            ConvZhProfile::JP2T => DefaultConfig::JP2T,
            ConvZhProfile::S2T => DefaultConfig::S2T,
            ConvZhProfile::S2TW => DefaultConfig::S2TW,
            ConvZhProfile::S2TWP => DefaultConfig::S2TWP,
            ConvZhProfile::T2HK => DefaultConfig::T2HK,
            ConvZhProfile::T2JP => DefaultConfig::T2JP,
            ConvZhProfile::T2TW => DefaultConfig::T2TW,
            ConvZhProfile::T2S => DefaultConfig::T2S,
            ConvZhProfile::S2HK => DefaultConfig::S2HK,
            ConvZhProfile::TW2S => DefaultConfig::TW2S,
            ConvZhProfile::TW2SP => DefaultConfig::TW2SP,
            ConvZhProfile::TW2T => DefaultConfig::TW2T,
        }
    }
}

fn init_open_cc(profile: &ConvZhProfile) -> Result<OpenCC, Error> {
    let config: DefaultConfig = (*profile).into();
    let temporary_path = env::temp_dir();
    generate_static_dictionary(&temporary_path, config)
        .map_err(|e| { anyhow!(e) })?;

    OpenCC::new(temporary_path.join(config))
        .map_err(|e| { anyhow!(e) })
}
