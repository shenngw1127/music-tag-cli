extern crate lazy_static;

use anyhow::{anyhow, Error};
use audiotags::{AudioTag, Tag};
use lazy_static::lazy_static;
use log::{info, warn};
use std::collections::HashSet;
use std::path::Path;

use crate::model::MyTag;
use crate::op::{MAX_NUMBER, MIN_NATURAL_NUMBER};
use super::{ReadTag, ReadWriteTag, WriteTag, WriteTagFile};

pub struct AudioTagWrapper<'a> {
    file_name: &'a Path,
    tag: Box<dyn AudioTag + Send + Sync>,
}

impl<'a> AudioTagWrapper<'a> {
    pub fn new(file_path: &'a dyn AsRef<Path>) -> Result<Self, Error> {
        let file_name = file_path.as_ref();
        info!("Open file {:?}", &file_name);

        let res = Tag::new().read_from_path(file_name);
        if res.is_ok() {
            let tag = res.unwrap();
            Ok(AudioTagWrapper {
                file_name,
                tag,
            })
        } else {
            Err(anyhow!("No available tags for {:?} (error: {:?})", &file_name, res.err()))
        }
    }
}

impl ReadTag for AudioTagWrapper<'_> {
    fn get_path(&self) -> &Path {
        self.file_name
    }

    fn get_text_tag(&self, key: &MyTag) -> Option<String> {
        if key.is_text() || key.is_date() {
            let t = &self.tag;
            let result = match key {
                MyTag::Title => t.title(),
                MyTag::Artist => t.artist(),
                MyTag::AlbumTitle => t.album_title(),
                MyTag::Genre => t.genre(),
                MyTag::Comment => t.comment(),

                MyTag::AlbumArtist => t.album_artist(),
                MyTag::Composer => t.composer(),
                MyTag::Copyright => {
                    warn!("Not supported tag: {} in file {:?}", key, &self.file_name);
                    None
                }
                MyTag::Date => {
                    warn!("Not supported tag: {} in file {:?}", key, &self.file_name);
                    None
                }
                _ => None,
            };
            result.map_or(None, |e| { Some(e.to_owned()) })
        } else {
            None
        }
    }

    fn get_numeric_tag(&self, key: &MyTag) -> Option<u32> {
        if key.is_numeric() {
            let t = &self.tag;
            let result = match key {
                MyTag::Year => t.year().map_or(None,
                                               |i| {
                                                   if i < 0 || i > (MAX_NUMBER as i32) {
                                                       None
                                                   } else {
                                                       Some(i as u32)
                                                   }
                                               }),
                MyTag::TrackNumber => t.track_number().map_or(None,
                                                              |u| {
                                                                  Some(u as u32)
                                                              }),
                MyTag::TrackTotal => t.total_tracks().map_or(None,
                                                             |u| {
                                                                 Some(u as u32)
                                                             }),
                MyTag::DiscNumber => t.disc_number().map_or(None,
                                                            |u| {
                                                                Some(u as u32)
                                                            }),
                MyTag::DiscTotal => t.total_discs().map_or(None,
                                                           |u| {
                                                               Some(u as u32)
                                                           }),

                _ => None,
            };
            result
        } else {
            None
        }
    }

    fn get_numeric_tag_string(&self, key: &MyTag) -> Option<String> {
        self.get_numeric_tag(key).map(|t| {
            t.to_string()
        })
    }

    fn get_property_keys(&self) -> Result<Vec<String>, Error> {
        Err(anyhow!("Unsupported any properties."))
    }

    fn get_property(&self, _key: &str) -> Result<Vec<String>, Error> {
        Err(anyhow!("Unsupported any properties."))
    }
}

impl WriteTagFile for AudioTagWrapper<'_> {
    fn save(&mut self) -> Result<(), Error> {
        let tag = &mut self.tag;
        tag.write_to_path(&self.file_name.to_string_lossy())
            .map(|_| info!("Save file {:?} ok.", &self.file_name))
            .map_err(|e| anyhow!("Save file {:?} FAILED! (error: {})", &self.file_name, e))
    }
}

