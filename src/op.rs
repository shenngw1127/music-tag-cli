extern crate lazy_static;

use anyhow::Error;
use clap::{Args, Parser, Subcommand, ValueEnum};
use opencc_rust::{DefaultConfig, generate_static_dictionary, OpenCC};
use std::env;
use std::path::PathBuf;

use lazy_static::lazy_static;

use crate::op::taglib_impl::{ConvEnAction, ConvZhAction, ViewAction};

mod audio_tag_impl;
mod taglib_impl;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct App {
    // #[clap(flatten)]
    // global_opts: GlobalOpts,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
#[command(author, version, about, long_about = None)]
enum Command {
    View(ViewArgs),
    ConvZh(ConvZhArgs),
    ConvEn(ConvEnArgs),
}

#[derive(Debug, Args)]
struct GlobalOpts {
    #[clap(short, long, value_delimiter = ',')]
    #[arg(help = "Process only specified tags, if not set, it will process ALL tags.")]
    tags: Vec<MyTag>,
}

#[derive(Debug, Args)]
#[command(arg_required_else_help = true, long_about = "View tags.")]
struct ViewArgs {
    #[clap(flatten)]
    global_opts: GlobalOpts,

    #[clap(long, default_value_t = false)]
    #[arg(help = "Show properties or NOT (default).")]
    with_properties: bool,

    #[arg(value_hint = clap::ValueHint::FilePath)]
    #[arg(help = "Assign the path of your SOURCE file. It must point to a file path.")]
    directory: PathBuf,
    // a list of other write args
}

#[derive(Parser, Debug)]
#[command(arg_required_else_help = true, long_about = "Convert text tags in Chinese between Traditional and Simplified.")]
struct ConvZhArgs {
    #[arg(short, long, value_enum)]
    profile: ConvZhProfile,

    #[clap(flatten)]
    global_opts: GlobalOpts,

    #[arg(value_hint = clap::ValueHint::FilePath)]
    #[arg(help = "Assign the path of your SOURCE file. It must point to a file path.")]
    directory: PathBuf,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum ConvZhProfile {
    /// Traditional Chinese (Hong Kong Standard) to Simplified Chinese
    HK2S,

    /// Traditional Chinese (Hong Kong Standard) to Traditional Chinese
    HK2T,

    /// New Japanese Kanji (Shinjitai) to Traditional Chinese Characters (Kyūjitai)
    JP2T,

    /// Simplified Chinese to Traditional Chinese
    S2T,

    /// Simplified Chinese to Traditional Chinese (Taiwan Standard)
    S2TW,

    /// Simplified Chinese to Traditional Chinese (Taiwan Standard) with Taiwanese idiom
    S2TWP,

    /// Traditional Chinese (OpenCC Standard) to Hong Kong Standard
    T2HK,

    /// Traditional Chinese Characters (Kyūjitai) to New Japanese Kanji (Shinjitai)
    T2JP,

    /// Traditional Chinese (OpenCC Standard) to Taiwan Standard
    T2TW,

    /// Traditional Chinese to Simplified Chinese
    T2S,

    /// Simplified Chinese to Traditional Chinese (Hong Kong Standard)
    S2HK,

    /// Traditional Chinese (Taiwan Standard) to Simplified Chinese
    TW2S,

    /// Traditional Chinese (Taiwan Standard) to Simplified Chinese with Mainland Chinese idiom
    TW2SP,

    /// Traditional Chinese (Taiwan Standard) to Traditional Chinese
    TW2T,
}

#[derive(Parser, Debug)]
#[command(arg_required_else_help = true, long_about = "Convert text tags in English between lowercase and uppercase.")]
struct ConvEnArgs {
    #[arg(short, long, value_enum)]
    profile: ConvEnProfile,

    #[clap(flatten)]
    global_opts: GlobalOpts,

    #[arg(value_hint = clap::ValueHint::FilePath)]
    #[arg(help = "Assign the path of your SOURCE file. It must point to a file path.")]
    directory: PathBuf,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum ConvEnProfile {
    LowerCase,
    UpperCase,
}

#[derive(Debug, Copy, Clone, ValueEnum, Eq, Hash, PartialEq)]
enum MyTag {
    Title,
    Artist,
    AlbumTitle,
    AlbumArtist,
    Genre,
    Composer,
    Year,
    TrackNumber,
    TrackTotal,
    DiscNumber,
    DiscTotal,
    Comment,
}

enum MyTagDataType {
    Text,
    Number,
    Date,
}

impl MyTag {
    pub fn is_text(&self) -> bool {
        match self.data_type() {
            MyTagDataType::Text => true,
            _ => false,
        }
    }

