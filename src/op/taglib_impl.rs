use anyhow::{anyhow, Error};

use opencc_rust::OpenCC;
use std::path::Path;
use taglib::File as TagLibFile;
use walkdir::WalkDir;

use crate::op::{Action, ALL_TAGS, ConvEnProfile, MyTag, TEXT_TAGS, to_string_or_empty};

pub struct ViewAction<'a> {
    dir: &'a Path,
    tags: &'a Vec<MyTag>,
    with_properties: bool,
}

impl<'a> ViewAction<'a> {
    pub fn new(dir: &'a Path, tags: &'a Vec<MyTag>, with_properties: bool) -> Self {
        ViewAction {
            dir,
            tags: if !tags.is_empty() {
                tags
            } else {
                &ALL_TAGS
            },
            with_properties,
        }
    }
}

impl<'a> Action for ViewAction<'a> {
    fn op_name(&self) -> &'static str {
        "view"
    }

    fn do_it(&self) -> Result<(), Error> {
        if self.dir.is_file() {
            view_one_file_with_check(self.dir, self.tags, self.with_properties)
        } else if self.dir.is_dir() {
            view_dir_with_check(self.dir, &self.tags, self.with_properties)
        } else {
            Err(anyhow!("Could NOT perform action {} for {:?}!", self.op_name(), self.dir))
        }
    }
}

fn view_dir_with_check(dir: &Path,
                       tags: &Vec<MyTag>,
                       with_properties: bool) -> Result<(), Error> {
    println!("dir: {:?}", dir);
    for entry in WalkDir::new(dir.to_str().unwrap())
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok()) {
        if entry.metadata().unwrap().is_file() {
            let path = entry.path();
            let file_name_lowercase = path.to_str().unwrap().to_lowercase();
            if file_name_lowercase.ends_with(".flac")
                || file_name_lowercase.ends_with(".mp3")
                || file_name_lowercase.ends_with(".dsf") {
                view_one_file(path, tags, with_properties)
            }
        }
    }

    Ok(())
}

fn view_one_file_with_check(input_path: &Path,
                            tags: &Vec<MyTag>,
                            with_properties: bool) -> Result<(), Error> {
    match check_input_path(input_path) {
        Ok(_) => (),
        Err(error) => return Err(error)
    }

    view_one_file(&input_path, tags, with_properties);
    Ok(())
}

fn view_one_file(input_path: &Path,
                 tags: &Vec<MyTag>,
                 with_properties: bool) {
    let file_str = input_path.to_str().unwrap();
    let file = TagLibFile::new(file_str);

    if file.is_ok() {
        println!("File: {}", file_str);
        view_tags(file_str, &file.unwrap(), tags, with_properties);
    } else {
        println!("Invalid file {} (error: {:?})", file_str, file.err());
    }
}

fn view_tags(file_name: &str,
             file: &TagLibFile,
             tags: &Vec<MyTag>,
             with_properties: bool) {
    if file.tag().is_err() {
        println!("No available tags for {} (error: {:?})", file_name, file.tag().err());
    } else {
        println!("-- TAGS --");
        view_tags_some(file, tags, with_properties);
    }
}

fn view_tags_some(file: &TagLibFile, tags: &Vec<MyTag>, with_properties: bool) {
    let t = file.tag().unwrap();
    for tag in tags {
        match tag {
            MyTag::Title =>
                println!("title           - {}", t.title().unwrap_or_default()),
            MyTag::Artist =>
                println!("artist          - {}", t.artist().unwrap_or_default()),
            MyTag::AlbumTitle =>
                println!("album           - {}", t.album().unwrap_or_default()),
            MyTag::AlbumArtist =>
                println!("album artist    - {}", t.album_artist().unwrap_or_default()),
            MyTag::Genre =>
                println!("genre           - {}", t.genre().unwrap_or_default()),
            MyTag::Composer =>
                println!("composer        - {}", t.composer().unwrap_or_default()),
            MyTag::Year =>
                println!("year            - {}", to_string_or_empty(t.year())),
            MyTag::TrackNumber =>
                println!("track number    - {}", to_string_or_empty(t.track())),
            MyTag::TrackTotal =>
                println!("track total     - {}", to_string_or_empty(t.track_total())),
            MyTag::DiscNumber =>
                println!("disc number     - {}", to_string_or_empty(t.disc_number())),
            MyTag::DiscTotal =>
                println!("disc total      - {}", to_string_or_empty(t.disc_total())),
            MyTag::Comment =>
                println!("comment         - {}", t.comment().unwrap_or_default()),
        }
    }

    if with_properties {
        view_properties(file);
    }
}

