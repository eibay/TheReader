use books_common::{MediaItem, api};
use gloo_utils::{document, window};
use wasm_bindgen::{prelude::Closure, JsCast};
use yew::{prelude::*, html::Scope};
use yew_router::prelude::Link;

use crate::{Route, request};


#[derive(Properties, PartialEq)]
pub struct Property {
	pub library_id: i64,
}

pub enum Msg {
	// Requests
	RequestMediaItems,

	// Results
	MediaListResults(api::GetBookListResponse),

	// Events
	OnScroll(i32),
	PosterItem(PosterItem),

	Ignore
}

pub struct LibraryPage {
	on_scroll_fn: Option<Closure<dyn FnMut()>>,

	media_items: Option<Vec<MediaItem>>,
	total_media_count: i64,

	is_fetching_media_items: bool,
}

impl Component for LibraryPage {
	type Message = Msg;
	type Properties = Property;

	fn create(_ctx: &Context<Self>) -> Self {
		Self {
			on_scroll_fn: None,
			media_items: None,
			total_media_count: 0,
			is_fetching_media_items: false,
		}
	}

	fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
		match msg {
			Msg::RequestMediaItems => {
				if self.is_fetching_media_items {
					return false;
				}

				self.is_fetching_media_items = true;

				let offset = Some(self.media_items.as_ref().map(|v| v.len()).unwrap_or_default()).filter(|v| *v != 0);

				let library = ctx.props().library_id;

				ctx.link()
				.send_future(async move {
					Msg::MediaListResults(request::get_books(library, offset, None).await)
				});
			}

			Msg::MediaListResults(mut resp) => {
				self.is_fetching_media_items = false;
				self.total_media_count = resp.count;

				if let Some(items) = self.media_items.as_mut() {
					items.append(&mut resp.items);
				} else {
					self.media_items = Some(resp.items);
				}
			}

			Msg::OnScroll(scroll_y) => {
				let scroll_height = document().body().unwrap().scroll_height();

				if scroll_height - scroll_y < 600 && self.can_req_more() {
					ctx.link().send_message(Msg::RequestMediaItems);
				}
			}

			Msg::PosterItem(item) => match item {
				PosterItem::Meta(file_id) => {
					ctx.link()
					.send_future(async move {
						request::update_metadata(&api::PostMetadataBody::File(file_id)).await;

						Msg::Ignore
					});
				}
			}

			Msg::Ignore => return false,
		}

		true
	}

	fn view(&self, ctx: &Context<Self>) -> Html {
		if let Some(items) = self.media_items.as_deref() {
			// TODO: Placeholders
			// let remaining = (self.total_media_count as usize - items.len()).min(50);

			html! {
				<div class="library-list normal">
					{ for items.iter().map(|item| Self::render_media_item(item, ctx.link())) }
					// { for (0..remaining).map(|_| Self::render_placeholder_item()) }
				</div>
			}
		} else {
			html! {
				<h1>{ "Loading..." }</h1>
			}
		}
	}

	fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
		if first_render {
			let link = ctx.link().clone();
			let func = Closure::wrap(Box::new(move || {
				link.send_message(Msg::OnScroll(
					(
						window().inner_height().map(|v| v.as_f64().unwrap()).unwrap_or_default() +
						window().scroll_y().unwrap_or_default()
					) as i32
				));
			}) as Box<dyn FnMut()>);

			let _ = document().add_event_listener_with_callback("scroll", func.as_ref().unchecked_ref());

			self.on_scroll_fn = Some(func);

			ctx.link().send_message(Msg::RequestMediaItems);
		}
	}

	fn destroy(&mut self, _ctx: &Context<Self>) {
		if let Some(f) = self.on_scroll_fn.take() {
			let _ = document().remove_event_listener_with_callback("scroll", f.as_ref().unchecked_ref());
		}
	}
}

impl LibraryPage {
	fn render_media_item(item: &MediaItem, scope: &Scope<Self>) -> Html {
		let file_id = item.id;
		let on_click_meta = scope.callback(move |e: MouseEvent| {
			e.prevent_default();
			e.stop_propagation();

			Msg::PosterItem(PosterItem::Meta(file_id))
		});

		html! {
			<Link<Route> to={Route::ReadBook { book_id: item.id as usize }} classes={ classes!("library-item") }>
				<div class="poster">
					<div class="bottom-right">
						<span class="material-icons" onclick={on_click_meta} title="Update Metadata">{ "search" }</span>
					</div>
					<img src={ if item.icon_path.is_some() { format!("/api/book/{}/thumbnail", item.id) } else { String::from("/images/missingthumbnail.jpg") } } />
				</div>
				<div class="info">
					<div class="title" title={ item.title.clone() }>{ item.title.clone() }</div>
					<div class="author" title={ item.author.clone() }>{ item.author.clone() }</div>
				</div>
			</Link<Route>>
		}
	}

	// fn render_placeholder_item() -> Html {
	// 	html! {
	// 		<div class="library-item placeholder">
	// 			<div class="poster"></div>
	// 			<div class="info">
	// 				<a class="author"></a>
	// 				<a class="title"></a>
	// 			</div>
	// 		</div>
	// 	}
	// }

	pub fn can_req_more(&self) -> bool {
		let count = self.media_items.as_ref().map(|v| v.len()).unwrap_or_default();

		count != 0 && count != self.total_media_count as usize
	}
}

pub enum PosterItem {
	Meta(i64)
}