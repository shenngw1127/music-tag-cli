use std::borrow::Cow;
use std::path::{MAIN_SEPARATOR, Path, PathBuf};
use std::ffi::{OsStr, OsString};

/// parent_path_and_ext: use it\'s parent path and extension;
/// file_stem: use it\'s as file stem;
/// If file_stem contains MAIN_SEPARATOR will return None.
/// If file_stem is empty and ext is empty will return None.
pub fn combine_path<P>(parent_path_and_ext: P,
                       file_stem: &str) -> Option<PathBuf>
    where P: AsRef<Path> {
    let path_and_ext = parent_path_and_ext.as_ref();

    if file_stem.contains(MAIN_SEPARATOR) {
        return None;
    }

    let ext = my_ext(path_and_ext);

    if file_stem.is_empty() && ext.is_none() {
        return None;
    }

    let mut filename = OsString::new();
    filename.push(file_stem);

    if let Some(ext) = ext {
        filename.push(".");
        filename.push(ext);
    }

    if let Some(parent) = path_and_ext.parent() {
        let mut res = if path_and_ext == Path::new(".") {
            PathBuf::from("..")
        } else if path_and_ext == Path::new("..") {
            PathBuf::from("../..")
        } else {
            PathBuf::from(parent)
        };
        res.push(&filename);
        Some(res)
    } else {
        None
    }
}

fn my_ext(path: &Path) -> Option<&OsStr> {
    let ext = path.extension();
    if !ext.is_none() && !ext.unwrap().is_empty() {
        return ext;
    }

    match ext {
        None => {
            if let Some(stem) = path.file_stem() {
                let bytes = stem.as_encoded_bytes();
                if bytes.first().eq(&Some(&('.' as u8))) {
                    let (_, right) = bytes.split_at(1);
                    let stem_as_ext = unsafe { OsStr::from_encoded_bytes_unchecked(right) };
                    if !stem_as_ext.is_empty() {
                        return Some(stem_as_ext);
                    }
                }
            }
            None
        }
        Some(ext) if !ext.is_empty() => Some(ext),
        _ => None,
    }
}

/// path: the original path
/// If path exists, then try file_stem(1).ext, file_stem(2).ext ... until it reached the u16::MAX
/// return None if path no filename
/// return None if path no file_stem
/// return None if order number exceed the U16::MAX
pub fn get_dup_path(path: &Path) -> Option<Cow<Path>> {
    if path.file_name().is_none() {
        return None;
    }

    let empty = OsString::new();
    let mut stem_res = OsString::new();
    let (stem, ext) = match path.extension() {
        Some(ext) => {
            match path.file_stem() {
                Some(stem) => {
                    if ext.is_empty() {
                        stem_res.push(stem);
                        stem_res.push(".");
                        (&stem_res as &OsStr, ext)
                    } else {
                        (stem, ext)
                    }
                }
                None => return None,
            }
        }
        None => {
            match path.file_stem() {
                Some(stem) => {
                    if stem.as_encoded_bytes().first().eq(&Some(&('.' as u8))) {
                        let (_, right) = stem.as_encoded_bytes().split_at(1);
                        (&empty as &OsStr, unsafe { OsStr::from_encoded_bytes_unchecked(right) })
                    } else {
                        (stem, &empty as &OsStr)
                    }
                }
                None => return None,
            }
        }
    };

    if !path.exists() {
        return Some(Cow::Borrowed(path));
    }

    let mut i = 1u16;
    loop {
        let mut filename = stem.to_owned();
        filename.push(&format!("({})", i));
        if !ext.is_empty() {
            filename.push(".");
            filename.push(ext);
        }

        let mut new_path = PathBuf::from(path);
        new_path.set_file_name(&filename);

        if !new_path.exists() {
            break Some(Cow::Owned(new_path));
        } else {
            if i == u16::MAX {
                break None;
            }
            i += 1;
        }
    }
}

