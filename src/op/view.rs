use std::path::Path;
use anyhow::Error;
use log::error;

use crate::model::{MyTag, ALL_TAGS};
use crate::op::{Action, ReadAction, WalkAction};
use crate::op::tag_impl::{ReadTag, TagImpl};

pub struct ViewAction<'a> {
    dir: &'a Path,
    tags: &'a Vec<MyTag>,
    with_properties: bool,
}

impl<'a> ViewAction<'a> {
    pub fn new(dir: &'a Path,
               tags: &'a Vec<MyTag>,
               with_properties: bool) -> Self {
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

    const MAX_WIDTH: usize = 16 - 2;

    fn println_text_tag<T: ReadTag + ?Sized>(&self, t: &T, tag: &MyTag) {
        let tag_name = tag.to_string();
        println!("{} {} - {}", &tag_name,
                 " ".repeat(Self::MAX_WIDTH - &tag_name.len()),
                 &t.get_text_tag(tag).unwrap_or_default());
    }

    fn println_numeric_tag<T: ReadTag + ?Sized>(&self, t: &T, tag: &MyTag) {
        let tag_name = tag.to_string();
        println!("{} {} - {}", &tag_name,
                 " ".repeat(Self::MAX_WIDTH - tag_name.len()),
                 &to_string_default_empty(t.get_numeric_tag_string(tag)));
    }

    fn println_date_tag<T: ReadTag + ?Sized>(&self, t: &T, tag: &MyTag) {
        self.println_text_tag(t, tag)
    }

    fn println_properties<T: ReadTag + ?Sized>(t: &T) {
        println!("-- PROPERTY --");
        t.get_property_keys()
            .map_or_else(
                |e| error!("{:?}", e),
                |keys| {
                    let len = keys.len();
                    if len == 0 {
                        println!("no key.");
                    } else if len == 1 {
                        println!("{} key", len);
                    } else {
                        println!("{} keys.", len);
                    }

                    for key in keys {
                        println!("{}: {:?}", key, t.get_property(&key).unwrap_or_default());
                    }
                });
    }
}

impl Action for ViewAction<'_> {
    fn do_dir(&self) -> Result<(), Error> {
        self.do_dir_walk()
    }

    fn do_file(&self) -> Result<(), Error> {
        self.do_file_impl()
    }

    fn op_name(&self) -> &'static str {
        "view"
    }

    fn get_path(&self) -> &Path {
        &self.dir
    }

    fn get_tags(&self) -> &Vec<MyTag> {
        &self.tags
    }
}

impl WalkAction for ViewAction<'_> {
    fn do_one_file(&self, path: &Path) -> Result<(), Error> {
        self.do_one_file_read(path)
    }
}

impl ReadAction for ViewAction<'_> {
    fn get_tags_some(&self, t: &TagImpl) {
        if self.tags.is_empty() {
            return;
        }

        println!("-- TAGS --");
        for tag in self.tags {
            match tag {
                MyTag::Title => self.println_text_tag(t, &MyTag::Title),
                MyTag::Artist => self.println_text_tag(t, &MyTag::Artist),
                MyTag::AlbumTitle => self.println_text_tag(t, &MyTag::AlbumTitle),
                MyTag::Genre => self.println_text_tag(t, &MyTag::Genre),
                MyTag::Comment => self.println_text_tag(t, &MyTag::Comment),
                MyTag::AlbumArtist => self.println_text_tag(t, &MyTag::AlbumArtist),
                MyTag::Composer => self.println_text_tag(t, &MyTag::Composer),
                MyTag::Copyright => self.println_text_tag(t, &MyTag::Copyright),

                MyTag::Year => self.println_numeric_tag(t, &MyTag::Year),
                MyTag::TrackNumber => self.println_numeric_tag(t, &MyTag::TrackNumber),
                MyTag::TrackTotal => self.println_numeric_tag(t, &MyTag::TrackTotal),
                MyTag::DiscNumber => self.println_numeric_tag(t, &MyTag::DiscNumber),
                MyTag::DiscTotal => self.println_numeric_tag(t, &MyTag::DiscTotal),

                MyTag::Date => self.println_date_tag(t, &MyTag::Date),
            }
        }

        if self.with_properties {
            Self::println_properties(t);
        }
    }
}

const EMPTY_STR: &str = "";

fn to_string_default_empty(input: Option<impl ToString>) -> String {
    input.map_or(EMPTY_STR.to_string(), |v| { v.to_string() })
}

#[cfg(test)]
mod test {
    use super::to_string_default_empty;

    #[test]
    fn test_to_string_or_empty() {
        assert_eq!("", to_string_default_empty(None::<String>));
        assert_eq!("abc", to_string_default_empty(Some("abc")));
        assert_eq!("abc", to_string_default_empty(Some("abc".to_owned())));
    }
}
