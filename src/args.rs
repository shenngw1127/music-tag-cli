use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

use crate::model::{AddDirection, CalcMethod, ConstValue, ConvEnProfile, ConvZhProfile, Direction, ModifyMode, MyTag, QueryResultPosition, SetWhen, TextConst};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct App {
    #[clap(subcommand)]
    pub(crate) command: Command,
}

#[derive(Debug, Subcommand)]
#[command(author, version, about, long_about = None)]
pub enum Command {
    View(ViewArgs),
    ConvEn(ConvEnArgs),
    ConvUtf8(ConvUtf8Args),
    ConvZh(ConvZhArgs),
    Exp(ExpArgs),
    Imp(ImpArgs),
    ModNum(ModNumArgs),
    ModTextConst(ModTexConstArgs),
    ModTextRegex(ModTextRegexArgs),
    SetConst(SetConstArgs),
    SetName(SetNameArgs),
    SetSeq(SetSeqArgs),
    Ren(RenArgs),
}

#[derive(Debug, Args)]
#[command(arg_required_else_help = true, long_about = "Export tags to file.")]
pub struct ExpArgs {
    #[arg(long, default_value_t = false)]
    #[arg(help = "Export properties or NOT (default).")]
    pub with_properties: bool,

    #[arg(short, long, value_delimiter = ',')]
    #[arg(help = "Process specified tags, if not set, it will process ALL tags.")]
    pub tags: Vec<MyTag>,

    #[arg(long = "where")]
    #[arg(help = "`Where` clause for prediction. It is like SQL, supported `NOT` `AND` `OR` \
    logic operators, `=` `<` `<=` `>` `>=` `!=` `<>` comparison operators, `LIKE` also is \
    supported with `%` `_` wildcards, `ILIKE` is same but case insensitive. \
    Note: `'` should be escaped as `''` like in SQL string.")]
    pub where_clause: Option<String>,

    #[arg(short, long)]
    #[arg(value_hint = clap::ValueHint::FilePath)]
    #[arg(help = "Output file, it must be NOT exists!")]
    pub output_file: PathBuf,

    #[arg(value_hint = clap::ValueHint::FilePath)]
    #[arg(help = "The path of your music file(s). It must point to a file or directory path.")]
    pub directory: PathBuf,
}

#[derive(Debug, Args)]
#[command(arg_required_else_help = true, long_about = "View tags.")]
pub struct ViewArgs {
    #[arg(long, default_value_t = false)]
    #[arg(help = "Show properties or NOT (default).")]
    pub with_properties: bool,

    #[arg(short, long, value_delimiter = ',')]
    #[arg(help = "Process specified tags, if not set, it will process ALL tags.")]
    pub tags: Vec<MyTag>,

    #[arg(long = "where")]
    #[arg(help = "`Where` clause for prediction. It is like SQL, supported `NOT` `AND` `OR` \
    logic operators, `=` `<` `<=` `>` `>=` `!=` `<>` comparison operators, `LIKE` also is \
    supported with `%` `_` wildcards, `ILIKE` is same but case insensitive. \
    Note: `'` should be escaped as `''` like in SQL string.")]
    pub where_clause: Option<String>,

    #[arg(value_hint = clap::ValueHint::FilePath)]
    #[arg(help = "The path of your music file(s). It must point to a file or directory path.")]
    pub directory: PathBuf,
}

#[derive(Debug, Args)]
#[command(arg_required_else_help = true)]
#[command(long_about = "Set tags from filename.")]
pub struct SetNameArgs {
    #[arg(short = 'w', long, value_enum, default_value_t = SetWhen::Always)]
    #[arg(help = "When to set the tag.")]
    pub set_when: SetWhen,

    #[arg(long)]
    #[arg(help = "Template for parsing filename like \"${track-number} - ${title} - ${artist}\".")]
    pub template: String,

    #[arg(long = "where")]
    #[arg(help = "`Where` clause for prediction. It is like SQL, supported `NOT` `AND` `OR` \
    logic operators, `=` `<` `<=` `>` `>=` `!=` `<>` comparison operators, `LIKE` also is \
    supported with `%` `_` wildcards, `ILIKE` is same but case insensitive. \
    Note: `'` should be escaped as `''` like in SQL string.")]
    pub where_clause: Option<String>,

    #[arg(long, default_value_t = false)]
    #[arg(help = "Only show how to modify tags, but do NOT write any file, if it was set as true.")]
    pub dry_run: bool,

