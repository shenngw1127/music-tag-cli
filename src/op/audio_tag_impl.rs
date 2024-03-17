use audiotags::{AudioTag, Tag};
use opencc_rust::OpenCC;
use std::path::Path;

use crate::op::to_string_or_empty;

fn view_tags_via_audio_tag(file_str: &str, t: &mut Box<dyn AudioTag + Send + Sync>) {
    println!("-- TAGS --");
    println!("title           - {}", t.title().unwrap_or_default());
    println!("artist          - {}", t.artist().unwrap_or_default());
    println!("album           - {}", t.album_title().unwrap_or_default());
    println!("album artist    - {}", t.album_artist().unwrap_or_default());
    println!("genre           - {}", t.genre().unwrap_or_default());
    println!("composer        - {}", t.composer().unwrap_or_default());

    println!("year            - {}", to_string_or_empty(t.year()));
    println!("track number    - {}", to_string_or_empty(t.track_number()));
    println!("track total     - {}", to_string_or_empty(t.total_tracks()));
    println!("disc number     - {}", to_string_or_empty(t.disc_number()));
    println!("disc total      - {}", to_string_or_empty(t.total_discs()));

    println!("comment         - {}", t.comment().unwrap_or_default());
}

pub fn convert_tags_via_audio_tag(file_str: &str, t: &mut Box<dyn AudioTag + Send + Sync>, open_cc: &OpenCC) {
    let title = open_cc.convert(&t.title().unwrap_or_default());
    let artist = open_cc.convert(&t.artist().unwrap_or_default());
    let album_title = open_cc.convert(&t.album_title().unwrap_or_default());
    //let year = t.year().unwrap_or_default();
    let comment = open_cc.convert(&t.comment().unwrap_or_default());
    //let track = t.track().unwrap_or_default();
    let genre = open_cc.convert(&t.genre().unwrap_or_default());
    let album_artist = open_cc.convert(&t.album_artist().unwrap_or_default());
    let composer = open_cc.convert(&t.composer().unwrap_or_default());

    println!("Get tags from file {} ok.", file_str);
    t.remove_title();
    t.set_title(&title);

    t.remove_artist();
    t.set_artist(&artist);

    t.remove_album_title();
    t.set_album_title(&album_title);

    t.remove_comment();
    t.set_comment(comment);

    t.remove_genre();
    t.set_genre(&genre);

    t.remove_album_artist();
    t.set_album_artist(&album_artist);

    t.remove_composer();
    t.set_composer(composer);

    let result = t.write_to_path(file_str);
    if result.is_ok() {
        println!("Set tags into file {} ok.", file_str);
    } else {
        println!("Error {:?} for file {}.", result.err(), file_str);
    }
}

fn convert_one_file(input_path: &Path, open_cc: &OpenCC) {
    let file_str = input_path.to_str().unwrap();
    let tag = Tag::new().read_from_path(file_str);

    if tag.is_ok() {
        println!("File: {}", file_str);
        convert_tags_via_audio_tag(file_str, &mut tag.unwrap(), open_cc);
    } else {
        println!("Invalid file {} (error: {:?})", file_str, tag.err());
    }
}
