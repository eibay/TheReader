use actix_web::{get, web, HttpResponse, post};
use books_common::{api, LibraryColl};

use crate::database::Database;


#[get("/options")]
async fn load_options(db: web::Data<Database>) -> web::Json<api::GetOptionsResponse> {
	let libraries = db.list_all_libraries().unwrap();
	let mut directories = db.get_all_directories().unwrap();

	web::Json(api::GetOptionsResponse {
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
	})
}

// TODO: Move to utils file.
fn take_from_and_swap<V, P: Fn(&V) -> bool>(array: &mut Vec<V>, predicate: P) -> Vec<V> {
	let mut ret = Vec::new();

	for i in (0..array.len()).rev() {
		if predicate(&array[i]) {
			ret.push(array.swap_remove(i));
		}
	}

	ret.reverse();

	ret
}

#[post("/options/add")]
async fn update_options_add(modify: web::Json<api::ModifyOptionsBody>, db: web::Data<Database>) -> HttpResponse {
	let api::ModifyOptionsBody {
		library,
		directory
	} = modify.into_inner();

	if let Some(library) = library {
		db.add_library(library.name.unwrap()).unwrap();
	}

	if let Some(directory) = directory {
		// TODO: Don't trust that the path is correct. Also remove slashes at the end of path.
		db.add_directory(directory.library_id, directory.path).unwrap();
	}

	HttpResponse::Ok().finish()
}

#[post("/options/remove")]
async fn update_options_remove(modify: web::Json<api::ModifyOptionsBody>, db: web::Data<Database>) -> HttpResponse {
	let api::ModifyOptionsBody {
		library,
		directory
	} = modify.into_inner();

	if let Some(library) = library {
		db.remove_library(library.id.unwrap()).unwrap();
	}

	if let Some(directory) = directory {
		db.remove_directory(&directory.path).unwrap();
	}

	HttpResponse::Ok().finish()
}