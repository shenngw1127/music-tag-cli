use std::borrow::Cow;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Error};
use log::info;

use crate::config::get_tag_lab;
use crate::model::MyTag;

pub use self::audio_tags_impl::{AudioTagWrapper, available_suffix as audio_tags_available_suffix};
pub use self::taglib_impl::{available_suffix as taglib_available_suffix, TaglibWrapper};

mod audio_tags_impl;
mod taglib_impl;

pub struct TagImpl<'c> {
    raw: TagImplRaw<'c>,
    file_name: Cow<'c, str>,
    dry_run: bool,
}

enum TagImplRaw<'c> {
    Taglib(TaglibWrapper<'c>),
    AudioTag(AudioTagWrapper<'c>),
}

const AUDIO_TAGS: &'static str = "audiotags";

impl<'c> TagImpl<'c> {
    pub fn new(path: &'c Path, dry_run: bool) -> Result<TagImpl, Error> {
        let file_name = path.to_string_lossy();
        match get_tag_lab() {
            Some(s) => {
                if s.eq(AUDIO_TAGS) {
                    AudioTagWrapper::new(path)
                        .map(|t|
                            TagImpl { raw: TagImplRaw::AudioTag(t), file_name, dry_run })
                } else {
                    TaglibWrapper::new(path)
                        .map(|t|
                            TagImpl { raw: TagImplRaw::Taglib(t), file_name, dry_run })
                }
            }
            None => {
                TaglibWrapper::new(path)
                    .map(|t|
                        TagImpl { raw: TagImplRaw::Taglib(t), file_name, dry_run })
            }
        }
    }
}

pub fn is_available_suffix(file_name: &str) -> bool {
    match get_tag_lab() {
        Some(s) => {
            if s.eq(AUDIO_TAGS) {
                audio_tags_available_suffix(file_name)
            } else {
                taglib_available_suffix(file_name)
            }
        }
        None => { taglib_available_suffix(file_name) }
    }
}

impl ReadTag for TagImpl<'_> {
    fn get_text_tag(&self, key: &MyTag) -> Option<String> {
        match &self.raw {
            TagImplRaw::Taglib(inner) => inner.get_text_tag(key),
            TagImplRaw::AudioTag(inner) => inner.get_text_tag(key),
        }
    }

    fn get_numeric_tag(&self, key: &MyTag) -> Option<u32> {
        match &self.raw {
            TagImplRaw::Taglib(inner) => inner.get_numeric_tag(key),
            TagImplRaw::AudioTag(inner) => inner.get_numeric_tag(key),
        }
    }

    fn get_numeric_tag_string(&self, key: &MyTag) -> Option<String> {
        match &self.raw {
            TagImplRaw::Taglib(inner) => inner.get_numeric_tag_string(key),
            TagImplRaw::AudioTag(inner) => inner.get_numeric_tag_string(key),
        }
    }

    fn get_property_keys(&self) -> Result<Vec<String>, Error> {
        match &self.raw {
            TagImplRaw::Taglib(inner) => inner.get_property_keys(),
            TagImplRaw::AudioTag(inner) => inner.get_property_keys(),
        }
    }

    fn get_property(&self, key: &str) -> Result<Vec<String>, Error> {
        match &self.raw {
            TagImplRaw::Taglib(inner) => inner.get_property(key),
            TagImplRaw::AudioTag(inner) => inner.get_property(key),
        }
    }
}

impl WriteTagFile for TagImpl<'_> {
    fn save(&mut self) -> Result<(), Error> {
        if !self.dry_run {
            match &mut self.raw {
                TagImplRaw::Taglib(inner) => inner.save(),
                TagImplRaw::AudioTag(inner) => inner.save(),
            }
        } else {
            let path = PathBuf::from(self.file_name.to_string());
            if path.exists()
                && path.is_file()
                && fs::metadata(&path).map_or(
                false, |m| !m.permissions().readonly()) {
                info!("Save file {} ok.", self.file_name);
                Ok(())
            } else {
                Err(anyhow!("Save file {} FAILED! \
                Please check if file exists and it's attribute is NOT \"Read-only\".",
                    self.file_name))
            }
        }
    }
}

impl WriteTag for TagImpl<'_> {
    fn write_text_tag(&mut self, key: &MyTag, value: &str) {
        if !self.dry_run {
            match &mut self.raw {
                TagImplRaw::Taglib(inner) => inner.write_text_tag(key, value),
                TagImplRaw::AudioTag(inner) => inner.write_text_tag(key, value),
            }
        } else {
            info!("file {} set {}: {}", self.file_name, key, value);
        }
    }

    fn write_numeric_tag(&mut self, key: &MyTag, value: u32, padding: usize) {
        if !self.dry_run {
            match &mut self.raw {
                TagImplRaw::Taglib(t) => t.write_numeric_tag(key, value, padding),
                TagImplRaw::AudioTag(t) => t.write_numeric_tag(key, value, padding),
            }
        } else {
            match &self.raw {
                TagImplRaw::Taglib(_) =>
                    info!("file {} set {}: {} with padding {}", self.file_name, key, value, padding),
                TagImplRaw::AudioTag(_) =>
                    info!("file {} set {}: {}", self.file_name, key, value),
            }
        }
    }
}

impl ReadWriteTag for TagImpl<'_> {}

pub trait ReadTag {
    fn get_text_tag(&self, key: &MyTag) -> Option<String>;
    fn get_numeric_tag(&self, key: &MyTag) -> Option<u32>;

    fn get_numeric_tag_string(&self, key: &MyTag) -> Option<String>;

    fn get_property_keys(&self) -> Result<Vec<String>, Error>;

    fn get_property(&self, key: &str) -> Result<Vec<String>, Error>;
}

pub trait WriteTagFile {
    fn save(&mut self) -> Result<(), Error>;
}

pub trait WriteTag: WriteTagFile {
    fn write_text_tag(&mut self, key: &MyTag, value: &str);

    fn write_numeric_tag(&mut self, key: &MyTag, value: u32, padding: usize);
}

pub trait ReadWriteTag: ReadTag + WriteTag {}