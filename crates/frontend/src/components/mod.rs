pub mod navbar;
pub mod notes;
pub mod popup;
pub mod reader;

pub use navbar::NavbarModule;
pub use notes::Notes;
pub use popup::{Popup, PopupType, edit_metadata::PopupEditMetadata, search_book::PopupSearchBook};
pub use reader::Reader;