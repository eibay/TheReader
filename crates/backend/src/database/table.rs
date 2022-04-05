use books_common::{Progression, MetadataItemCached, DisplayMetaItem, MediaItem};
use chrono::{DateTime, TimeZone, Utc};
use rusqlite::Row;
use serde::{Serialize, Serializer};


// Metadata

// TODO: Place into common
#[derive(Debug, Clone, Serialize)]
pub struct MetadataItem {
	pub id: i64,

	pub library_id: i64,

	pub source: String,
	pub file_item_count: i64,
	pub title: Option<String>,
	pub original_title: Option<String>,
	pub description: Option<String>,
	pub rating: f64,
	pub thumb_path: Option<String>,

	// TODO: Make table for all tags. Include publisher in it. Remove country.
	pub cached: MetadataItemCached,

	#[serde(serialize_with = "serialize_datetime")]
	pub refreshed_at: DateTime<Utc>,
	#[serde(serialize_with = "serialize_datetime")]
	pub created_at: DateTime<Utc>,
	#[serde(serialize_with = "serialize_datetime")]
	pub updated_at: DateTime<Utc>,
	#[serde(serialize_with = "serialize_datetime_opt")]
	pub deleted_at: Option<DateTime<Utc>>,

	pub available_at: Option<i64>,
	pub year: Option<i64>,

	pub hash: String
}

impl Default for MetadataItem {
	fn default() -> Self {
		Self {
			id: Default::default(),
			library_id: Default::default(),
			source: Default::default(),
			file_item_count: Default::default(),
			title: Default::default(),
			original_title: Default::default(),
			description: Default::default(),
			rating: Default::default(),
			thumb_path: Default::default(),
			cached: Default::default(),
			refreshed_at: Utc::now(),
			created_at: Utc::now(),
			updated_at: Utc::now(),
			deleted_at: Default::default(),
			available_at: Default::default(),
			year: Default::default(),
			hash: Default::default()
		}
	}
}

impl<'a> TryFrom<&Row<'a>> for MetadataItem {
	type Error = rusqlite::Error;

	fn try_from(value: &Row<'a>) -> std::result::Result<Self, Self::Error> {
		Ok(Self {
			id: value.get(0)?,
			library_id: value.get(1)?,
			source: value.get(2)?,
			file_item_count: value.get(3)?,
			title: value.get(4)?,
			original_title: value.get(5)?,
			description: value.get(6)?,
			rating: value.get(7)?,
			thumb_path: value.get(8)?,
			cached: value.get::<_, Option<String>>(9)?
				.map(|v| MetadataItemCached::from_string(&v))
				.unwrap_or_default(),
			available_at: value.get(10)?,
			year: value.get(11)?,
			refreshed_at: Utc.timestamp_millis(value.get(12)?),
			created_at: Utc.timestamp_millis(value.get(13)?),
			updated_at: Utc.timestamp_millis(value.get(14)?),
			deleted_at: value.get::<_, Option<_>>(15)?.map(|v| Utc.timestamp_millis(v)),
			hash: value.get(16)?
		})
	}
}

impl From<MetadataItem> for DisplayMetaItem {
	fn from(val: MetadataItem) -> Self {
		DisplayMetaItem {
			id: val.id,
			library_id: val.library_id,
			source: val.source,
			file_item_count: val.file_item_count,
			title: val.title,
			original_title: val.original_title,
			description: val.description,
			rating: val.rating,
			thumb_path: val.thumb_path,
			cached: val.cached,
			refreshed_at: val.refreshed_at,
			created_at: val.created_at,
			updated_at: val.updated_at,
			deleted_at: val.deleted_at,
			available_at: val.available_at,
			year: val.year,
			hash: val.hash,
		}
	}
}


// Tag Person Alt

#[derive(Debug, Serialize)]
pub struct MetadataPerson {
	pub metadata_id: i64,
	pub person_id: i64,
}

impl<'a> TryFrom<&Row<'a>> for MetadataPerson {
	type Error = rusqlite::Error;

