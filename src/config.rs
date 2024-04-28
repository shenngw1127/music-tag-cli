use std::fs;

use homedir::get_my_home;
use lazy_static::lazy_static;
use serde::Deserialize;

lazy_static! {
    static ref CONFIG: Config = toml::from_str(
            &get_toml_content().unwrap_or_else(|| "".to_owned()))
        .unwrap_or_else(|_| Config::default());
}

#[derive(Deserialize)]
struct Config {
    log_level: Option<String>,
    tag_lib: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            log_level: None,
            tag_lib: None,
        }
    }
}

fn get_toml_content() -> Option<String> {
    let home = get_my_home().unwrap().unwrap().to_path_buf();
    let toml_path = home.join(".music-tag-cli.toml");
    if toml_path.exists() && toml_path.is_file() {
        fs::read_to_string(toml_path)
            .map_or_else(|_| None, |s| Some(s))
    } else {
        None
    }
}

pub fn get_log_level() -> &'static Option<String> {
    &CONFIG.log_level
}

pub fn get_tag_lab() -> &'static Option<String> {
    &CONFIG.tag_lib
}
