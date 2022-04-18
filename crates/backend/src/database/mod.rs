use std::sync::{Mutex, MutexGuard};

use anyhow::Result;
use books_common::{Progression, Source};
use chrono::Utc;
use rusqlite::{Connection, params, OptionalExtension};
// TODO: use tokio::task::spawn_blocking;

pub mod table;
use table::*;


pub async fn init() -> Result<Database> {
	let conn = rusqlite::Connection::open("database.db")?;

	// TODO: Migrations https://github.com/rusqlite/rusqlite/discussions/1117

	// Library
	conn.execute(
		r#"CREATE TABLE IF NOT EXISTS "library" (
			"id" 				INTEGER NOT NULL UNIQUE,

			"name" 				TEXT UNIQUE,
			"type_of" 			TEXT,

			"scanned_at" 		DATETIME NOT NULL,
			"created_at" 		DATETIME NOT NULL,
			"updated_at" 		DATETIME NOT NULL,

			PRIMARY KEY("id" AUTOINCREMENT)
		);"#,
		[]
	)?;

	// Directory
	conn.execute(
		r#"CREATE TABLE IF NOT EXISTS "directory" (
			"library_id"	INTEGER NOT NULL,
			"path"			TEXT NOT NULL UNIQUE
		);"#,
		[]
	)?;

	// File
	conn.execute(
		r#"CREATE TABLE IF NOT EXISTS "file" (
			"id" 				INTEGER NOT NULL UNIQUE,

			"path" 				TEXT NOT NULL UNIQUE,
			"file_name" 		TEXT NOT NULL,
			"file_type" 		TEXT,
			"file_size" 		INTEGER NOT NULL,

			"library_id" 		INTEGER,
			"metadata_id" 		INTEGER,
			"chapter_count" 	INTEGER,

			"modified_at" 		DATETIME NOT NULL,
			"accessed_at" 		DATETIME NOT NULL,
			"created_at" 		DATETIME NOT NULL,

			PRIMARY KEY("id" AUTOINCREMENT)
		);"#,
		[]
	)?;

	// Metadata Item
	conn.execute(
		r#"CREATE TABLE IF NOT EXISTS "metadata_item" (
			"id"					INTEGER NOT NULL,

			"library_id" 			INTEGER,

			"source"				TEXT,
			"file_item_count"		INTEGER,
			"title"					TEXT,
			"original_title"		TEXT,
			"description"			TEXT,
			"rating"				FLOAT,
			"thumb_url"				TEXT,

			"cached"				TEXT,

			"tags_genre"			TEXT,
			"tags_collection"		TEXT,
			"tags_author"			TEXT,
			"tags_country"			TEXT,

			"available_at"			DATETIME,
			"year"					INTEGER,

			"refreshed_at"			DATETIME,
			"created_at"			DATETIME,
			"updated_at"			DATETIME,
			"deleted_at"			DATETIME,

			"hash"					TEXT,

			PRIMARY KEY("id" AUTOINCREMENT)
		);"#,
		[]
	)?;

	// Metadata People
	conn.execute(
		r#"CREATE TABLE IF NOT EXISTS "metadata_person" (
			"metadata_id"	INTEGER NOT NULL,
			"person_id"		INTEGER NOT NULL,

			UNIQUE(metadata_id, person_id)
		);"#,
		[]
	)?;


	// TODO: Versionize Notes. Keep last 20 versions for X one month. Auto delete old versions.
	// File Note
	conn.execute(
		r#"CREATE TABLE IF NOT EXISTS "file_note" (
			"file_id" 		INTEGER NOT NULL,
			"user_id" 		INTEGER NOT NULL,

			"data" 			TEXT NOT NULL,
			"data_size" 	INTEGER NOT NULL,

			"updated_at" 	DATETIME NOT NULL,
			"created_at" 	DATETIME NOT NULL,

			UNIQUE(file_id, user_id)
		);"#,
		[]
	)?;

	// File Progression
	conn.execute(
		r#"CREATE TABLE IF NOT EXISTS "file_progression" (
			"file_id" INTEGER NOT NULL,
			"user_id" INTEGER NOT NULL,

			"type_of" INTEGER NOT NULL,

			"chapter" INTEGER,
			"page" INTEGER,
			"char_pos" INTEGER,
			"seek_pos" INTEGER,

			"updated_at" DATETIME NOT NULL,
			"created_at" DATETIME NOT NULL,

			UNIQUE(file_id, user_id)
		);"#,
		[]
	)?;

	// File Notation
	conn.execute(
		r#"CREATE TABLE IF NOT EXISTS "file_notation" (
			"file_id" 		INTEGER NOT NULL,
			"user_id" 		INTEGER NOT NULL,

			"data" 			TEXT NOT NULL,
			"data_size" 	INTEGER NOT NULL,

			"updated_at" 	DATETIME NOT NULL,
			"created_at" 	DATETIME NOT NULL,

			UNIQUE(file_id, user_id)
		);"#,
		[]
	)?;

	// Tags People
	conn.execute(
		r#"CREATE TABLE IF NOT EXISTS "tag_person" (
			"id"			INTEGER NOT NULL,

			"source" 		TEXT NOT NULL,

			"name"			TEXT NOT NULL COLLATE NOCASE,
			"description"	TEXT,
			"birth_date"	TEXT,

			"thumb_url"		TEXT,

			"updated_at" 	DATETIME NOT NULL,
			"created_at" 	DATETIME NOT NULL,

			PRIMARY KEY("id" AUTOINCREMENT)
		);"#,
		[]
	)?;

	// People Alt names
	conn.execute(
		r#"CREATE TABLE IF NOT EXISTS "tag_person_alt" (
			"person_id"		INTEGER NOT NULL,

			"name"			TEXT NOT NULL COLLATE NOCASE,

			UNIQUE(person_id, name)
		);"#,
		[]
	)?;

	// Members
	conn.execute(
		r#"CREATE TABLE IF NOT EXISTS "members" (
			"id"			INTEGER NOT NULL,

			"name"			TEXT NOT NULL COLLATE NOCASE,
			"email"			TEXT COLLATE NOCASE,
			"password"		TEXT,
			"is_local"		INTEGER NOT NULL,
			"config"		TEXT,

			"created_at" 	DATETIME NOT NULL,
			"updated_at" 	DATETIME NOT NULL,

			UNIQUE(email),
			PRIMARY KEY("id" AUTOINCREMENT)
		);"#,
		[]
	)?;

	// Auths
	conn.execute(
		r#"CREATE TABLE IF NOT EXISTS "auths" (
			"oauth_token"			TEXT NOT NULL,
			"oauth_token_secret"	TEXT NOT NULL,

			"created_at"			DATETIME NOT NULL,

			UNIQUE(oauth_token)
		);"#,
		[]
	)?;


	// TODO: type_of for Author, Book Meta, etc..
	// Uploaded Images
	conn.execute(
		r#"CREATE TABLE IF NOT EXISTS "uploaded_images" (
			"id"			INTEGER NOT NULL,

			"link_id"		INTEGER NOT NULL,

			"path"			TEXT NOT NULL,

			"created_at"	DATETIME NOT NULL,

			UNIQUE(link_id, path),
			PRIMARY KEY("id" AUTOINCREMENT)
		);"#,
		[]
	)?;

	Ok(Database(Mutex::new(conn)))
}

