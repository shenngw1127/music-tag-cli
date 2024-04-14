use anyhow::{anyhow, Error};
use encoding::all::ISO_8859_1;
use encoding::{EncoderTrap, Encoding};
use encoding_rs::Encoding as EncodingRs;
use log::error;
use std::path::Path;

use crate::model::{MyTag, TEXT_TAGS};
use crate::op::tag_impl::TagImpl;
use crate::op::{Action, WalkAction, WriteAction, WriteTextAction};

pub struct ConvUtf8Action<'a> {
    dir: &'a Path,
    dry_run: bool,
    tags: &'a Vec<MyTag>,
    encoding: &'static EncodingRs,
}

impl<'a> ConvUtf8Action<'a> {
    pub fn new(dir: &'a Path,
               dry_run: bool,
               tags: &'a Vec<MyTag>,
               encoding_name: &'a str) -> Result<ConvUtf8Action<'a>, Error> {
        let encoding = Self::get_encoding(encoding_name)?;
        Ok(ConvUtf8Action {
            dir,
            dry_run,
            tags: if !tags.is_empty() {
                tags
            } else {
                &TEXT_TAGS
            },
            encoding,
        })
    }

    fn get_encoding(enc_name: &str) -> Result<&'static EncodingRs, Error> {
        let lowercase_enc_name = &enc_name.to_lowercase();
        if !lowercase_enc_name.eq("utf8") && !lowercase_enc_name.eq("utf-8") {
            let encoding = EncodingRs::for_label(enc_name.as_bytes());
            encoding.ok_or(anyhow!("Unsupported encoding: {}", enc_name))
        } else {
            Err(anyhow!("Encoding could NOT be UTF-8."))
        }
    }

    fn convert(&self, current: &str) -> Option<String> {
        ISO_8859_1.encode(&current, EncoderTrap::Strict)
            .map_or_else(
                |e| {
                    error!("encode error. original value: {}. (e: {})", &current, e);
                    None
                },
                |bytes| {
                    let (new_v, .., had_errors) = self.encoding.decode(&bytes);
                    if !had_errors {
                        if !new_v.eq(current) {
                            Some(new_v.to_string())
                        } else {
                            None
                        }
                    } else {
                        error!("decode error. original value: {}.", &current);
                        None
                    }
                })
    }
}

impl Action for ConvUtf8Action<'_> {
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

impl WalkAction for ConvUtf8Action<'_> {
    fn do_one_file(&self, path: &Path) -> Result<(), Error> {
        self.do_one_file_write(path)
    }
}

impl WriteAction for ConvUtf8Action<'_> {
    fn is_dry_run(&self) -> bool {
        self.dry_run
    }

    fn set_tags_some(&self, t: &mut TagImpl) -> Result<(), Error> {
        self.set_tags_some_impl(t)
    }
}

impl WriteTextAction for ConvUtf8Action<'_> {
    fn get_new_text(&self, current: &Option<String>) -> Option<String> {
        if let Some(curr) = current {
            self.convert(curr)
        } else {
            None
        }
    }
}