#[cfg(test)]
mod test {
    use std::fs;
    use std::path::{MAIN_SEPARATOR, Path, PathBuf};
    use crate::util::path::{combine_path, get_dup_path};

    #[test]
    fn test_combine_path() {
        let temp_fn = "examples/files/abc.lrc";
        assert_eq!(combine_path(temp_fn, "efg"),
                   Some(PathBuf::from("examples/files/efg.lrc")));

        let temp_fn = "examples/files/abc";
        assert_eq!(combine_path(temp_fn, "efg"),
                   Some(PathBuf::from("examples/files/efg")));

        let temp_fn = "examples/files/abc";
        assert_eq!(combine_path(temp_fn, ".txt"),
                   Some(PathBuf::from("examples/files/.txt")));

        let temp_fn = "examples/files/.txt";
        assert_eq!(combine_path(temp_fn, "efg"),
                   Some(PathBuf::from("examples/files/efg.txt")));

        let temp_fn = "examples/files/abc.lrc";
        assert_eq!(combine_path(temp_fn, ""),
                   Some(PathBuf::from("examples/files/.lrc")));

        let temp_fn = "examples/files/abc.lrc";
        assert_eq!(combine_path(temp_fn, ".txt"),
                   Some(PathBuf::from("examples/files/.txt.lrc")));

        let temp_fn = "examples/files/abc.lrc";
        assert_eq!(combine_path(temp_fn, "abc.tar.txt"),
                   Some(PathBuf::from("examples/files/abc.tar.txt.lrc")));

        let temp_fn = "examples/files/abc.lrc";
        assert_eq!(combine_path(temp_fn, ".abc.tar.txt"),
                   Some(PathBuf::from("examples/files/.abc.tar.txt.lrc")));

        let temp_fn = "examples/files/abc.lrc";
        assert_eq!(combine_path(temp_fn,
                                &(String::from("prefix")
                                    + &MAIN_SEPARATOR.to_string()
                                    + "suffix")),
                   None);

        let temp_fn = "examples/files/";
        assert_eq!(combine_path(temp_fn, ".txt"),
                   Some(PathBuf::from("examples/.txt")));

        let temp_fn = "examples/files/";
        assert_eq!(combine_path(temp_fn, ""),
                   None);

        let temp_fn = "examples/files/";
        assert_eq!(combine_path(temp_fn, "abc"),
                   Some(PathBuf::from("examples/abc")));

        let temp_fn = "/a.txt";
        assert_eq!(combine_path(temp_fn, "b"),
                   Some(PathBuf::from("/b.txt")));

        let temp_fn = "a.txt";
        assert_eq!(combine_path(temp_fn, "b"),
                   Some(PathBuf::from("b.txt")));

        let temp_fn = "/";
        assert_eq!(combine_path(temp_fn, "b"),
                   None);

        let temp_fn = ".";
        assert_eq!(combine_path(temp_fn, "b"),
                   Some(PathBuf::from("../b")));

        let temp_fn = "./";
        assert_eq!(combine_path(temp_fn, "b"),
                   Some(PathBuf::from("../b")));

        let temp_fn = ".../";
        assert_eq!(combine_path(temp_fn, "b"),
                   Some(PathBuf::from("b")));

        let temp_fn = "..";
        assert_eq!(combine_path(temp_fn, "b"),
                   Some(PathBuf::from("../../b")));

        let temp_fn = "../..";
        assert_eq!(combine_path(temp_fn, "b"),
                   Some(PathBuf::from("../b")));
    }

