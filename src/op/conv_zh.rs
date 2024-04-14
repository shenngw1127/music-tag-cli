use anyhow::{anyhow, Error};
use opencc_rust::{DefaultConfig, generate_static_dictionary, OpenCC};
use std::path::Path;
use std::env;

use crate::model::ConvZhProfile;
use crate::model::{MyTag, TEXT_TAGS};
use crate::op::tag_impl::TagImpl;
use crate::op::{Action, WalkAction, WriteAction, WriteTextAction};

pub struct ConvZhAction<'a> {
    dir: &'a Path,
    dry_run: bool,
    tags: &'a Vec<MyTag>,
    open_cc: OpenCC,
}

impl<'a> ConvZhAction<'a> {
    pub fn new(dir: &'a Path,
               dry_run: bool,
               tags: &'a Vec<MyTag>,
               profile: &'a ConvZhProfile) -> Result<ConvZhAction<'a>, Error> {
        let open_cc = Self::init_open_cc(profile)?;
        Ok(ConvZhAction {
            dir,
            dry_run,
            tags: if !tags.is_empty() {
                tags
            } else {
                &TEXT_TAGS
            },
            open_cc,
        })
    }

    fn init_open_cc(profile: &ConvZhProfile) -> Result<OpenCC, Error> {
        let config: DefaultConfig = profile.into();
        let temporary_path = env::temp_dir();
        generate_static_dictionary(&temporary_path, config)
            .map_err(|e| { anyhow!(e) })?;

        OpenCC::new(temporary_path.join(config))
            .map_err(|e| { anyhow!(e) })
    }
}

impl Action for ConvZhAction<'_> {
    fn do_dir(&self) -> Result<(), Error> {
        self.do_dir_walk()
    }

    fn do_file(&self) -> Result<(), Error> {
        self.do_file_impl()
    }

    fn op_name(&self) -> &'static str {
        "conv-zh"
    }

    fn get_path(&self) -> &Path {
        self.dir
    }

    fn get_tags(&self) -> &Vec<MyTag> {
        self.tags
    }
}

impl WalkAction for ConvZhAction<'_> {
    fn do_one_file(&self, path: &Path) -> Result<(), Error> {
        self.do_one_file_write(path)
    }
}

impl WriteAction for ConvZhAction<'_> {
    fn is_dry_run(&self) -> bool {
        self.dry_run
    }

    fn set_tags_some(&self, t: &mut TagImpl) -> Result<(), Error> {
        self.set_tags_some_impl(t)
    }
}

impl WriteTextAction for ConvZhAction<'_> {
    fn get_new_text(&self, current: &Option<String>) -> Option<String> {
        if let Some(curr) = current {
            let new_v = self.open_cc.convert(curr);
            if !new_v.eq(curr) {
                Some(new_v)
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl From<&ConvZhProfile> for DefaultConfig {
    fn from(value: &ConvZhProfile) -> Self {
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
