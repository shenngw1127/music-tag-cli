use clap::ValueEnum;
use enum_display::EnumDisplay;
use lazy_static::lazy_static;

enum MyTagType {
    // Text tag
    Text,

    // Numeric tag
    Numeric,

    // Date tag
    Date,
}

#[derive(Debug, Copy, Clone, ValueEnum, Eq, Hash, PartialEq, EnumDisplay)]
#[enum_display(case = "Kebab")]
pub enum MyTag {
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
    Date,
    Comment,
    Copyright,
}

impl MyTag {
    pub fn is_text(&self) -> bool {
        match self.data_type() {
            MyTagType::Text => true,
            _ => false,
        }
    }

    pub fn is_numeric(&self) -> bool {
        match self.data_type() {
            MyTagType::Numeric => true,
            _ => false,
        }
    }

    pub fn is_date(&self) -> bool {
        match self.data_type() {
            MyTagType::Date => true,
            _ => false,
        }
    }

    fn data_type(&self) -> MyTagType {
        match self {
            MyTag::Title => MyTagType::Text,
            MyTag::Artist => MyTagType::Text,
            MyTag::AlbumTitle => MyTagType::Text,
            MyTag::AlbumArtist => MyTagType::Text,
            MyTag::Genre => MyTagType::Text,
            MyTag::Composer => MyTagType::Text,
            MyTag::Year => MyTagType::Numeric,
            MyTag::TrackNumber => MyTagType::Numeric,
            MyTag::TrackTotal => MyTagType::Numeric,
            MyTag::DiscNumber => MyTagType::Numeric,
            MyTag::DiscTotal => MyTagType::Numeric,
            MyTag::Date => MyTagType::Date,
            MyTag::Comment => MyTagType::Text,
            MyTag::Copyright => MyTagType::Text,
        }
    }
}

lazy_static! {
    pub static ref ALL_TAGS: Vec<MyTag> = vec![MyTag::Title,
        MyTag::Artist,
        MyTag::AlbumTitle,
        MyTag::AlbumArtist,
        MyTag::Genre,
        MyTag::Composer,
        MyTag::Year,
        MyTag::Date,
        MyTag::TrackNumber,
        MyTag::TrackTotal,
        MyTag::DiscNumber,
        MyTag::DiscTotal,
        MyTag::Comment,
        MyTag::Copyright];

    pub static ref TEXT_TAGS: Vec<MyTag> = ALL_TAGS.iter()
                                .filter(|e| e.is_text())
                                .copied()
                                .collect::<Vec<MyTag>>();

    static ref NUMERIC_TAGS: Vec<MyTag> = ALL_TAGS.iter()
                                .filter(|e| e.is_numeric())
                                .copied()
                                .collect::<Vec<MyTag>>();
    static ref DATE_TAGS: Vec<MyTag> = ALL_TAGS.iter()
                                .filter(|e| e.is_date())
                                .copied()
                                .collect::<Vec<MyTag>>();
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum ConvEnProfile {
    Lowercase,
    Uppercase,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub(crate) enum CalcMethod {
    Increase,
    Decrease,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum SetWhen {
    Always,
    OnlyEmpty,
    OnlyNotEmpty,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub enum ModifyMode {
    Overwrite,
    Insert,
    Append,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub(crate) enum AddDirection {
    InsertFromBeginning,
    AppendFromEnd,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub(crate) enum Direction {
    Beginning,
    End,
}

#[derive(Clone, Debug, ValueEnum, PartialEq)]
pub(crate) enum QueryResultPosition {
    Any,
    First,
    Last,
    Beginning,
    End,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum ConvZhProfile {
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
