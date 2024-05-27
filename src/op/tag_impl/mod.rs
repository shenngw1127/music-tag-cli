use std::fs;
use std::path::Path;

use anyhow::{anyhow, Error};
use as_dyn_trait::as_dyn_trait;
use log::info;

use crate::config::get_tag_lab;
use crate::model::MyTag;

pub use self::audio_tags_impl::{AudioTagWrapper, available_suffix as audio_tags_available_suffix};
pub use self::taglib_impl::{available_suffix as taglib_available_suffix, TaglibWrapper};

mod audio_tags_impl;
mod taglib_impl;

pub struct TagImpl<'a> {
    raw: TagImplRaw<'a>,
    dry_run: bool,
}

enum TagImplRaw<'a> {
    Taglib(TaglibWrapper<'a>),
    AudioTag(AudioTagWrapper<'a>),
}

const AUDIO_TAGS: &'static str = "audiotags";

impl<'a> TagImpl<'a> {
    pub fn new(path: &'a dyn AsRef<Path>,
               dry_run: bool,
    ) -> Result<Self, Error> {
        match get_tag_lab() {
            Some(ref s) => {
                if s.eq(AUDIO_TAGS) {
                    AudioTagWrapper::new(path)
                        .map(|t|
                            TagImpl {
                                raw: TagImplRaw::AudioTag(t),
                                dry_run,
                            })
                } else {
                    TaglibWrapper::new(path)
                        .map(|t|
                            TagImpl {
                                raw: TagImplRaw::Taglib(t),
                                dry_run,
                            })
                }
            }
            None => {
                TaglibWrapper::new(path)
                    .map(|t|
                        TagImpl {
                            raw: TagImplRaw::Taglib(t),
                            dry_run,
                        })
            }
        }
    }
}

pub fn is_available_suffix(file_name: &str) -> bool {
    match get_tag_lab() {
        Some(ref s) => {
            if s.eq(AUDIO_TAGS) {
                audio_tags_available_suffix(file_name)
            } else {
                taglib_available_suffix(file_name)
            }
        }
        None => { taglib_available_suffix(file_name) }
    }
}

impl<'a> ReadTag for TagImpl<'a> {
    fn get_path(&self) -> &Path {
        match &self.raw {
            TagImplRaw::Taglib(inner) => inner.get_path(),
            TagImplRaw::AudioTag(inner) => inner.get_path(),
        }
    }

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
            let path = self.get_path();
            if path.exists()
                && path.is_file()
                && fs::metadata(&path).map_or(
                false, |m| !m.permissions().readonly()) {
                info!("Save file {:?} ok.", path);
                Ok(())
            } else {
                Err(anyhow!("Save file {:?} FAILED! \
                Please check if file exists and it's attribute is NOT \"Read-only\".",
                    path))
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
            info!("file {:?} set {}: {}", self.get_path(), key, value);
        }
    }

    fn write_numeric_tag(&mut self, key: &MyTag, value: u32, padding: usize) {
        if !self.dry_run {
            match &mut self.raw {
                TagImplRaw::Taglib(t) => t.write_numeric_tag(key, value, padding),
                TagImplRaw::AudioTag(t) => t.write_numeric_tag(key, value, padding),
            }
        } else {
            let path = self.get_path();
            match &self.raw {
                TagImplRaw::Taglib(_) =>
                    info!("file {:?} set {}: {} with padding {}", path, key, value, padding),
                TagImplRaw::AudioTag(_) =>
                    info!("file {:?} set {}: {}", path, key, value),
            }
        }
    }
}

impl ReadWriteTag for TagImpl<'_> {}

#[as_dyn_trait]
pub trait ReadTag {
    fn get_path(&self) -> &Path;

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