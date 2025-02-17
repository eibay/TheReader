use serde::{Serialize, Deserialize};

use crate::reader::ReaderColor;

// TODO: I don't want to store it like this but it's easiest way.

#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemberPreferences {
    pub version: usize,

    // TODO: May want to separate preferences.
    pub desktop: MemberBasicPreferences,
    pub mobile: MemberBasicPreferences,
}


#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemberBasicPreferences {
    pub reader: MemberReaderPreferences,
}

// Reader

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemberReaderPreferences {
    pub always_show_progress: bool,

    pub auto_full_screen: bool,
    pub default_full_screen: bool,

    pub width: u32,
    pub height: u32,

    pub text_size: u32,
    pub color: ReaderColor,

    pub display_type: u8,
    pub load_type: u8,
}

impl Default for MemberReaderPreferences {
    fn default() -> Self {
        Self {
            always_show_progress: false,
            auto_full_screen: false,
            default_full_screen: false,
            width: 1040,
            height: 548,
            text_size: 0,
            color: ReaderColor::default(),
            // SectionDisplay::Double
            display_type: 1,
            // PageLoadType::Select
            load_type: 1,
        }
    }
}