	fn try_from(value: &Row<'a>) -> std::result::Result<Self, Self::Error> {
		Ok(Self {
			metadata_id: value.get(0)?,
			person_id: value.get(1)?,
		})
	}
}


// Notes

#[derive(Debug, Serialize)]
pub struct FileNote {
	pub file_id: i64,
	pub user_id: i64,

	pub data: String,
	pub data_size: i64,

	#[serde(serialize_with = "serialize_datetime")]
	pub updated_at: DateTime<Utc>,
	#[serde(serialize_with = "serialize_datetime")]
	pub created_at: DateTime<Utc>,
}

impl FileNote {
	pub fn new(file_id: i64, user_id: i64, data: String) -> Self {
		Self {
			file_id,
			user_id,
			data_size: data.len() as i64,
			data,
			updated_at: Utc::now(),
			created_at: Utc::now(),
		}
	}
}


impl<'a> TryFrom<&Row<'a>> for FileNote {
	type Error = rusqlite::Error;

	fn try_from(value: &Row<'a>) -> std::result::Result<Self, Self::Error> {
		Ok(Self {
			file_id: value.get(0)?,
			user_id: value.get(1)?,

			data: value.get(2)?,

			data_size: value.get(3)?,

			updated_at: Utc.timestamp_millis(value.get(4)?),
			created_at: Utc.timestamp_millis(value.get(5)?),
		})
	}
}

// File Progression

#[derive(Debug, Serialize)]
pub struct FileProgression {
	pub file_id: i64,
	pub user_id: i64,

	pub type_of: u8,

	// Ebook/Audiobook
	pub chapter: Option<i64>,

	// Ebook
	pub page: Option<i64>, // TODO: Remove page. Change to byte pos. Most accurate since screen sizes can change.
	pub char_pos: Option<i64>,

	// Audiobook
	pub seek_pos: Option<i64>,

	#[serde(serialize_with = "serialize_datetime")]
	pub updated_at: DateTime<Utc>,
	#[serde(serialize_with = "serialize_datetime")]
	pub created_at: DateTime<Utc>,
}

impl FileProgression {
	pub fn new(progress: Progression, user_id: i64, file_id: i64) -> Self {
		match progress {
			Progression::Complete => Self {
				file_id,
				user_id,
				type_of: 0,
				chapter: None,
				page: None,
				char_pos: None,
				seek_pos: None,
				updated_at: Utc::now(),
				created_at: Utc::now(),
			},

			Progression::Ebook { chapter, page, char_pos } => Self {
				file_id,
				user_id,
				type_of: 1,
				char_pos: Some(char_pos),
				chapter: Some(chapter),
				page: Some(page),
				seek_pos: None,
				updated_at: Utc::now(),
				created_at: Utc::now(),
			},

			Progression::AudioBook { chapter, seek_pos } => Self {
				file_id,
				user_id,
				type_of: 2,
				chapter: Some(chapter),
				page: None,
				char_pos: None,
				seek_pos: Some(seek_pos),
				updated_at: Utc::now(),
				created_at: Utc::now(),
			}
		}
	}
}

impl<'a> TryFrom<&Row<'a>> for FileProgression {
	type Error = rusqlite::Error;

	fn try_from(value: &Row<'a>) -> std::result::Result<Self, Self::Error> {
		Ok(Self {
			file_id: value.get(0)?,
			user_id: value.get(1)?,

			type_of: value.get(2)?,

			chapter: value.get(3)?,

			page: value.get(4)?,
			char_pos: value.get(5)?,

			seek_pos: value.get(6)?,

			updated_at: Utc.timestamp_millis(value.get(7)?),
			created_at: Utc.timestamp_millis(value.get(8)?),
		})
	}
}

impl From<FileProgression> for Progression {
    fn from(val: FileProgression) -> Self {
        match val.type_of {
			0 => Progression::Complete,

			1 => Progression::Ebook {
				char_pos: val.char_pos.unwrap(),
				chapter: val.chapter.unwrap(),
				page: val.page.unwrap(),
			},

			2 => Progression::AudioBook {
				chapter: val.chapter.unwrap(),
				seek_pos: val.seek_pos.unwrap(),
			},

			_ => unreachable!()
		}
    }
}