impl WriteTag for AudioTagWrapper<'_> {
    fn write_text_tag(&mut self, key: &MyTag, value: &str) {
        if key.is_text() || key.is_date() {
            let t = &mut self.tag;
            match key {
                MyTag::Title => {
                    t.remove_title();
                    t.set_title(value);
                    info!("file {:?} set {}: {}", &self.file_name, key, value);
                }
                MyTag::Artist => {
                    t.remove_artist();
                    t.set_artist(value);
                    info!("file {:?} set {}: {}", &self.file_name, key, value);
                }
                MyTag::AlbumTitle => {
                    t.remove_album_title();
                    t.set_album_title(value);
                    info!("file {:?} set {}: {}", &self.file_name, key, value);
                }
                MyTag::Genre => {
                    t.remove_genre();
                    t.set_genre(value);
                    info!("file {:?} set {}: {}", &self.file_name, key, value);
                }
                MyTag::Comment => {
                    t.remove_comment();
                    t.set_comment(value.to_owned());
                    info!("file {:?} set {}: {}", &self.file_name, key, value);
                }
                MyTag::AlbumArtist => {
                    t.remove_album_artist();
                    t.set_album_artist(value);
                    info!("file {:?} set {}: {}", &self.file_name, key, value);
                }
                MyTag::Composer => {
                    t.remove_composer();
                    t.set_composer(value.to_owned());
                    info!("file {:?} set {}: {}", &self.file_name, key, value);
                }
                MyTag::Copyright => {
                    warn!("Not supported tag {} in file {:?}, could NOT set {}",
                        key, &self.file_name, value);
                }
                MyTag::Date => {
                    warn!("Not supported tag {} in file {:?}, could NOT set {}",
                        key, &self.file_name, value);
                }
                _ => (),
            }
        }
    }

    fn write_numeric_tag(&mut self, key: &MyTag, value: u32, _padding: usize) {
        if key.is_numeric() {
            let t = &mut self.tag;
            match key {
                MyTag::Year => {
                    if value <= MAX_NUMBER {
                        t.set_year(value as i32);
                        info!("file {:?} set {}: {}", &self.file_name, key, value);
                    }
                }
                MyTag::TrackNumber => {
                    if value >= MIN_NATURAL_NUMBER && value <= MAX_NUMBER {
                        t.set_track_number(value as u16);
                        info!("file {:?} set {}: {}", &self.file_name, key, value);
                    }
                }
                MyTag::TrackTotal => {
                    if value >= MIN_NATURAL_NUMBER && value <= MAX_NUMBER {
                        t.set_total_tracks(value as u16);
                        info!("file {:?} set {}: {}", &self.file_name, key, value);
                    }
                }
                MyTag::DiscNumber => {
                    if value >= MIN_NATURAL_NUMBER && value <= MAX_NUMBER {
                        t.set_disc_number(value as u16);
                        info!("file {:?} set {}: {}", &self.file_name, key, value);
                    }
                }
                MyTag::DiscTotal => {
                    if value >= MIN_NATURAL_NUMBER && value <= MAX_NUMBER {
                        t.set_total_discs(value as u16);
                        info!("file {:?} set {}: {}", &self.file_name, key, value);
                    }
                }
                _ => (),
            }
        }
    }
}

impl ReadWriteTag for AudioTagWrapper<'_> {}

lazy_static! {
    static ref MP3_SUFFIX: Vec<&'static str> = vec![".mp3"];
    static ref MP4_SUFFIX: Vec<&'static str> = vec![".m4a", ".m4b", ".m4p", ".m4v", ".isom", ".mp4"];
    static ref FLAC_SUFFIX: Vec<&'static str> = vec![".flac"];

    static ref ALL_SUFFIX: HashSet<&'static str> = {
        let mut m = HashSet::new();
        m.extend(MP3_SUFFIX.iter().cloned().collect::<HashSet<&str>>());
        m.extend(MP4_SUFFIX.iter().cloned().collect::<HashSet<&str>>());
        m.extend(FLAC_SUFFIX.iter().cloned().collect::<HashSet<&str>> ());

        m
    };
}

fn all_suffix() -> &'static HashSet<&'static str> {
    &*ALL_SUFFIX
}

pub fn available_suffix(file_name: &str) -> bool {
    for suffix in all_suffix().iter() {
        if file_name.to_lowercase().ends_with(*suffix) {
            return true;
        }
    }
    false
}