    #[arg(short, long, default_value_t = false)]
    #[arg(help = "Only show error in console, if it was set as true.")]
    pub quiet: bool,

    #[arg(value_hint = clap::ValueHint::FilePath)]
    #[arg(help = "The path of your music file(s). It must point to a file or directory path.")]
    pub directory: PathBuf,
}

#[derive(Debug, Args)]
#[command(arg_required_else_help = true, long_about = "Set a constant value for tags.")]
pub struct SetConstArgs {
    #[command(subcommand)]
    pub value: ConstValueArgs,

    #[arg(short = 'w', long, value_enum, default_value_t = SetWhen::Always)]
    #[arg(help = "When to set the tag.")]
    pub set_when: SetWhen,

    #[arg(short, long, value_enum, default_value_t = ModifyMode::Overwrite)]
    #[arg(help = "How to modify the tag if tag has already exist, only worked for TEXT tags.")]
    pub modify_mode: ModifyMode,

    #[arg(long = "where")]
    #[arg(help = "`Where` clause for prediction. It is like SQL, supported `NOT` `AND` `OR` \
    logic operators, `=` `<` `<=` `>` `>=` `!=` `<>` comparison operators, `LIKE` also is \
    supported with `%` `_` wildcards, `ILIKE` is same but case insensitive. \
    Note: `'` should be escaped as `''` like in SQL string.")]
    pub where_clause: Option<String>,

    #[clap(flatten)]
    pub global_opts: GlobalAllTagsDefaultEmpty,
}

#[derive(Debug, Args)]
#[command(arg_required_else_help = true, long_about = "Set sequence value for tags.")]
pub struct SetSeqArgs {
    #[clap(flatten)]
    pub value: Sequence,

    #[arg(short, long, value_enum, default_value_t = ModifyMode::Overwrite)]
    #[arg(help = "How to modify the tag if tag has already exist.")]
    pub modify_mode: ModifyMode,

    #[arg(short = 'y', long, value_enum, default_value = "-")]
    #[arg(help = "If \"--modify-mode\" was set \"append\" or \"insert\", \
    some characters could be filled between the stem.")]
    pub hyphen: String,

    #[clap(flatten)]
    pub global_opts: GlobalAllTagsDefaultEmpty,
}

#[derive(Debug, Args)]
pub struct Sequence {
    #[arg(short = 's', long, default_value_t = 1)]
    #[arg(help = "Sequence number start with.")]
    pub start: u32,

    #[arg(short = 'e', long, default_value_t = 1)]
    #[arg(help = "Sequence number step.")]
    pub step: u32,

    #[arg(short = 'd', long, default_value_t = 2)]
    #[arg(help = "Sequence number format padding.")]
    pub padding: usize,
}

#[derive(Debug, Subcommand)]
pub enum ConstValueArgs {
    #[command(arg_required_else_help = true)]
    #[command(long_about = "Set a text constant, only was applied to TEXt tags.")]
    Text {
        #[arg(help = "Text value.")]
        value: String,
    },
    #[command(long_about = "Set a numeric constant, only was applied to NUMERIC tags.")]
    Num {
        #[arg(help = "Numeric value.")]
        value: u32,

        #[arg(short = 'd', long, default_value_t = 2)]
        #[arg(help = "Numeric format padding.")]
        padding: usize,
    },
    #[command(long_about = "Set a date constant, only was applied to DATE tags.")]
    Date {
        #[arg(help = "Date value.")]
        value: String,

        #[arg(short, long, default_value = "%Y-%m-%d")]
        #[arg(help = "Date format, composited by \
        \"%Y\" ~ year, \"%m\" ~ month, \"%d\" ~ day of month. \
        It must be represent a complete date.")]
        format: String,
    },
}

impl From<ConstValueArgs> for ConstValue {
    fn from(value: ConstValueArgs) -> Self {
        match value {
            ConstValueArgs::Text { value } =>
                ConstValue::Text { value },
            ConstValueArgs::Num { value, padding } =>
                ConstValue::Num { value, padding },
            ConstValueArgs::Date { value, format } =>
                ConstValue::Date { value, format }
        }
    }
}