// Library

pub struct NewLibrary {
	pub name: String,
	pub type_of: String,

	pub scanned_at: DateTime<Utc>,
	pub created_at: DateTime<Utc>,
	pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct Library {
	pub id: i64,

	pub name: String,
	pub type_of: String,

	#[serde(serialize_with = "serialize_datetime")]
	pub scanned_at: DateTime<Utc>,
	#[serde(serialize_with = "serialize_datetime")]
	pub created_at: DateTime<Utc>,
	#[serde(serialize_with = "serialize_datetime")]
	pub updated_at: DateTime<Utc>,
}

impl<'a> TryFrom<&Row<'a>> for Library {
	type Error = rusqlite::Error;

	fn try_from(value: &Row<'a>) -> std::result::Result<Self, Self::Error> {
		Ok(Self {
			id: value.get(0)?,
			name: value.get(1)?,
			type_of: value.get(2)?,
			scanned_at: Utc.timestamp_millis(value.get(3)?),
			created_at: Utc.timestamp_millis(value.get(4)?),
			updated_at: Utc.timestamp_millis(value.get(5)?),
		})
	}
}


// Directory

pub struct Directory {
	pub library_id: i64,
	pub path: String,
}

impl<'a> TryFrom<&Row<'a>> for Directory {
	type Error = rusqlite::Error;

	fn try_from(value: &Row<'a>) -> std::result::Result<Self, Self::Error> {
		Ok(Self {
			library_id: value.get(0)?,
			path: value.get(1)?,
		})
	}
}


// File

pub struct NewFile {
	pub path: String,

	pub file_name: String,
	pub file_type: String,
	pub file_size: i64,

	pub library_id: i64,
	pub metadata_id: Option<i64>,
	pub chapter_count: i64,

	pub modified_at: DateTime<Utc>,
	pub accessed_at: DateTime<Utc>,
	pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct File {
	pub id: i64,

	pub path: String,

	pub file_name: String,
	pub file_type: String,
	pub file_size: i64,

	pub library_id: i64,
	pub metadata_id: Option<i64>,
	pub chapter_count: i64,

	#[serde(serialize_with = "serialize_datetime")]
	pub modified_at: DateTime<Utc>,
	#[serde(serialize_with = "serialize_datetime")]
	pub accessed_at: DateTime<Utc>,
	#[serde(serialize_with = "serialize_datetime")]
	pub created_at: DateTime<Utc>,
}

impl<'a> TryFrom<&Row<'a>> for File {
	type Error = rusqlite::Error;

	fn try_from(value: &Row<'a>) -> std::result::Result<Self, Self::Error> {
		Ok(Self {
			id: value.get(0)?,

			path: value.get(1)?,

			file_name: value.get(2)?,
			file_type: value.get(3)?,
			file_size: value.get(4)?,

			library_id: value.get(5)?,
			metadata_id: value.get(6)?,
			chapter_count: value.get(7)?,

			modified_at: Utc.timestamp_millis(value.get(8)?),
			accessed_at: Utc.timestamp_millis(value.get(9)?),
			created_at: Utc.timestamp_millis(value.get(10)?),
		})
	}
}

impl From<File> for MediaItem {
    fn from(file: File) -> Self {
        Self {
            id: file.id,

			path: file.path,

            file_name: file.file_name,
            file_type: file.file_type,
            file_size: file.file_size,

			library_id: file.library_id,
			metadata_id: file.metadata_id,
			chapter_count: file.chapter_count as usize,

            modified_at: file.modified_at.timestamp_millis(),
            accessed_at: file.accessed_at.timestamp_millis(),
            created_at: file.created_at.timestamp_millis(),
        }
    }
}


// Tags People

#[derive(Debug)]
pub struct NewTagPerson {
	pub source: String,

