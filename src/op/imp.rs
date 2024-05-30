use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::str::FromStr;

use anyhow::{anyhow, Error};
use log::error;
use serde::{Deserialize, Serialize};

use crate::model::{DEFAULT_PADDING, MyTag};
use crate::op::Action;
use crate::op::tag_impl::{ReadWriteTag, TagImpl, WriteTagFile};
use crate::util::json_de::iter_json_array;

const READ_BUFFER_SIZE: usize = 16 * 1024;

pub struct ImpAction {
    reader: Box<dyn Read>,
    base_dir: Rc<Option<PathBuf>>,
    dry_run: bool,
}

impl ImpAction {
    pub fn new<P>(src_file_path: P,
                  base_dir: &Option<PathBuf>,
                  dry_run: bool) -> Result<Self, Error>
        where P: AsRef<Path>
    {
        let reader = get_file_reader(src_file_path)?;
        let base_dir = Rc::new(get_base_dir(base_dir)?);
        Ok(Self {
            reader,
            base_dir,
            dry_run,
        })
    }
}

fn do_record(my_file: &JsonRecord,
             base_directory: &Option<PathBuf>,
             dry_run: bool) -> Result<(), Error> {
    let path =
        if let Some(base_dir) = base_directory {
            Ok(base_dir.clone().join(&my_file.path))
        } else {
            PathBuf::from_str(&my_file.path)
        }?;

    let mut t = TagImpl::new(&path, dry_run)?;

    if write_tag(my_file, &mut t) {
        t.save()
    } else {
        Ok(())
    }
}

fn get_file_reader<P>(source_file: P) -> Result<Box<dyn Read>, Error>
    where P: AsRef<Path>
{
    let f = File::open(source_file)?;
    Ok(Box::new(
        BufReader::with_capacity(READ_BUFFER_SIZE, f)
    ))
}

fn get_base_dir(base_directory: &Option<PathBuf>) -> Result<Option<PathBuf>, Error> {
    if let Some(path) = base_directory {
        if !path.is_dir() {
            Err(anyhow!("{:?} is NOT an directory!", path))
        } else {
            Ok(Some(PathBuf::from(path)))
        }
    } else {
        Ok(None)
    }
}

impl Action for ImpAction {
    fn do_any(&mut self) -> Result<(), Error> {
        let mut it = iter_json_array(&mut self.reader);
        while let Some(item) = it.next() {
            match item {
                Ok(record) => {
                    let base_dir = Rc::clone(&self.base_dir);
                    let dry_run = self.dry_run;
                    do_record(&record, &base_dir, dry_run)
                        .err()
                        .map(|e| error!("Error: {}", e));
                }
                Err(e) => {
                    return Err(anyhow!("Could NOT process JSON data! (error {:?})", e));
                }
            }
        }
        Ok(())
    }
}

fn write_tag<T: ReadWriteTag>(my_file: &JsonRecord, t: &mut T) -> bool {
    let mut changed = false;

    if let Some(ref s) = my_file.tags.title {
        t.write_text_tag(&MyTag::Title, s);
        if !changed { changed = true }
    }
    if let Some(ref s) = my_file.tags.artist {
        t.write_text_tag(&MyTag::Artist, s);
        if !changed { changed = true }
    }
    if let Some(ref s) = my_file.tags.album_title {
        t.write_text_tag(&MyTag::AlbumTitle, s);
        if !changed { changed = true }
    }
    if let Some(ref s) = my_file.tags.album_artist {
        t.write_text_tag(&MyTag::AlbumArtist, s);
        if !changed { changed = true }
    }
    if let Some(ref s) = my_file.tags.genre {
        t.write_text_tag(&MyTag::Genre, s);
        if !changed { changed = true }
    }
    if let Some(ref s) = my_file.tags.composer {
        t.write_text_tag(&MyTag::Composer, s);
        if !changed { changed = true }
    }

    if let Some(u) = my_file.tags.year {
        t.write_numeric_tag(&MyTag::Year, u, DEFAULT_PADDING);
        if !changed { changed = true }
    }
    if let Some(u) = my_file.tags.track_number {
        t.write_numeric_tag(&MyTag::TrackNumber, u, DEFAULT_PADDING);
        if !changed { changed = true }
    }
    if let Some(u) = my_file.tags.track_total {
        t.write_numeric_tag(&MyTag::TrackTotal, u, DEFAULT_PADDING);
        if !changed { changed = true }
    }
    if let Some(u) = my_file.tags.disc_number {
        t.write_numeric_tag(&MyTag::DiscNumber, u, DEFAULT_PADDING);
        if !changed { changed = true }
    }
    if let Some(u) = my_file.tags.disc_total {
        t.write_numeric_tag(&MyTag::DiscTotal, u, DEFAULT_PADDING);
        if !changed { changed = true }
    }

    if let Some(ref s) = my_file.tags.comment {
        t.write_text_tag(&MyTag::Comment, s);
        if !changed { changed = true }
    }
    if let Some(ref s) = my_file.tags.copyright {
        t.write_text_tag(&MyTag::Copyright, s);
        if !changed { changed = true }
    }
    if let Some(ref s) = my_file.tags.lyrics {
        t.write_text_tag(&MyTag::Lyrics, s);
        if !changed { changed = true }
    }
    changed
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct JsonTag {
    title: Option<String>,
    artist: Option<String>,
    album_title: Option<String>,
    album_artist: Option<String>,
    genre: Option<String>,
    composer: Option<String>,
    year: Option<u32>,
    track_number: Option<u32>,
    track_total: Option<u32>,
    disc_number: Option<u32>,
    disc_total: Option<u32>,
    date: Option<String>,
    comment: Option<String>,
    copyright: Option<String>,
    lyrics: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRecord {
    path: String,
    tags: JsonTag,
}
