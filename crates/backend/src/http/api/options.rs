use actix_web::{get, web, HttpResponse, post, delete};
use books_common::{api, LibraryColl, util::take_from_and_swap};

use crate::{database::Database, WebResult};


#[get("/options")]
async fn load_options(db: web::Data<Database>) -> WebResult<web::Json<api::ApiGetOptionsResponse>> {
	let libraries = db.list_all_libraries()?;
	let mut directories = db.get_all_directories()?;

	Ok(web::Json(api::GetOptionsResponse {
		libraries: libraries.into_iter()
			.map(|lib| {
				LibraryColl {
					id: lib.id,
					name: lib.name,
					scanned_at: lib.scanned_at.timestamp_millis(),
					created_at: lib.created_at.timestamp_millis(),
					updated_at: lib.updated_at.timestamp_millis(),
					directories: take_from_and_swap(&mut directories, |v| v.library_id == lib.id)
						.into_iter()
						.map(|v| v.path)
						.collect()
				}
			})
			.collect()
	}))
}

#[post("/options")]
async fn update_options_add(modify: web::Json<api::ModifyOptionsBody>, db: web::Data<Database>) -> WebResult<HttpResponse> {
	let api::ModifyOptionsBody {
		library,
		directory
	} = modify.into_inner();

	if let Some(name) = library.and_then(|v| v.name) {
		db.add_library(name)?;
	}

	if let Some(directory) = directory {
		// TODO: Don't trust that the path is correct. Also remove slashes at the end of path.
		db.add_directory(directory.library_id, directory.path)?;
	}

	Ok(HttpResponse::Ok().finish())
}

#[delete("/options")]
async fn update_options_remove(modify: web::Json<api::ModifyOptionsBody>, db: web::Data<Database>) -> WebResult<HttpResponse> {
	let api::ModifyOptionsBody {
		library,
		directory
	} = modify.into_inner();

	if let Some(id) = library.and_then(|v| v.id) {
		db.remove_library(id)?;
	}

	if let Some(directory) = directory {
		db.remove_directory(&directory.path)?;
	}

	Ok(HttpResponse::Ok().finish())
}