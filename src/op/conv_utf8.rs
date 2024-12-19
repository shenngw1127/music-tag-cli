use std::path::{Path, PathBuf};

use anyhow::Error;
use encoding::{EncoderTrap, Encoding};
use encoding::all::ISO_8859_1;
use encoding_rs::Encoding as EncodingRs;
use log::error;

use crate::model::{MyTag, TEXT_TAGS};
use crate::op::{check_encoding_not_utf8, get_encoding, get_file_iterator, get_tags_from_args,
                get_where};
use crate::op::{Action, WalkAction, WriteAction, WriteTextAction, WriteTextForCurrentAction};
use crate::op::tag_impl::ReadWriteTag;
use crate::where_clause::WhereClause;

pub struct ConvUtf8Action {
    it: Box<dyn Iterator<Item=PathBuf>>,
    dry_run: bool,
    tags: Vec<MyTag>,
    where_clause: Option<WhereClause>,
    encoding: &'static EncodingRs,
}

impl ConvUtf8Action {
    pub fn new<P>(dir: P,
                  dry_run: bool,
                  tags: &[MyTag],
                  where_string: &Option<String>,
                  encoding_name: &str) -> Result<Self, Error>
        where P: AsRef<Path>
    {
        let encoding = get_encoding(encoding_name)?;
        let it = get_file_iterator(dir.as_ref())?;
        let tags = get_tags_from_args(tags, &TEXT_TAGS)?;
        let where_clause = get_where(where_string)?;
        Self::check(encoding).map(|_|
            Self {
                it,
                dry_run,
                tags,
                where_clause,
                encoding,
            })
    }

    fn check(value: &'static EncodingRs) -> Result<(), Error> {
        check_encoding_not_utf8(value)?;
        Ok(())
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

impl Action for ConvUtf8Action {
    fn do_any(&mut self) -> Result<(), Error> {
        self.do_all()
    }
}

impl WalkAction for ConvUtf8Action {
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

impl WriteAction for ConvUtf8Action {
    fn is_dry_run(&self) -> bool {
        self.dry_run
    }

    fn write_tags(&self, t: &mut dyn ReadWriteTag) -> Result<bool, Error> {
        self.write_tags_impl(t)
    }
}

impl WriteTextAction for ConvUtf8Action {
    fn set_text_tag(&self, t: &mut dyn ReadWriteTag, tag: &MyTag) -> bool {
        self.set_text_tag_impl(t, tag)
    }
}

impl WriteTextForCurrentAction for ConvUtf8Action {
    fn get_new_text(&self, current: &Option<String>) -> Option<String> {
        if let Some(curr) = current {
            self.convert(curr)
        } else {
            None
        }
    }
}