#[derive(Debug, Subcommand)]
pub enum TextConstArgs {
    #[command(arg_required_else_help = true)]
    #[command(long_about = "Add a text constant.")]
    Add {
        #[arg(short = 'd', long, value_enum, default_value_t = AddDirection::InsertFromBeginning)]
        #[arg(help = "Direction")]
        add_direction: AddDirection,

        #[arg(short = 'o', long, default_value_t = 0)]
        #[arg(help = "Position offset, start from 0.")]
        offset: usize,

        #[arg(short, long)]
        #[arg(help = "Addend text.")]
        addend: String,
    },
    #[command(arg_required_else_help = true)]
    #[command(long_about = "Replace by a text constant.")]
    Replace {
        #[arg(long)]
        #[arg(help = "Query from")]
        from: String,

        #[arg(short, long, default_value_t = false)]
        #[arg(help = "Query match will be case insensitive, if it was set as true.")]
        ignore_case: bool,

        #[arg(short, long, value_enum, default_value_t = QueryResultPosition::Any)]
        #[arg(help = "Query result position to be replaced.")]
        position: QueryResultPosition,

        #[arg(long)]
        #[arg(help = "Replace to a text value.")]
        to: String,
    },
    #[command(arg_required_else_help = true)]
    #[command(long_about = "Remove a text constant.")]
    Remove {
        #[arg(short, long, value_enum, default_value_t = Direction::Beginning)]
        #[arg(help = "Direction")]
        direction: Direction,

        #[arg(short, long)]
        #[arg(help = "Beginning offset, first is 0.")]
        beginning_offset: usize,

        #[arg(short, long)]
        #[arg(help = "End offset, first is 0. \
        If not set it, command will remove all characters from beginning offset to the end \
        / beginning (according the direction).")]
        end_offset: Option<usize>,
    },
    #[command(arg_required_else_help = true)]
    #[command(long_about = "Truncate a text tag.")]
    Truncate {
        #[arg(short, long, value_enum, default_value_t = Direction::Beginning)]
        #[arg(help = "Direction")]
        direction: Direction,

        #[arg(short, long)]
        #[arg(help = "Max character count.")]
        limit: usize,
    },
}

impl From<TextConstArgs> for TextConst {
    fn from(value: TextConstArgs) -> Self {
        match value {
            TextConstArgs::Add {
                add_direction, offset, addend
            } => TextConst::Add { add_direction, offset, addend },
            TextConstArgs::Replace {
                from, ignore_case, position, to
            } => TextConst::Replace { from, ignore_case, position, to },
            TextConstArgs::Remove {
                direction, beginning_offset, end_offset
            } => TextConst::Remove { direction, beginning_offset, end_offset },
            TextConstArgs::Truncate {
                direction, limit
            } => TextConst::Truncate { direction, limit }
        }
    }
}

#[derive(Debug, Args)]
#[command(arg_required_else_help = true, long_about = "Import tags from file.")]
pub struct ImpArgs {
    #[arg(long, default_value_t = false)]
    #[arg(help = "Only show how to modify tags, but do NOT write any file, if it was set as true.")]
    pub dry_run: bool,

    #[arg(short, long, default_value_t = false)]
    #[arg(help = "Only show error in console, if it was set as true.")]
    pub quiet: bool,

    #[arg(short, long)]
    #[arg(value_hint = clap::ValueHint::FilePath)]
    #[arg(help = "The base path for your JSON content `path` attribute, if it is a relative path.")]
    pub base_directory: Option<PathBuf>,

    #[arg(value_hint = clap::ValueHint::FilePath)]
    #[arg(help = "The source file you want to import.")]
    pub source_file: PathBuf,
}

#[derive(Debug, Args)]
#[command(arg_required_else_help = true, long_about = "Rename file with tags.")]
pub struct RenArgs {
    #[arg(long, default_value_t = false)]
    #[arg(help = "Only show how to modify tags, but do NOT write any file, if it was set as true.")]
    pub dry_run: bool,

    #[arg(short, long, default_value_t = false)]
    #[arg(help = "Only show error in console, if it was set as true.")]
    pub quiet: bool,

    #[arg(long = "where")]
    #[arg(help = "`Where` clause for prediction. It is like SQL, supported `NOT` `AND` `OR` \
    logic operators, `=` `<` `<=` `>` `>=` `!=` `<>` comparison operators, `LIKE` also is \
    supported with `%` `_` wildcards, `ILIKE` is same but case insensitive. \
    Note: `'` should be escaped as `''` like in SQL string.")]
    pub where_clause: Option<String>,