	pub name: String,
	pub description: Option<String>,
	pub birth_date: Option<String>,

	pub updated_at: DateTime<Utc>,
	pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct TagPerson {
	pub id: i64,

	pub source: String,

	pub name: String,
	pub description: Option<String>,
	pub birth_date: Option<String>,

	#[serde(serialize_with = "serialize_datetime")]
	pub updated_at: DateTime<Utc>,
	#[serde(serialize_with = "serialize_datetime")]
	pub created_at: DateTime<Utc>,
}

impl<'a> TryFrom<&Row<'a>> for TagPerson {
	type Error = rusqlite::Error;

	fn try_from(value: &Row<'a>) -> std::result::Result<Self, Self::Error> {
		Ok(Self {
			id: value.get(0)?,

			source: value.get(1)?,

			name: value.get(2)?,
			description: value.get(3)?,
			birth_date: value.get(4)?,

			created_at: Utc.timestamp_millis(value.get(5)?),
			updated_at: Utc.timestamp_millis(value.get(6)?),
		})
	}
}


// Tag Person Alt

#[derive(Debug, Serialize)]
pub struct TagPersonAlt {
	pub person_id: i64,
	pub name: String,
}

impl<'a> TryFrom<&Row<'a>> for TagPersonAlt {
	type Error = rusqlite::Error;

	fn try_from(value: &Row<'a>) -> std::result::Result<Self, Self::Error> {
		Ok(Self {
			person_id: value.get(0)?,
			name: value.get(1)?,
		})
	}
}






// Utils

fn serialize_datetime<S>(value: &DateTime<Utc>, s: S) -> std::result::Result<S::Ok, S::Error> where S: Serializer {
	s.serialize_i64(value.timestamp_millis())
}

fn serialize_datetime_opt<S>(value: &Option<DateTime<Utc>>, s: S) -> std::result::Result<S::Ok, S::Error> where S: Serializer {
	match value {
		Some(v) => s.serialize_i64(v.timestamp_millis()),
		None => s.serialize_none()
	}
}



// Non Table Items

pub struct FileWithMetadata {
	pub file: File,
	pub meta: Option<MetadataItem>
}

impl<'a> TryFrom<&Row<'a>> for FileWithMetadata {
	type Error = rusqlite::Error;

	fn try_from(value: &Row<'a>) -> std::result::Result<Self, Self::Error> {
		Ok(Self {
			file: File {
				id: value.get(0)?,

				path: value.get(1)?,

				file_name: value.get(2)?,
				file_type: value.get(3)?,
				file_size: value.get(4)?,

				library_id: value.get(5)?,
				metadata_id: value.get(6)?,
				chapter_count: value.get(7)?,

				modified_at: Utc.timestamp_millis(value.get(8)?),
				accessed_at: Utc.timestamp_millis(value.get(9)?),
				created_at: Utc.timestamp_millis(value.get(10)?),
			},

			meta: value.get(11)
				.ok()
				.map(|_: i64| std::result::Result::<_, Self::Error>::Ok(MetadataItem {
					id: value.get(11)?,
					library_id: value.get(12)?,
					source: value.get(13)?,
					file_item_count: value.get(14)?,
					title: value.get(15)?,
					original_title: value.get(16)?,
					description: value.get(17)?,
					rating: value.get(18)?,
					thumb_path: value.get(19)?,
					cached: value.get::<_, Option<String>>(20)?
						.map(|v| MetadataItemCached::from_string(&v))
						.unwrap_or_default(),
					available_at: value.get(21)?,
					year: value.get(22)?,
					refreshed_at: Utc.timestamp_millis(value.get(23)?),
					created_at: Utc.timestamp_millis(value.get(24)?),
					updated_at: Utc.timestamp_millis(value.get(25)?),
					deleted_at: value.get::<_, Option<_>>(26)?.map(|v| Utc.timestamp_millis(v)),
					hash: value.get(27)?
				}))
				.transpose()?
		})
	}
}