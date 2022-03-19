use serde::{Serialize, Deserialize};
use wasm_bindgen::{JsValue, JsCast};
use wasm_bindgen_futures::JsFuture;
use web_sys::{RequestInit, Request, RequestMode, Response, Headers, FormData};

use books_common::{api::{GetBookIdResponse, GetBookListResponse, GetOptionsResponse, ModifyOptionsBody}, Progression};
use crate::pages::reading::ChapterInfo;

// TODO: Manage Errors.
// TODO: Correct different integer types.


// Books

pub async fn get_books(offset: Option<usize>, limit: Option<usize>) -> GetBookListResponse {
	let mut url = String::from("/api/books?");

	if let Some(value) = offset {
		url += "offset=";
		url += &value.to_string();
		url += "&";
	}

	if let Some(value) = limit {
		url += "limit=";
		url += &value.to_string();
	}

	fetch("GET", &url, Option::<&()>::None).await.unwrap()
}

pub async fn get_book_info(id: usize) -> GetBookIdResponse {
	fetch("GET", &format!("/api/book/{}", id), Option::<&()>::None).await.unwrap()
}

pub async fn get_book_pages(book_id: i64, start: usize, end: usize) -> ChapterInfo {
	fetch("GET", &format!("/api/book/{}/pages/{}-{}", book_id, start, end), Option::<&()>::None).await.unwrap()
}


// Progress

pub async fn update_book_progress(book_id: i64, progression: &Progression) {
	let _: Option<String> = fetch(
		"POST",
		&format!("/api/book/{}/progress", book_id),
		Some(progression)
	).await.ok();
}

pub async fn remove_book_progress(book_id: i64) {
	let _: Option<String> = fetch(
		"DELETE",
		&format!("/api/book/{}/progress", book_id),
		Option::<&()>::None
	).await.ok();
}


// Notes

pub async fn get_book_notes(book_id: i64) -> Option<String> {
	fetch("GET", &format!("/api/book/{}/notes", book_id), Option::<&()>::None).await.ok()
}

pub async fn update_book_notes(book_id: i64, data: String) {
	let _: Option<String> = fetch(
		"POST",
		&format!("/api/book/{}/notes", book_id),
		Some(&data)
	).await.ok();
}

pub async fn remove_book_notes(book_id: i64) {
	let _: Option<String> = fetch(
		"DELETE",
		&format!("/api/book/{}/notes", book_id),
		Option::<&()>::None
	).await.ok();
}


// Options

pub async fn get_options() -> GetOptionsResponse {
	fetch("GET", "/api/options", Option::<&()>::None).await.unwrap()
}

pub async fn update_options_add(options: ModifyOptionsBody) {
	let _: Option<String> = fetch(
		"POST",
		"/api/options/add",
		Some(&options)
	).await.ok();
}

pub async fn update_options_remove(options: ModifyOptionsBody) {
	let _: Option<String> = fetch(
		"POST",
		"/api/options/remove",
		Some(&options)
	).await.ok();
}

pub async fn run_task() { // TODO: Use common::api::RunTaskBody
	let _: Option<String> = fetch(
		"POST",
		"/api/task",
		Some(&serde_json::json!({
			"run_search": true,
			"run_metadata": true
		}))
	).await.ok();
}





async fn fetch<V: for<'a> Deserialize<'a>>(method: &str, url: &str, body: Option<&impl Serialize>) -> Result<V, JsValue> {
	let mut opts = RequestInit::new();
	opts.method(method);
	opts.mode(RequestMode::Cors);

	if let Some(body) = body {
		opts.body(Some(&JsValue::from_str(&serde_json::to_string(body).unwrap())));

		let headers = Headers::new()?;
		headers.append("Content-Type", "application/json")?;
		opts.headers(&headers);
	}

	let request = Request::new_with_str_and_init(url, &opts)?;

	let window = gloo_utils::window();
	let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
	let resp: Response = resp_value.dyn_into().unwrap();

	let text = JsFuture::from(resp.json()?).await?;

	Ok(text.into_serde().unwrap())
}


async fn fetch_url_encoded<V: for<'a> Deserialize<'a>>(method: &str, url: &str, form_data: FormData) -> Result<V, JsValue> {
	let mut opts = RequestInit::new();
	opts.method(method);
	opts.mode(RequestMode::Cors);

	opts.body(Some(&form_data));

	let request = Request::new_with_str_and_init(url, &opts)?;

	let window = gloo_utils::window();
	let resp_value = JsFuture::from(window.fetch_with_request(&request)).await?;
	let resp: Response = resp_value.dyn_into().unwrap();

	let text = JsFuture::from(resp.json()?).await?;

	Ok(text.into_serde().unwrap())
}