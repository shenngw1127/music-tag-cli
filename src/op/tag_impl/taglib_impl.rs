use anyhow::{anyhow, Error};
use log::info;
use taglib::{File as TagLibFile, FileType as TagLibFileType};
use std::path::Path;

use crate::model::MyTag;
use crate::op::{MAX_NUMBER, MIN_NATURAL_NUMBER};
use super::{ReadTag, ReadWriteTag, WriteTag, WriteTagFile};

pub struct TaglibWrapper<'a> {
    file_name: &'a Path,
    file: TagLibFile,
}

impl<'a> TaglibWrapper<'a> {
    pub fn new(file_path: &'a dyn AsRef<Path>) -> Result<Self, Error> {
        let file_name = file_path.as_ref();
        info!("Open file {:?}", &file_name);

        let res = TagLibFile::new(&file_name);
        if res.is_ok() {
            let file = res.unwrap();
            if file.tag().is_ok() {
                Ok(TaglibWrapper {
                    file_name,
                    file,
                })
            } else {
                Err(anyhow!("No available tags for {:?} (error: {:?})",
                    &file_name,
                    file.tag().err().unwrap()))
            }
        } else {
            Err(anyhow!("Invalid file {:?} (error: {:?})", &file_name, res.err()))
        }
    }
}

impl ReadTag for TaglibWrapper<'_> {
    fn get_path(&self) -> &Path {
        self.file_name
    }

    fn get_text_tag(&self, key: &MyTag) -> Option<String> {
        if key.is_text() || key.is_date() {
            let t = &self.file.tag().unwrap();
            let result = match key {
                MyTag::Title => t.title(),
                MyTag::Artist => t.artist(),
                MyTag::AlbumTitle => t.album(),
                MyTag::Genre => t.genre(),
                MyTag::Comment => t.comment(),

                MyTag::AlbumArtist => t.album_artist(),
                MyTag::Composer => t.composer(),
                MyTag::Copyright => t.copyright(),

                MyTag::Date => t.date(),

                _ => None,
            };
            result
        } else {
            None
        }
    }

    fn get_numeric_tag(&self, key: &MyTag) -> Option<u32> {
        if key.is_numeric() {
            let t = &self.file.tag().unwrap();
            let result = match key {
                MyTag::Year => t.year(),
                MyTag::TrackNumber => t.track(),

                MyTag::TrackTotal => t.track_total(),
                MyTag::DiscNumber => t.disc_number(),
                MyTag::DiscTotal => t.disc_total(),

                _ => None,
            };
            result
        } else {
            None
        }
    }

    fn get_numeric_tag_string(&self, key: &MyTag) -> Option<String> {
        if key.is_numeric() {
            let t = &self.file.tag().unwrap();
            let result = match key {
                MyTag::Year => t.year().map(|t| t.to_string()),
                MyTag::TrackNumber => t.track_number_string(),

                MyTag::TrackTotal => t.track_total_string(),
                MyTag::DiscNumber => t.disc_number_string(),
                MyTag::DiscTotal => t.disc_total_string(),

                _ => None,
            };
            result
        } else {
            None
        }
    }

    fn get_property_keys(&self) -> Result<Vec<String>, Error> {
        self.file.keys().map_err(|e| anyhow!(e))
    }

    fn get_property(&self, key: &str) -> Result<Vec<String>, Error> {
        self.file.get_property(key).map_err(|e| anyhow!(e))
    }
}

impl WriteTagFile for TaglibWrapper<'_> {
    fn save(&mut self) -> Result<(), Error> {
        if self.file.save() {
            info!("Save file {:?} ok.", &self.file_name);
            Ok(())
        } else {
            Err(anyhow!("Save file {:?} FAILED! \
            Please check if file exists and it's attribute is NOT \"Read-only\".",
                     &self.file_name))
        }
    }
}