    #[arg(long)]
    #[arg(help = "Template for new filename like \"${track-number} - ${title} - ${artist}\".")]
    pub template: String,

    #[arg(value_hint = clap::ValueHint::FilePath)]
    #[arg(help = "The path of your music file(s). It must point to a file or directory path.")]
    pub directory: PathBuf,
}

#[derive(Debug, Args)]
#[command(arg_required_else_help = true)]
#[command(long_about = "Modify numeric tags by increase/decrease an integer.")]
pub struct ModNumArgs {
    #[arg(short, long, value_enum, default_value_t = CalcMethod::Increase)]
    #[arg(help = "Calculate method.")]
    pub calc_method: CalcMethod,

    #[arg(short, long)]
    #[arg(help = "Operand, an integer must great than 0.")]
    pub operand: u32,

    #[arg(short = 'd', long, default_value_t = 2)]
    #[arg(help = "Numeric format padding.")]
    pub padding: usize,

    #[arg(short, long, value_delimiter = ',')]
    #[arg(help = "Process specified NUMERIC tags, if not set, it will process ALL NUMERIC tags.")]
    pub tags: Vec<NumericTagArgs>,

    #[arg(long = "where")]
    #[arg(help = "`Where` clause for prediction. It is like SQL, supported `NOT` `AND` `OR` \
    logic operators, `=` `<` `<=` `>` `>=` `!=` `<>` comparison operators, `LIKE` also is \
    supported with `%` `_` wildcards, `ILIKE` is same but case insensitive. \
    Note: `'` should be escaped as `''` like in SQL string.")]
    pub where_clause: Option<String>,

    #[arg(long, default_value_t = false)]
    #[arg(help = "Only show how to modify tags, but do NOT write any file, if it was set as true.")]
    pub dry_run: bool,

    #[arg(short, long, default_value_t = false)]
    #[arg(help = "Only show error in console, if it was set as true.")]
    pub quiet: bool,

    #[arg(value_hint = clap::ValueHint::FilePath)]
    #[arg(help = "The path of your music file(s). It must point to a file or directory path.")]
    pub directory: PathBuf,
}

#[derive(Debug, Args)]
#[command(arg_required_else_help = true)]
#[command(long_about = "Modify text tags by add/replace/remove a constant value,\
 also could truncate.")]
pub struct ModTexConstArgs {
    #[command(subcommand)]
    pub value: TextConstArgs,

    #[clap(flatten)]
    pub global_opts: GlobalTextTagsDefaultAll,
}

#[derive(Debug, Args)]
#[command(arg_required_else_help = true, long_about = "Modify text tags by REGEX replace.")]
pub struct ModTextRegexArgs {
    #[arg(long)]
    #[arg(help = "Query pattern. (ref: https://docs.rs/regex/latest/regex/#syntax)")]
    pub from: String,

    #[arg(short, long, default_value_t = false)]
    #[arg(help = "Query match will be case insensitive, if it was set as true.")]
    pub ignore_case: bool,

    #[arg(long)]
    #[arg(help = "Replace to. It could use $0 $1 $2 ... for captured group. \
    (Note: $1a looks up the capture group named 1a and not the capture group at index 1. \
    To exert more precise control over the name, use braces, e.g., ${1}a. \"$$\" for literal $) \
    Please read: https://docs.rs/regex/latest/regex/struct.Regex.html#method.replace")]
    pub to: String,

    #[clap(flatten)]
    pub global_opts: GlobalTextTagsDefaultAll,
}

#[derive(Parser, Debug)]
#[command(arg_required_else_help = true)]
#[command(long_about = "Convert text tags in Chinese characters between Traditional / Simplified /\
 Japanese Shinjitai.")]
pub struct ConvZhArgs {
    #[arg(short, long, value_enum)]
    #[arg(help = "Profile, often used: s2t / t2s / jp2t / t2jp. \
    (ref: https://github.com/BYVoid/OpenCC)")]
    pub profile: ConvZhProfile,

    #[clap(flatten)]
    pub global_opts: GlobalTextTagsDefaultAll,
}

#[derive(Parser, Debug)]
#[command(arg_required_else_help = true)]
#[command(long_about = "Convert text tags in English between lowercase / uppercase / tilecase.")]
pub struct ConvEnArgs {
    #[arg(short, long, value_enum)]
    #[arg(help = "Profile.")]
    pub profile: ConvEnProfile,