// TODO: Replace with tokio Mutex?
pub struct Database(Mutex<Connection>);


impl Database {
	fn lock(&self) -> Result<MutexGuard<Connection>> {
		self.0.lock().map_err(|_| anyhow::anyhow!("Database Poisoned"))
	}


	// Libraries

	pub fn add_library(&self, name: String) -> Result<()> {
		// TODO: Create outside of fn.
		let lib = NewLibrary {
			name,
			type_of: String::new(),
			scanned_at: Utc::now(),
			created_at: Utc::now(),
			updated_at: Utc::now(),
		};

		self.lock()?.execute(
			r#"INSERT INTO library (name, type_of, scanned_at, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?5)"#,
			params![&lib.name, &lib.type_of, lib.scanned_at.timestamp_millis(), lib.created_at.timestamp_millis(), lib.updated_at.timestamp_millis()]
		)?;

		Ok(())
	}

	pub fn remove_library(&self, id: usize) -> Result<usize> {
		self.remove_directories_by_library_id(id)?;

		Ok(self.lock()?.execute(
			r#"DELETE FROM library WHERE id = ?1"#,
			params![id]
		)?)
	}

	pub fn list_all_libraries(&self) -> Result<Vec<Library>> {
		let this = self.lock()?;

		let mut conn = this.prepare("SELECT * FROM library")?;

		let map = conn.query_map([], |v| Library::try_from(v))?;

		Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
	}

