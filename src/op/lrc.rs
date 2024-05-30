use std::fs::File;
use std::io::{BufWriter, Read, Write};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Error};
use encoding_rs::Encoding as EncodingRs;
use log::{debug, error, info};

use crate::model::MyTag;
use crate::op::{get_encoding, get_file_iterator, get_where, is_utf8, MyValues, ReadAction};
use crate::op::{Action, WalkAction, WriteAction, WriteTextAction};
use crate::op::tag_impl::ReadWriteTag;
use crate::where_clause::WhereClause;

const MAX_FILE_LENGTH: u64 = 1024 * 1024;
const BUFFER_SIZE: usize = 4 * 1024;

pub struct LrcImpAction {
    it: Box<dyn Iterator<Item=PathBuf>>,
    encoding: &'static EncodingRs,
    dry_run: bool,
    tags: Vec<MyTag>,
    where_clause: Option<WhereClause>,
}

impl LrcImpAction {
    pub fn new<P>(dir: P,
                  encoding_name: &str,
                  dry_run: bool,
                  where_string: &Option<String>) -> Result<Self, Error>
        where P: AsRef<Path>
    {
        let encoding = get_encoding(encoding_name)?;
        let it = get_file_iterator(dir.as_ref())?;
        let tags = vec![MyTag::Lyrics];
        let where_clause = get_where(where_string)?;
        Ok(Self {
            it,
            encoding,
            dry_run,
            tags,
            where_clause,

        })
    }

    fn get_new_text(&self, t: &dyn ReadWriteTag) -> Option<String> {
        let mut path = PathBuf::from(t.get_path());
        path.set_extension("lrc");

        if !path.exists() || !path.is_file() {
            error!("path {:?} not exists or not a file!", &path);
            return None;
        }

        if let Some(m) = path.metadata().ok() {
            if m.len() > MAX_FILE_LENGTH {
                error!("file {:?} is too big!", &path);
                return None;
            }

            match File::open(&path) {
                Ok(mut input_file) => {
                    if is_utf8(self.encoding) {
                        read_file(&mut input_file, &path)
                    } else {
                        read_file_decode(&mut input_file, &path, self.encoding)
                    }
                }
                Err(e) => {
                    error!("read file {:?} error {:?}", & path, e);
                    None
                }
            }
        } else {
            error!("could NOT found path {:?} metadata!", & path);
            None
        }
    }
}

fn read_file<P>(input_file: &mut File, path: P) -> Option<String>
    where P: AsRef<Path>
{
    let buf: &mut Vec<u8> = &mut Vec::with_capacity(BUFFER_SIZE);
    match input_file.read_to_end(buf) {
        Ok(0) => Some("".to_owned()),
        Ok(1) => Some(String::from_utf8_lossy(&buf[..1]).to_string()),
        Ok(2) => Some(String::from_utf8_lossy(&buf[..2]).to_string()),
        Ok(count) => {
            let res =
                if start_with_bom(buf) {
                    String::from_utf8(Vec::from(&buf[3..count]))
                } else {
                    String::from_utf8(Vec::from(&buf[..count]))
                };

            res.map_or_else(
                |e| {
                    error!("read file {:?} error {:?}", path.as_ref(), e);
                    None
                },
                |t| Some(t),
            )
        }

        Err(e) => {
            error!("read file {:?} error {:?}", path.as_ref(), e);
            None
        }
    }
}

#[inline]
fn start_with_bom(buf: &[u8]) -> bool {
    buf[0] == 0xEF && buf[1] == 0xBB && buf[2] == 0xBF
}