fn view_tags_all(file: &TagLibFile, with_properties: bool) {
    let t = file.tag().unwrap();
    println!("title           - {}", t.title().unwrap_or_default());
    println!("artist          - {}", t.artist().unwrap_or_default());
    println!("album           - {}", t.album().unwrap_or_default());
    println!("album artist    - {}", t.album_artist().unwrap_or_default());
    println!("genre           - {}", t.genre().unwrap_or_default());
    println!("composer        - {}", t.composer().unwrap_or_default());

    println!("year            - {}", to_string_or_empty(t.year()));
    println!("track number    - {}", to_string_or_empty(t.track()));
    println!("track total     - {}", to_string_or_empty(t.track_total()));
    println!("disc number     - {}", to_string_or_empty(t.disc_number()));
    println!("disc total      - {}", to_string_or_empty(t.disc_total()));

    println!("comment         - {}", t.comment().unwrap_or_default());

    if with_properties {
        view_properties(file);
    }
}

fn view_properties(file: &TagLibFile) {
    println!("-- PROPERTY --");
    let result_keys = file.keys();
    if result_keys.is_ok() {
        let keys = result_keys.unwrap();
        let len = keys.len();
        if len == 0 {
            println!("no key.");
        } else if len == 1 {
            println!("{} key", len);
        } else {
            println!("{} keys.", len);
        }

        for key in keys {
            println!("{}: {:?}",
                     key,
                     file.get_property(&key).unwrap_or_default());
        }
    }
}

pub struct ConvZhAction<'a> {
    open_cc: &'a OpenCC,
    dir: &'a Path,
    tags: &'a Vec<MyTag>,
}

impl<'a> ConvZhAction<'a> {
    pub fn new(open_cc: &'a OpenCC,
               dir: &'a Path,
               tags: &'a mut Vec<MyTag>) -> Self {
        ConvZhAction {
            open_cc,
            dir,
            tags: if !tags.is_empty() {
                tags.retain(|&x| x.is_text());
                tags
            } else {
                &TEXT_TAGS
            },
        }
    }
}

impl<'a> Action for ConvZhAction<'a> {
    fn op_name(&self) -> &'static str {
        "conv-zh"
    }

    fn do_it(&self) -> Result<(), Error> {
        if self.dir.is_file() {
            convert_one_file_with_check(self.dir, self.open_cc, self.tags)
        } else if self.dir.is_dir() {
            convert_dir_with_check(self.dir, self.open_cc, self.tags)
        } else {
            Err(anyhow!("Could NOT perform action {} for {:?}!", self.op_name(), self.dir))
        }
    }
}

pub fn convert_dir_with_check(dir: &Path,
                              open_cc: &OpenCC,
                              tags: &Vec<MyTag>) -> Result<(), Error> {
    println!("dir: {:?}", dir);
    for entry in WalkDir::new(dir.to_str().unwrap())
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok()) {
        if entry.metadata().unwrap().is_file() {
            let path = entry.path();
            let file_name_lowercase = path.to_str().unwrap().to_lowercase();
            if file_name_lowercase.ends_with(".flac")
                || file_name_lowercase.ends_with(".mp3")
                || file_name_lowercase.ends_with(".dsf") {
                convert_one_file(path, open_cc, tags)
            }
        }
    }

    Ok(())
}

pub fn convert_one_file_with_check(input_path: &Path,
                                   open_cc: &OpenCC,
                                   tags: &Vec<MyTag>) -> Result<(), Error> {
    match check_input_path(input_path) {
        Ok(_) => (),
        Err(error) => return Err(error)
    }

    convert_one_file(input_path, &open_cc, tags);
    Ok(())
}

fn check_input_path(input_path: &Path) -> Result<(), Error> {
    if !input_path.is_file() {
        Err(anyhow!("{input_path:?} is not file!"))
    } else {
        Ok(())
    }
}

fn convert_one_file(input_path: &Path,
                    open_cc: &OpenCC,
                    tags: &Vec<MyTag>) {
    let file_str = input_path.to_str().unwrap();
    let file = TagLibFile::new(file_str);

    if file.is_ok() {
        println!("File: {}", file_str);
        convert_tags(file_str, &mut file.unwrap(), open_cc, tags);
    } else {
        println!("Invalid file {} (error: {:?})", file_str, file.err());
    }
}