impl WriteTag for TaglibWrapper<'_> {
    fn write_text_tag(&mut self, key: &MyTag, value: &str) {
        if key.is_text() || key.is_date() {
            {
                let t = &mut self.file.tag().unwrap();
                let completed = match key {
                    MyTag::Title => {
                        t.set_title(value);
                        info!("file {:?} set {}: {}", &self.file_name, key, value);
                        true
                    }
                    MyTag::Artist => {
                        t.set_artist(value);
                        info!("file {:?} set {}: {}", &self.file_name, key, value);
                        true
                    }
                    MyTag::AlbumTitle => {
                        t.set_album(value);
                        info!("file {:?} set {}: {}", &self.file_name, key, value);
                        true
                    }
                    MyTag::Genre => {
                        t.set_genre(value);
                        info!("file {:?} set {}: {}", &self.file_name, key, value);
                        true
                    }
                    MyTag::Comment => {
                        t.set_comment(value);
                        info!("file {:?} set {}: {}", &self.file_name, key, value);
                        true
                    }
                    _ => false,
                };

                if completed {
                    return;
                }
            }

            match key {
                MyTag::AlbumArtist => {
                    self.file.set_album_artist(value);
                    info!("file {:?} set {}: {}", &self.file_name, key, value);
                }
                MyTag::Composer => {
                    self.file.set_composer(value);
                    info!("file {:?} set {}: {}", &self.file_name, key, value);
                }
                MyTag::Copyright => {
                    self.file.set_copyright(value);
                    info!("file {:?} set {}: {}", &self.file_name, key, value);
                }

                MyTag::Date => {
                    self.file.set_date(value);
                    info!("file {:?} set {}: {}", &self.file_name, key, value);
                }
                _ => (),
            }
        }
    }

    fn write_numeric_tag(&mut self, key: &MyTag, value: u32, padding: usize) {
        if key.is_numeric() {
            {
                let t = &mut self.file.tag().unwrap();
                let completed = match key {
                    MyTag::Year => {
                        if value <= MAX_NUMBER {
                            t.set_year(value);
                            info!("file {:?} set {}: {}", &self.file_name, key, value);
                            true
                        } else {
                            false
                        }
                    }
                    _ => false,
                };
                if completed {
                    return;
                }
            }

            if value >= MIN_NATURAL_NUMBER && value <= MAX_NUMBER {
                match key {
                    MyTag::TrackNumber => {
                        self.file.set_track_number(value, padding);
                        info!("file {:?} set {}: {} with padding {}", &self.file_name,
                            key, value, padding);
                    }
                    MyTag::TrackTotal => {
                        self.file.set_track_total(value, padding);
                        info!("file {:?} set {}: {} with padding {}", &self.file_name,
                            key, value, padding);
                    }
                    MyTag::DiscNumber => {
                        self.file.set_disc_number(value, padding);
                        info!("file {:?} set {}: {} with padding {}", &self.file_name,
                            key, value, padding);
                    }
                    MyTag::DiscTotal => {
                        self.file.set_disc_total(value, padding);
                        info!("file {:?} set {}: {} with padding {}", &self.file_name,
                            key, value, padding);
                    }
                    _ => (),
                }
            }
        }
    }
}

impl ReadWriteTag for TaglibWrapper<'_> {}

pub fn available_suffix(file_name: &str) -> bool {
    for suffix in TagLibFileType::all_suffix().iter() {
        if file_name.to_lowercase().ends_with(*suffix) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod test {
    use super::available_suffix;

    #[test]
    fn test_available_suffix() {
        // lowercase
        assert_eq!(true, available_suffix("abc.aac"));
        assert_eq!(true, available_suffix("abc.mp3"));
        assert_eq!(true, available_suffix("abc.ogg"));
        assert_eq!(true, available_suffix("abc.flac"));
        assert_eq!(true, available_suffix("abc.mpc"));
        assert_eq!(true, available_suffix("abc.wv"));
        assert_eq!(true, available_suffix("abc.spx"));
        assert_eq!(true, available_suffix("abc.tta"));
        assert_eq!(true, available_suffix("abc.mp4"));
        assert_eq!(true, available_suffix("abc.m4a"));
        assert_eq!(true, available_suffix("abc.asf"));
        assert_eq!(true, available_suffix("abc.aif"));
        assert_eq!(true, available_suffix("abc.aiff"));
        assert_eq!(true, available_suffix("abc.wav"));
        assert_eq!(true, available_suffix("abc.ape"));
        assert_eq!(true, available_suffix("abc.mod"));
        assert_eq!(true, available_suffix("abc.s3m"));
        assert_eq!(true, available_suffix("abc.xm"));
        assert_eq!(true, available_suffix("abc.opus"));
        assert_eq!(true, available_suffix("abc.dsf"));
        assert_eq!(true, available_suffix("abc.dff"));
        // uppercase
        assert_eq!(true, available_suffix("ABC.DFF"));

        // not exists
        assert_eq!(false, available_suffix("others.data"));
        assert_eq!(false, available_suffix("others"));
    }
}