fn read_file_decode<P>(input_file: &mut File,
                       path: P,
                       encoding: &'static EncodingRs) -> Option<String>
    where P: AsRef<Path>
{
    let buf: &mut Vec<u8> = &mut Vec::with_capacity(BUFFER_SIZE);
    match input_file.read_to_end(buf) {
        Ok(count) => {
            if count != 0 {
                let (value, .., had_errors) =
                    encoding.decode(&buf[..count]);
                if !had_errors {
                    Some(value.to_string())
                } else {
                    error!("decode file {:?} with error, bytes: {:?}",
                                        path.as_ref(), &buf[..count]);
                    None
                }
            } else {
                None
            }
        }
        Err(e) => {
            error!("read file {:?} error {:?}", path.as_ref(), e);
            None
        }
    }
}

impl Action for LrcImpAction {
    fn do_any(&mut self) -> Result<(), Error> {
        self.do_all()
    }
}

impl WalkAction for LrcImpAction {
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

impl WriteAction for LrcImpAction {
    fn is_dry_run(&self) -> bool {
        self.dry_run
    }

    fn write_tags(&self, t: &mut dyn ReadWriteTag) -> Result<bool, Error> {
        self.write_tags_impl(t)
    }
}

impl WriteTextAction for LrcImpAction {
    fn set_text_tag(&self, t: &mut dyn ReadWriteTag, tag: &MyTag) -> bool {
        let new_value = self.get_new_text(t);
        debug!("new_value: {:?}", &new_value);

        if let Some(new_v) = &new_value {
            t.write_text_tag(tag, new_v);
            true
        } else {
            false
        }
    }
}

pub struct LrcExpAction {
    it: Box<dyn Iterator<Item=PathBuf>>,
    encoding: &'static EncodingRs,
    tags: Vec<MyTag>,
    dry_run: bool,
    where_clause: Option<WhereClause>,
}

impl LrcExpAction {
    pub fn new<P>(dir: P,
                  encoding_name: &str,
                  dry_run: bool,
                  where_string: &Option<String>) -> Result<Self, Error>
        where P: AsRef<Path>
    {
        let encoding = get_encoding(encoding_name)?;
        let it = get_file_iterator(dir.as_ref())?;
        let tags = vec![MyTag::Lyrics];
        let where_clause = get_where(where_string)?;
        Ok(Self {
            it,
            encoding,
            tags,
            dry_run,
            where_clause,
        })
    }
}

impl Action for LrcExpAction {
    fn do_any(&mut self) -> Result<(), Error> {
        self.do_all()
    }
}

impl WalkAction for LrcExpAction {
    fn get_iterator(&mut self) -> &mut dyn Iterator<Item=PathBuf> {
        &mut self.it
    }

    fn do_one_file(&mut self, path: &Path) -> Result<bool, Error> {
        self.do_one_file_read(path)
    }

    fn get_where(&self) -> &Option<WhereClause> {
        &self.where_clause
    }

    fn tags(&self) -> &Vec<MyTag> {
        &self.tags
    }
}

impl ReadAction for LrcExpAction {
    fn with_properties(&self) -> bool {
        false
    }

    fn get_content(&self, _path: &Path, v: &MyValues) -> Result<Option<String>, Error> {
        Ok(v.get_text(&MyTag::Lyrics).map(|t| t.to_owned()))
    }

    fn do_output(&mut self, path: &Path, content: &str) -> Result<bool, Error> {
        let mut new_path = PathBuf::from(path);
        new_path.set_extension("lrc");
        if new_path.exists() {
            error!("file: {:?} exists.", path);
            return Ok(false);
        }

        if is_utf8(self.encoding) {
            write_file(&new_path, content.as_bytes(), self.dry_run)?;
            Ok(true)
        } else {
            let (new_v, .., had_errors) = self.encoding.encode(content);
            if had_errors {
                error!("encode error. original value: {}.", &content);
                Ok(false)
            } else {
                write_file(&new_path, new_v.as_ref(), self.dry_run)?;
                Ok(true)
            }
        }
    }
}

fn write_file<P>(path: P,
                 bytes: &[u8],
                 dry_run: bool) -> Result<(), Error>
    where P: AsRef<Path>
{
    if !dry_run {
        let path = path.as_ref();
        let f = File::create(path)?;
        let mut writer = BufWriter::with_capacity(BUFFER_SIZE, f);

        writer.write(bytes)?;
        info!("Save lyric file {:?} ok.", path);
        Ok(())
    } else {
        write_file_dry_run(path)
    }
}

fn write_file_dry_run<P>(path: P) -> Result<(), Error>
    where P: AsRef<Path>
{
    let path = path.as_ref();
    if !path.exists() {
        info!("Save lyric file {:?} ok.", path);
        Ok(())
    } else {
        Err(anyhow!("Save lyric file {:?} failed! File already exists.", path))
    }
}