	pub fn get_library_by_name(&self, value: &str) -> Result<Option<Library>> {
		Ok(self.lock()?.query_row(
			r#"SELECT * FROM library WHERE name = ?1 LIMIT 1"#,
			params![value],
			|v| Library::try_from(v)
		).optional()?)
	}


	// Directories

	pub fn add_directory(&self, library_id: usize, path: String) -> Result<()> {
		self.lock()?.execute(
			r#"INSERT INTO directory (library_id, path) VALUES (?1, ?2)"#,
			params![&library_id, &path]
		)?;

		Ok(())
	}

	pub fn remove_directory(&self, path: &str) -> Result<usize> {
		Ok(self.lock()?.execute(
			r#"DELETE FROM directory WHERE path = ?1"#,
			params![path]
		)?)
	}

	pub fn remove_directories_by_library_id(&self, id: usize) -> Result<usize> {
		Ok(self.lock()?.execute(
			r#"DELETE FROM directory WHERE library_id = ?1"#,
			params![id]
		)?)
	}

	pub fn get_directories(&self, library_id: usize) -> Result<Vec<Directory>> {
		let this = self.lock()?;

		let mut conn = this.prepare("SELECT * FROM directory WHERE library_id = ?1")?;

		let map = conn.query_map([library_id], |v| Directory::try_from(v))?;

		Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
	}

	pub fn get_all_directories(&self) -> Result<Vec<Directory>> {
		let this = self.lock()?;

		let mut conn = this.prepare("SELECT * FROM directory")?;

		let map = conn.query_map([], |v| Directory::try_from(v))?;

		Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
	}


	// Files

	pub fn add_file(&self, file: &NewFile) -> Result<()> {
		self.lock()?.execute(r#"
			INSERT INTO file (path, file_type, file_name, file_size, modified_at, accessed_at, created_at, library_id, metadata_id, chapter_count)
			VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
		"#,
		params![
			&file.path, &file.file_type, &file.file_name, file.file_size,
			file.modified_at.timestamp_millis(), file.accessed_at.timestamp_millis(), file.created_at.timestamp_millis(),
			file.library_id, file.metadata_id, file.chapter_count
		])?;

		Ok(())
	}