    pub fn is_number(&self) -> bool {
        match self.data_type() {
            MyTagDataType::Number => true,
            _ => false,
        }
    }

    pub fn is_date(&self) -> bool {
        match self.data_type() {
            MyTagDataType::Date => true,
            _ => false,
        }
    }

    fn data_type(&self) -> MyTagDataType {
        match self {
            MyTag::Title => MyTagDataType::Text,
            MyTag::Artist => MyTagDataType::Text,
            MyTag::AlbumTitle => MyTagDataType::Text,
            MyTag::AlbumArtist => MyTagDataType::Text,
            MyTag::Genre => MyTagDataType::Text,
            MyTag::Composer => MyTagDataType::Text,
            MyTag::Year => MyTagDataType::Date,
            MyTag::TrackNumber => MyTagDataType::Number,
            MyTag::TrackTotal => MyTagDataType::Number,
            MyTag::DiscNumber => MyTagDataType::Number,
            MyTag::DiscTotal => MyTagDataType::Number,
            MyTag::Comment => MyTagDataType::Text,
        }
    }
}

lazy_static! {
    static ref ALL_TAGS: Vec<MyTag> = vec![MyTag::Title,
                                  MyTag::Artist,
                                  MyTag::AlbumTitle,
                                  MyTag::AlbumArtist,
                                  MyTag::Genre,
                                  MyTag::Composer,
                                  MyTag::Year,
                                  MyTag::TrackNumber,
                                  MyTag::TrackTotal,
                                  MyTag::DiscNumber,
                                  MyTag::DiscTotal,
                                  MyTag::Comment];

    static ref TEXT_TAGS: Vec<MyTag> = ALL_TAGS.iter()
                                .filter(|e| e.is_text())
                                .copied()
                                .collect::<Vec<MyTag>>();

    static ref NUMBER_TAGS: Vec<MyTag> = ALL_TAGS.iter()
                                .filter(|e| e.is_number())
                                .copied()
                                .collect::<Vec<MyTag>>();

    static ref DATE_TAGS: Vec<MyTag> = ALL_TAGS.iter()
                                .filter(|e| e.is_date())
                                .copied()
                                .collect::<Vec<MyTag>>();
}

pub trait Action {
    fn op_name(&self) -> &'static str;
    fn do_it(&self) -> Result<(), Error>;
}

pub fn entry() -> Result<(), Error> {
    let app = App::parse();
    match app.command {
        Command::View(args) => {
            println!("args: {:?}", args);

            let action = ViewAction::new(&args.directory,
                                         &args.global_opts.tags,
                                         args.with_properties);
            action.do_it()
        }
        Command::ConvZh(mut args) => {
            println!("args: {:?}", args);

            let open_cc = init_open_cc(&args.profile);
            let action = ConvZhAction::new(&open_cc,
                                           &args.directory,
                                           &mut args.global_opts.tags);
            action.do_it()
        }
        Command::ConvEn(mut args) => {
            println!("args: {:?}", args);

            let action = ConvEnAction::new(&args.directory,
                                           &mut args.global_opts.tags,
                                           args.profile);
            action.do_it()
        }
    }
}

const EMPTY_STR: &str = "";

fn to_string_or_empty(x: Option<impl ToString>) -> String {
    x.map_or(EMPTY_STR.to_string(), |v| { v.to_string() })
}

fn init_open_cc(profile: &ConvZhProfile) -> OpenCC {
    let default_config: DefaultConfig = to_config(profile);
    let temporary_path = env::temp_dir();
    generate_static_dictionary(&temporary_path, default_config).unwrap();

    OpenCC::new(temporary_path.join(default_config)).unwrap()
}

fn to_config(profile: &ConvZhProfile) -> DefaultConfig {
    match profile {
        ConvZhProfile::HK2S => DefaultConfig::HK2S,
        ConvZhProfile::HK2T => DefaultConfig::HK2T,
        ConvZhProfile::JP2T => DefaultConfig::JP2T,
        ConvZhProfile::S2T => DefaultConfig::S2T,
        ConvZhProfile::S2TW => DefaultConfig::S2TW,
        ConvZhProfile::S2TWP => DefaultConfig::S2TWP,
        ConvZhProfile::T2HK => DefaultConfig::T2HK,
        ConvZhProfile::T2JP => DefaultConfig::T2JP,
        ConvZhProfile::T2TW => DefaultConfig::T2TW,
        ConvZhProfile::T2S => DefaultConfig::T2S,
        ConvZhProfile::S2HK => DefaultConfig::S2HK,
        ConvZhProfile::TW2S => DefaultConfig::TW2S,
        ConvZhProfile::TW2SP => DefaultConfig::TW2SP,
        ConvZhProfile::TW2T => DefaultConfig::TW2T,
    }
}