fn convert_tags(file_name: &str,
                file: &mut TagLibFile,
                open_cc: &OpenCC,
                tags: &Vec<MyTag>) {
    if file.tag().is_err() {
        println!("No available tags for {} (error: {:?})", file_name, file.tag().err());
    } else {
        convert_tags_some(file, open_cc, tags);
    }
}

fn convert_tags_some(file: &mut TagLibFile,
                     open_cc: &OpenCC,
                     tags: &Vec<MyTag>) {
    {
        if tags.is_empty() {
            return;
        }

        let mut t = file.tag().unwrap();

        for tag in tags {
            match tag {
                MyTag::Title => {
                    let title = open_cc.convert(&t.title().unwrap_or_default());
                    t.set_title(&title);
                    println!("set title: {}", title);
                }
                MyTag::Artist => {
                    let artist = open_cc.convert(&t.artist().unwrap_or_default());
                    t.set_artist(&artist);
                    println!("set artist: {}", artist);
                }
                MyTag::AlbumTitle => {
                    let album = open_cc.convert(&t.album().unwrap_or_default());
                    t.set_album(&album);
                    println!("set album: {}", album);
                }
                MyTag::Genre => {
                    let genre = open_cc.convert(&t.genre().unwrap_or_default());
                    t.set_genre(&genre);
                    println!("set genre: {}", genre);
                }
                MyTag::Comment => {
                    let comment = open_cc.convert(&t.comment().unwrap_or_default());
                    t.set_comment(&comment);
                    println!("set comment: {}", comment);
                }
                _ => {}
            }
        }
    }

    for tag in tags {
        match tag {
            MyTag::AlbumArtist => {
                let album_artist = open_cc.convert(&file.album_artist().unwrap_or_default());
                file.set_album_artist(&album_artist);
                println!("set album_artist: {}", album_artist);
            }
            MyTag::Composer => {
                let composer = open_cc.convert(&file.composer().unwrap_or_default());
                file.set_composer(&composer);
                println!("set composer: {}", composer);
            }
            _ => {}
        }
    }

    file.save();
}

// fn convert_tags_all(file: &mut TagLibFile, open_cc: &OpenCC) {
//     let mut t = file.tag().unwrap();
//
//     let title = open_cc.convert(&t.title().unwrap_or_default());
//     let artist = open_cc.convert(&t.artist().unwrap_or_default());
//     let album = open_cc.convert(&t.album().unwrap_or_default());
//     let comment = open_cc.convert(&t.comment().unwrap_or_default());
//     let genre = open_cc.convert(&t.genre().unwrap_or_default());
//
//     let album_artist = open_cc.convert(&file.album_artist().unwrap_or_default());
//     let composer = open_cc.convert(&file.composer().unwrap_or_default());
//
//     println!("Get tags ok.");
//
//     t.set_title(&title);
//     t.set_artist(&artist);
//     t.set_album(&album);
//     t.set_comment(&comment);
//     t.set_genre(&genre);
//
//     file.set_album_artist(&album_artist);
//     file.set_composer(&composer);
//
//     file.save();
//     println!("Set tags ok.");
// }

pub struct ConvEnAction<'a> {
    dir: &'a Path,
    tags: &'a Vec<MyTag>,
    profile: ConvEnProfile,
}

impl<'a> ConvEnAction<'a> {
    pub fn new(dir: &'a Path,
               tags: &'a mut Vec<MyTag>,
               profile: ConvEnProfile) -> Self {
        ConvEnAction {
            dir,
            tags: if !tags.is_empty() {
                tags.retain(|&x| x.is_text());
                tags
            } else {
                &TEXT_TAGS
            },
            profile,
        }
    }
}

impl<'a> Action for ConvEnAction<'a> {
    fn op_name(&self) -> &'static str {
        "conv-en"
    }

    fn do_it(&self) -> Result<(), Error> {
        println!("ConvEnAction");
        Ok(())
        // if self.dir.is_file() {
        //     view_one_file_with_check(self.dir, self.tags, self.with_properties)
        // } else if self.dir.is_dir() {
        //     view_dir_with_check(self.dir, &self.tags, self.with_properties)
        // } else {
        //     Err(anyhow!("Could NOT perform action {} for {:?}!", self.op_name(), self.dir))
        // }
    }
}