	pub fn file_exist(&self, file: &NewFile) -> Result<bool> {
		Ok(self.lock()?.query_row(r#"SELECT id FROM file WHERE path = ?1"#, [&file.path], |_| Ok(1)).optional()?.is_some())
	}

	pub fn get_files_by(&self, library: usize, offset: usize, limit: usize) -> Result<Vec<File>> {
		let this = self.lock()?;

		let mut conn = this.prepare("SELECT * FROM file WHERE library_id = ?1  LIMIT ?2 OFFSET ?3")?;

		let map = conn.query_map([library, limit, offset], |v| File::try_from(v))?;

		Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
	}

	pub fn get_files_with_metadata_by(&self, library: usize, offset: usize, limit: usize) -> Result<Vec<FileWithMetadata>> {
		let this = self.lock()?;

		let mut conn = this.prepare(r#"
			SELECT * FROM file
			LEFT JOIN metadata_item ON metadata_item.id = file.metadata_id
			WHERE library_id = ?1
			LIMIT ?2
			OFFSET ?3
		"#)?;

		let map = conn.query_map([library, limit, offset], |v| FileWithMetadata::try_from(v))?;

		Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
	}

	pub fn get_files_of_no_metadata(&self) -> Result<Vec<File>> {
		let this = self.lock()?;

		let mut conn = this.prepare("SELECT * FROM file WHERE metadata_id = 0 OR metadata_id = NULL")?;

		let map = conn.query_map([], |v| File::try_from(v))?;

		Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
	}

	pub fn find_file_by_id(&self, id: usize) -> Result<Option<File>> {
		Ok(self.lock()?.query_row(
			r#"SELECT * FROM file WHERE id=?1 LIMIT 1"#,
			params![id],
			|v| File::try_from(v)
		).optional()?)
	}

	pub fn find_file_by_id_with_metadata(&self, id: usize) -> Result<Option<FileWithMetadata>> {
		Ok(self.lock()?.query_row(
			r#"SELECT * FROM file LEFT JOIN metadata_item ON metadata_item.id = file.metadata_id WHERE file.id = ?1"#,
			[id],
			|v| FileWithMetadata::try_from(v)
		).optional()?)
	}

	pub fn get_files_by_metadata_id(&self, metadata_id: usize) -> Result<Vec<File>> {
		let this = self.lock()?;

		let mut conn = this.prepare("SELECT * FROM file WHERE metadata_id=?1")?;

		let map = conn.query_map([metadata_id], |v| File::try_from(v))?;

		Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
	}

	pub fn get_file_count(&self) -> Result<usize> {
		Ok(self.lock()?.query_row(r#"SELECT COUNT(*) FROM file"#, [], |v| v.get(0))?)
	}

	pub fn update_file_metadata_id(&self, file_id: usize, metadata_id: usize) -> Result<()> {
		self.lock()?
		.execute(r#"UPDATE file SET metadata_id = ?1 WHERE id = ?2"#,
			params![metadata_id, file_id]
		)?;

		Ok(())
	}

	pub fn change_files_metadata_id(&self, old_metadata_id: usize, new_metadata_id: usize) -> Result<usize> {
		Ok(self.lock()?
		.execute(r#"UPDATE file SET metadata_id = ?1 WHERE metadata_id = ?2"#,
			params![new_metadata_id, old_metadata_id]
		)?)
	}


	// Progression

	pub fn add_or_update_progress(&self, member_id: usize, file_id: usize, progress: Progression) -> Result<()> {
		let prog = FileProgression::new(progress, member_id, file_id);

		if self.get_progress(member_id, file_id)?.is_some() {
			self.lock()?.execute(
				r#"UPDATE file_progression SET chapter = ?1, char_pos = ?2, page = ?3, seek_pos = ?4, updated_at = ?5 WHERE file_id = ?6 AND user_id = ?7"#,
				params![prog.chapter, prog.char_pos, prog.page, prog.seek_pos, prog.updated_at.timestamp_millis(), prog.file_id, prog.user_id]
			)?;
		} else {
			self.lock()?.execute(
				r#"INSERT INTO file_progression (file_id, user_id, type_of, chapter, char_pos, page, seek_pos, updated_at, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)"#,
				params![prog.file_id, prog.user_id, prog.type_of, prog.chapter, prog.char_pos, prog.page, prog.seek_pos, prog.updated_at.timestamp_millis(), prog.created_at.timestamp_millis()]
			)?;
		}

		Ok(())
	}

	pub fn get_progress(&self, member_id: usize, file_id: usize) -> Result<Option<FileProgression>> {
		Ok(self.lock()?.query_row(
			"SELECT * FROM file_progression WHERE user_id = ?1 AND file_id = ?2",
			params![member_id, file_id],
			|v| FileProgression::try_from(v)
		).optional()?)
	}

	pub fn delete_progress(&self, member_id: usize, file_id: usize) -> Result<()> {
		self.lock()?.execute(
			"DELETE FROM file_progression WHERE user_id = ?1 AND file_id = ?2",
			params![member_id, file_id]
		)?;

		Ok(())
	}


	// Notes

	pub fn add_or_update_notes(&self, member_id: usize, file_id: usize, data: String) -> Result<()> {
		let prog = FileNote::new(file_id, member_id, data);

		if self.get_notes(member_id, file_id)?.is_some() {
			self.lock()?.execute(
				r#"UPDATE file_note SET data = ?1, data_size = ?2, updated_at = ?3 WHERE file_id = ?4 AND user_id = ?5"#,
				params![prog.data, prog.data_size, prog.updated_at.timestamp_millis(), prog.file_id, prog.user_id]
			)?;
		} else {
			self.lock()?.execute(
				r#"INSERT INTO file_note (file_id, user_id, data, data_size, updated_at, created_at) VALUES (?1, ?2, ?3, ?4, ?5, ?6)"#,
				params![prog.file_id, prog.user_id, prog.data, prog.data_size, prog.updated_at.timestamp_millis(), prog.created_at.timestamp_millis()]
			)?;
		}

		Ok(())
	}

	pub fn get_notes(&self, member_id: usize, file_id: usize) -> Result<Option<FileNote>> {
		Ok(self.lock()?.query_row(
			"SELECT * FROM file_note WHERE user_id = ?1 AND file_id = ?2",
			params![member_id, file_id],
			|v| FileNote::try_from(v)
		).optional()?)
	}

	pub fn delete_notes(&self, member_id: usize, file_id: usize) -> Result<()> {
		self.lock()?.execute(
			"DELETE FROM file_note WHERE user_id = ?1 AND file_id = ?2",
			params![member_id, file_id]
		)?;

		Ok(())
	}


	// Metadata

	pub fn add_or_increment_metadata(&self, meta: &MetadataItem) -> Result<MetadataItem> {
		let table_meta = if meta.id != 0 {
			self.get_metadata_by_id(meta.id)?
		} else {
			self.get_metadata_by_source(&meta.source)?
		};

		if table_meta.is_none() {
			self.lock()?
			.execute(r#"
				INSERT INTO metadata_item (
					library_id, source, file_item_count,
					title, original_title, description, rating, thumb_url,
					cached,
					available_at, year,
					refreshed_at, created_at, updated_at, deleted_at,
					hash
				)
				VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)"#,
				params![
					meta.library_id, meta.source.to_string(), &meta.file_item_count,
					&meta.title, &meta.original_title, &meta.description, &meta.rating, meta.thumb_path.to_optional_string(),
					&meta.cached.as_string_optional(),
					&meta.available_at, &meta.year,
					&meta.refreshed_at.timestamp_millis(), &meta.created_at.timestamp_millis(), &meta.updated_at.timestamp_millis(),
					meta.deleted_at.as_ref().map(|v| v.timestamp_millis()),
					&meta.hash
				]
			)?;

			return Ok(self.get_metadata_by_source(&meta.source)?.unwrap());
		} else if meta.id != 0 {
			self.lock()?
			.execute(r#"UPDATE metadata_item SET file_item_count = file_item_count + 1 WHERE id = ?1"#,
				params![meta.id]
			)?;
		} else {
			self.lock()?
			.execute(r#"UPDATE metadata_item SET file_item_count = file_item_count + 1 WHERE source = ?1"#,
				params![meta.source.to_string()]
			)?;
		}

		Ok(table_meta.unwrap())
	}

	pub fn update_metadata(&self, meta: &MetadataItem) -> Result<()> {
		self.lock()?
		.execute(r#"
			UPDATE metadata_item SET
				library_id = ?2, source = ?3, file_item_count = ?4,
				title = ?5, original_title = ?6, description = ?7, rating = ?8, thumb_url = ?9,
				cached = ?10,
				available_at = ?11, year = ?12,
				refreshed_at = ?13, created_at = ?14, updated_at = ?15, deleted_at = ?16,
				hash = ?17
			WHERE id = ?1"#,
			params![
				meta.id,
				meta.library_id, meta.source.to_string(), &meta.file_item_count,
				&meta.title, &meta.original_title, &meta.description, &meta.rating, meta.thumb_path.to_optional_string(),
				&meta.cached.as_string_optional(),
				&meta.available_at, &meta.year,
				&meta.refreshed_at.timestamp_millis(), &meta.created_at.timestamp_millis(), &meta.updated_at.timestamp_millis(),
				meta.deleted_at.as_ref().map(|v| v.timestamp_millis()),
				&meta.hash
			]
		)?;

		Ok(())
	}

	pub fn decrement_or_remove_metadata(&self, id: usize) -> Result<()> {
		if let Some(meta) = self.get_metadata_by_id(id)? {
			if meta.file_item_count < 1 {
				self.lock()?
				.execute(
					r#"UPDATE metadata_item SET file_item_count = file_item_count - 1 WHERE id = ?1"#,
					params![id]
				)?;
			} else {
				self.lock()?
				.execute(
					r#"DELETE FROM metadata_item WHERE id = ?1"#,
					params![id]
				)?;
			}
		}

		Ok(())
	}

	pub fn decrement_metadata(&self, id: usize) -> Result<()> {
		if let Some(meta) = self.get_metadata_by_id(id)? {
			if meta.file_item_count > 0 {
				self.lock()?
				.execute(
					r#"UPDATE metadata_item SET file_item_count = file_item_count - 1 WHERE id = ?1"#,
					params![id]
				)?;
			}
		}

		Ok(())
	}

	pub fn set_metadata_file_count(&self, id: usize, file_count: usize) -> Result<()> {
		self.lock()?
		.execute(
			r#"UPDATE metadata_item SET file_item_count = ?2 WHERE id = ?1"#,
			params![id, file_count]
		)?;

		Ok(())
	}

	// TODO: Change to get_metadata_by_hash. We shouldn't get metadata by source. Local metadata could be different with the same source id.
	pub fn get_metadata_by_source(&self, source: &Source) -> Result<Option<MetadataItem>> {
		Ok(self.lock()?.query_row(
			r#"SELECT * FROM metadata_item WHERE source = ?1 LIMIT 1"#,
			params![source.to_string()],
			|v| MetadataItem::try_from(v)
		).optional()?)
	}

	pub fn get_metadata_by_id(&self, id: usize) -> Result<Option<MetadataItem>> {
		Ok(self.lock()?.query_row(
			r#"SELECT * FROM metadata_item WHERE id = ?1 LIMIT 1"#,
			params![id],
			|v| MetadataItem::try_from(v)
		).optional()?)
	}

	pub fn remove_metadata_by_id(&self, id: usize) -> Result<usize> {
		Ok(self.lock()?.execute(
			r#"DELETE FROM metadata_item WHERE id = ?1"#,
			params![id]
		)?)
	}

	pub fn get_metadata_by(&self, library: usize, offset: usize, limit: usize) -> Result<Vec<MetadataItem>> {
		let this = self.lock()?;

		let mut conn = this.prepare(r#"SELECT * FROM metadata_item WHERE library_id = ?1 LIMIT ?2 OFFSET ?3"#)?;

		let map = conn.query_map([library, limit, offset], |v| MetadataItem::try_from(v))?;

		Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
	}


	// Metadata Person

	pub fn add_meta_person(&self, person: &MetadataPerson) -> Result<()> {
		self.lock()?.execute(r#"INSERT OR IGNORE INTO metadata_person (metadata_id, person_id) VALUES (?1, ?2)"#,
		params![
			&person.metadata_id,
			&person.person_id
		])?;

		Ok(())
	}

	pub fn remove_meta_person(&self, person: &MetadataPerson) -> Result<()> {
		self.lock()?.execute(r#"DELETE FROM metadata_person WHERE metadata_id = ?1 AND person_id = ?2"#,
		params![
			&person.metadata_id,
			&person.person_id
		])?;

		Ok(())
	}

	pub fn remove_persons_by_meta_id(&self, id: usize) -> Result<()> {
		self.lock()?.execute(r#"DELETE FROM metadata_person WHERE metadata_id = ?1"#,
		params![
			id
		])?;

		Ok(())
	}

	pub fn remove_meta_person_by_person_id(&self, id: usize) -> Result<()> {
		self.lock()?.execute(r#"DELETE FROM metadata_person WHERE person_id = ?1"#,
		params![
			id
		])?;

		Ok(())
	}

	pub fn transfer_meta_person(&self, from_id: usize, to_id: usize) -> Result<usize> {
		Ok(self.lock()?.execute(r#"UPDATE metadata_person SET person_id = ?2 WHERE person_id = ?1"#,
		params![
			from_id,
			to_id
		])?)
	}

	pub fn get_meta_person_list(&self, id: usize) -> Result<Vec<MetadataPerson>> {
		let this = self.lock()?;

		let mut conn = this.prepare(r#"SELECT * FROM metadata_person WHERE metadata_id = ?1"#)?;

		let map = conn.query_map([id], |v| MetadataPerson::try_from(v))?;

		Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
	}


	// Person

	pub fn add_person(&self, person: &NewTagPerson) -> Result<usize> {
		let conn = self.lock()?;

		conn.execute(r#"
			INSERT INTO tag_person (source, name, description, birth_date, thumb_url, updated_at, created_at)
			VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
		"#,
		params![
			person.source.to_string(), &person.name, &person.description, &person.birth_date, person.thumb_url.to_string(),
			person.updated_at.timestamp_millis(), person.created_at.timestamp_millis()
		])?;

		Ok(conn.last_insert_rowid() as usize)
	}

	pub fn get_person_list(&self, offset: usize, limit: usize) -> Result<Vec<TagPerson>> {
		let this = self.lock()?;

		let mut conn = this.prepare(r#"SELECT * FROM tag_person LIMIT ?1 OFFSET ?2"#)?;

		let map = conn.query_map([limit, offset], |v| TagPerson::try_from(v))?;

		Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
	}

	pub fn get_person_list_by_meta_id(&self, id: usize) -> Result<Vec<TagPerson>> {
		let this = self.lock()?;

		let mut conn = this.prepare(r#"
			SELECT tag_person.* FROM metadata_person
			LEFT JOIN
				tag_person ON tag_person.id = metadata_person.person_id
			WHERE metadata_id = ?1
		"#)?;

		let map = conn.query_map([id], |v| TagPerson::try_from(v))?;

		Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
	}

	pub fn search_person_list(&self, query: &str, offset: usize, limit: usize) -> Result<Vec<TagPerson>> {
		let mut escape_char = '\\';
		// Change our escape character if it's in the query.
		if query.contains(escape_char) {
			for car in [ '!', '@', '#', '$', '^', '&', '*', '-', '=', '+', '|', '~', '`', '/', '?', '>', '<', ',' ] {
				if !query.contains(car) {
					escape_char = car;
					break;
				}
			}
		}

		let sql = format!(
			r#"SELECT * FROM tag_person WHERE name LIKE '%{}%' ESCAPE '{}' LIMIT ?1 OFFSET ?2"#,
			query.replace('%', &format!("{}%", escape_char)).replace('_', &format!("{}_", escape_char)),
			escape_char
		);


		let this = self.lock()?;

		let mut conn = this.prepare(&sql)?;

		let map = conn.query_map(params![limit, offset], |v| TagPerson::try_from(v))?;

		Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
	}

	pub fn get_person_by_name(&self, value: &str) -> Result<Option<TagPerson>> {
		let person = self.lock()?.query_row(
			r#"SELECT * FROM tag_person WHERE name = ?1 LIMIT 1"#,
			params![value],
			|v| TagPerson::try_from(v)
		).optional()?;

		if let Some(person) = person {
			Ok(Some(person))
		} else if let Some(alt) = self.get_person_alt_by_name(value)? {
			self.get_person_by_id(alt.person_id)
		} else {
			Ok(None)
		}
	}

	pub fn get_person_by_id(&self, id: usize) -> Result<Option<TagPerson>> {
		Ok(self.lock()?.query_row(
			r#"SELECT * FROM tag_person WHERE id = ?1 LIMIT 1"#,
			params![id],
			|v| TagPerson::try_from(v)
		).optional()?)
	}

	pub fn get_person_by_source(&self, value: &str) -> Result<Option<TagPerson>> {
		Ok(self.lock()?.query_row(
			r#"SELECT * FROM tag_person WHERE source = ?1 LIMIT 1"#,
			params![value],
			|v| TagPerson::try_from(v)
		).optional()?)
	}

	pub fn get_person_count(&self) -> Result<usize> {
		Ok(self.lock()?.query_row(r#"SELECT COUNT(*) FROM tag_person"#, [], |v| v.get(0))?)
	}

	pub fn update_person(&self, person: &TagPerson) -> Result<()> {
		self.lock()?
		.execute(r#"
			UPDATE tag_person SET
				source = ?2,
				name = ?3,
				description = ?4,
				birth_date = ?5,
				thumb_url = ?6,
				updated_at = ?7,
				created_at = ?8
			WHERE id = ?1"#,
			params![
				person.id,
				person.source.to_string(), &person.name, &person.description, &person.birth_date, person.thumb_url.to_string(),
				person.updated_at.timestamp_millis(), person.created_at.timestamp_millis()
			]
		)?;

		Ok(())
	}

	pub fn remove_person_by_id(&self, id: usize) -> Result<usize> {
		Ok(self.lock()?.execute(
			r#"DELETE FROM tag_person WHERE id = ?1"#,
			params![id]
		)?)
	}


	// Person Alt

	pub fn add_person_alt(&self, person: &TagPersonAlt) -> Result<()> {
		self.lock()?.execute(r#"INSERT INTO tag_person_alt (name, person_id) VALUES (?1, ?2)"#,
		params![
			&person.name, &person.person_id
		])?;

		Ok(())
	}

	pub fn get_person_alt_by_name(&self, value: &str) -> Result<Option<TagPersonAlt>> {
		Ok(self.lock()?.query_row(
			r#"SELECT * FROM tag_person_alt WHERE name = ?1 LIMIT 1"#,
			params![value],
			|v| TagPersonAlt::try_from(v)
		).optional()?)
	}

	pub fn remove_person_alt(&self, tag_person: &TagPersonAlt) -> Result<usize> {
		Ok(self.lock()?.execute(
			r#"DELETE FROM tag_person_alt WHERE name = ?1 AND person_id = ?2"#,
			params![
				&tag_person.name,
				&tag_person.person_id
			]
		)?)
	}

	pub fn remove_person_alt_by_person_id(&self, id: usize) -> Result<usize> {
		Ok(self.lock()?.execute(
			r#"DELETE FROM tag_person_alt WHERE person_id = ?1"#,
			params![id]
		)?)
	}

	pub fn transfer_person_alt(&self, from_id: usize, to_id: usize) -> Result<usize> {
		Ok(self.lock()?.execute(r#"UPDATE OR IGNORE tag_person_alt SET person_id = ?2 WHERE person_id = ?1"#,
		params![
			from_id,
			to_id
		])?)
	}


	// Members

	pub fn add_member(&self, member: &NewMember) -> Result<usize> {
		let conn = self.lock()?;

		conn.execute(r#"
			INSERT INTO members (name, email, password, is_local, config, created_at, updated_at)
			VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
		"#,
		params![
			&member.name, member.email.as_ref(), member.password.as_ref(), member.type_of, member.config.as_ref(),
			member.created_at.timestamp_millis(), member.updated_at.timestamp_millis()
		])?;

		Ok(conn.last_insert_rowid() as usize)
	}

	pub fn get_member_by_email(&self, value: &str) -> Result<Option<Member>> {
		Ok(self.lock()?.query_row(
			r#"SELECT * FROM members WHERE email = ?1 LIMIT 1"#,
			params![value],
			|v| Member::try_from(v)
		).optional()?)
	}

	pub fn get_member_by_id(&self, id: usize) -> Result<Option<Member>> {
		Ok(self.lock()?.query_row(
			r#"SELECT * FROM members WHERE id = ?1 LIMIT 1"#,
			params![id],
			|v| Member::try_from(v)
		).optional()?)
	}


	// Verify

	pub fn add_verify(&self, auth: &NewAuth) -> Result<usize> {
		let conn = self.lock()?;

		conn.execute(r#"
			INSERT INTO auths (oauth_token, oauth_token_secret, created_at)
			VALUES (?1, ?2, ?3)
		"#,
		params![
			&auth.oauth_token,
			&auth.oauth_token_secret,
			auth.created_at.timestamp_millis()
		])?;

		Ok(conn.last_insert_rowid() as usize)
	}

	pub fn remove_verify_if_found_by_oauth_token(&self, value: &str) -> Result<bool> {
		Ok(self.lock()?.execute(
			r#"DELETE FROM auths WHERE oauth_token = ?1 LIMIT 1"#,
			params![value],
		)? != 0)
	}


	// Poster

	pub fn add_poster(&self, poster: &NewPoster) -> Result<usize> {
		if poster.path.is_none() {
			return Ok(0);
		}

		let conn = self.lock()?;

		conn.execute(r#"
			INSERT INTO uploaded_images (link_id, path, created_at)
			VALUES (?1, ?2, ?3)
		"#,
		params![
			poster.link_id,
			poster.path.to_string(),
			poster.created_at.timestamp_millis()
		])?;

		Ok(conn.last_insert_rowid() as usize)
	}

	pub fn get_posters_by_linked_id(&self, id: usize) -> Result<Vec<Poster>> {
		let this = self.lock()?;

		let mut conn = this.prepare(r#"SELECT * FROM uploaded_images WHERE link_id = ?1"#)?;

		let map = conn.query_map([id], |v| Poster::try_from(v))?;

		Ok(map.collect::<std::result::Result<Vec<_>, _>>()?)
	}

	pub fn get_poster_by_id(&self, id: usize) -> Result<Option<Poster>> {
		Ok(self.lock()?.query_row(
			r#"SELECT * FROM uploaded_images WHERE id = ?1 LIMIT 1"#,
			params![id],
			|v| Poster::try_from(v)
		).optional()?)
	}
}