    #[test]
    fn test_get_dup_path_some() {
        const SRC_FILE: &'static str = "examples/files/003.lrc";

        let temp_fn = "examples/files/abc.lrc";
        let _ = fs::remove_file(temp_fn);
        assert_eq!(get_dup_path(Path::new(temp_fn)).unwrap(),
                   PathBuf::from("examples/files/abc.lrc"));

        fs::copy(SRC_FILE, temp_fn).unwrap();
        let temp_fn1 = "examples/files/abc(1).lrc";
        assert_eq!(get_dup_path(Path::new(temp_fn)).unwrap(),
                   PathBuf::from(temp_fn1));

        fs::copy(SRC_FILE, temp_fn1).unwrap();
        let temp_fn2 = "examples/files/abc(2).lrc";
        assert_eq!(get_dup_path(Path::new(temp_fn)).unwrap(),
                   PathBuf::from(temp_fn2));

        fs::copy(SRC_FILE, temp_fn2).unwrap();
        let temp_fn3 = "examples/files/abc(3).lrc";
        assert_eq!(get_dup_path(Path::new(temp_fn)).unwrap(),
                   PathBuf::from(temp_fn3));

        fs::remove_file(temp_fn2).unwrap();
        fs::remove_file(temp_fn1).unwrap();
        fs::remove_file(temp_fn).unwrap();

        assert_eq!(get_dup_path(Path::new("examples/")).unwrap(),
                   PathBuf::from("examples(1)"));
        assert_eq!(get_dup_path(Path::new("examples/.")).unwrap(),
                   PathBuf::from("examples(1)"));

        let temp_fn = "examples/files/.lrc";
        let _ = fs::remove_file(temp_fn);
        assert_eq!(get_dup_path(Path::new(temp_fn)).unwrap(),
                   PathBuf::from("examples/files/.lrc"));

        fs::copy(SRC_FILE, temp_fn).unwrap();
        let temp_fn1 = "examples/files/(1).lrc";
        assert_eq!(get_dup_path(Path::new(temp_fn)).unwrap(),
                   PathBuf::from(temp_fn1));

        fs::remove_file(temp_fn).unwrap();

        let temp_fn = "examples/files/....";
        let _ = fs::remove_file(temp_fn);
        assert_eq!(get_dup_path(Path::new(temp_fn)).unwrap(),
                   PathBuf::from("examples/files/...."));

        fs::copy(SRC_FILE, temp_fn).unwrap();
        let temp_fn1 = "examples/files/....(1)";
        assert_eq!(get_dup_path(Path::new(temp_fn)).unwrap(),
                   PathBuf::from(temp_fn1));

        fs::remove_file(temp_fn).unwrap();
    }

    #[test]
    fn test_get_dup_path_none() {
        assert_eq!(get_dup_path(Path::new(".")),
                   None);
        assert_eq!(get_dup_path(Path::new("")),
                   None);
        assert_eq!(get_dup_path(Path::new("examples/..")),
                   None);
        assert_eq!(get_dup_path(Path::new("/")),
                   None);
    }

    #[test]
    fn test_path_parent() {
        assert_eq!(Path::new("/").parent(), None);
        assert_eq!(Path::new(".").parent(), Some(Path::new("")));
        assert_eq!(Path::new("..").parent(), Some(Path::new("")));
        assert_eq!(Path::new("foo").parent(), Some(Path::new("")));
        assert_eq!(Path::new("...").parent(), Some(Path::new("")));
    }

    #[test]
    fn test_path_ext() {
        assert_eq!(Path::new("/").extension(), None);
        assert_eq!(Path::new(".").extension(), None);
        assert_eq!(Path::new("..").extension(), None);
        assert_eq!(Path::new("foo").extension(), None);
        // Note here: the last point is SEPARATOR, the first two are stem, so ext is empty str.
        assert_eq!(Path::new("...").extension().unwrap(), "");
    }

    #[test]
    fn test_file_stem() {
        assert_eq!(Path::new("/").file_stem(), None);
        assert_eq!(Path::new(".").file_stem(), None);
        assert_eq!(Path::new("..").file_stem(), None);
        assert_eq!(Path::new("foo").file_stem().unwrap(), "foo");

        // Note here: the last point is SEPARATOR, the first two are stem, so ext is empty str.
        assert_eq!(Path::new("...").file_stem().unwrap(), "..");
    }
}