    #[clap(flatten)]
    pub global_opts: GlobalTextTagsDefaultAll,
}

#[derive(Parser, Debug)]
#[command(arg_required_else_help = true, long_about = "Convert text tags to UTF-8 encoding.")]
pub struct ConvUtf8Args {
    #[arg(short, long)]
    #[arg(help = "Original encoding. eg. GBK Big5 shift_jis Windows-1252 ISO-8859-15 ... \
    (ref: https://docs.rs/encoding_rs/latest/encoding_rs/)")]
    pub encoding_name: String,

    #[clap(flatten)]
    pub global_opts: GlobalTextTagsDefaultAll,
}

#[derive(Debug, Args)]
pub struct GlobalAllTagsDefaultEmpty {
    #[arg(short, long, value_delimiter = ',')]
    #[arg(help = "Process specified tags, if not set, it will NOT process any tag.")]
    pub tags: Vec<MyTag>,

    #[arg(long, default_value_t = false)]
    #[arg(help = "Only show how to modify tags, but do NOT write any file, if it was set as true.")]
    pub dry_run: bool,

    #[arg(short, long, default_value_t = false)]
    #[arg(help = "Only show error in console, if it was set as true.")]
    pub quiet: bool,

    #[arg(value_hint = clap::ValueHint::FilePath)]
    #[arg(help = "The path of your music file(s). It must point to a file or directory path.")]
    pub directory: PathBuf,
}

#[derive(Debug, Args)]
pub struct GlobalTextTagsDefaultAll {
    #[arg(short, long, value_delimiter = ',')]
    #[arg(help = "Process specified TEXT tags, if not set, it will process ALL TEXT tags.")]
    pub tags: Vec<TextTagArgs>,

    #[arg(long = "where")]
    #[arg(help = "`Where` clause for prediction. It is like SQL, supported `NOT` `AND` `OR` \
    logic operators, `=` `<` `<=` `>` `>=` `!=` `<>` comparison operators, `LIKE` also is \
    supported with `%` `_` wildcards, `ILIKE` is same but case insensitive. \
    Note: `'` should be escaped as `''` like in SQL string.")]
    pub where_clause: Option<String>,

    #[arg(long, default_value_t = false)]
    #[arg(help = "Only show how to modify tags, but do NOT write any file, if it was set as true.")]
    pub dry_run: bool,

    #[arg(short, long, default_value_t = false)]
    #[arg(help = "Only show error in console, if it was set as true.")]
    pub quiet: bool,

    #[arg(value_hint = clap::ValueHint::FilePath)]
    #[arg(help = "The path of your music file(s). It must point to a file or directory path.")]
    pub directory: PathBuf,
}

#[derive(Debug, Copy, Clone, ValueEnum, Eq, Hash, PartialEq)]
pub enum TextTagArgs {
    Title,
    Artist,
    AlbumTitle,
    AlbumArtist,
    Genre,
    Composer,
    Comment,
    Copyright,
}

impl From<TextTagArgs> for MyTag {
    fn from(value: TextTagArgs) -> Self {
        match value {
            TextTagArgs::Title => MyTag::Title,
            TextTagArgs::Artist => MyTag::Artist,
            TextTagArgs::AlbumTitle => MyTag::AlbumTitle,
            TextTagArgs::AlbumArtist => MyTag::AlbumArtist,
            TextTagArgs::Genre => MyTag::Genre,
            TextTagArgs::Composer => MyTag::Composer,
            TextTagArgs::Comment => MyTag::Comment,
            TextTagArgs::Copyright => MyTag::Copyright,
        }
    }
}

#[derive(Debug, Copy, Clone, ValueEnum, Eq, Hash, PartialEq)]
pub enum NumericTagArgs {
    Year,
    TrackNumber,
    TrackTotal,
    DiscNumber,
    DiscTotal,
}

impl From<NumericTagArgs> for MyTag {
    fn from(value: NumericTagArgs) -> Self {
        match value {
            NumericTagArgs::Year => MyTag::Year,
            NumericTagArgs::TrackNumber => MyTag::TrackNumber,
            NumericTagArgs::TrackTotal => MyTag::TrackTotal,
            NumericTagArgs::DiscNumber => MyTag::DiscNumber,
            NumericTagArgs::DiscTotal => MyTag::DiscTotal,
        }
    }
}
