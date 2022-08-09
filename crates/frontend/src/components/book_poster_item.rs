use common::BookId;
use common_local::{DisplayItem, api};
use web_sys::{MouseEvent, HtmlElement, HtmlInputElement};
use yew::{Component, Properties, Callback, NodeRef, Context, Html, html, TargetCast};
use yew_router::prelude::Link;

use crate::{Route, request};




#[derive(Properties)]
pub struct BookPosterItemProps {
    pub item: DisplayItem,
    pub callback: Callback<BookPosterItemMsg>,
    pub container_ref: Option<NodeRef>,
    #[prop_or_default]
    pub is_editing: bool,
    #[prop_or_default]
    pub is_updating: bool,
}

impl PartialEq for BookPosterItemProps {
    fn eq(&self, other: &Self) -> bool {
        self.item == other.item &&
        self.container_ref == other.container_ref &&
        self.is_editing == other.is_editing &&
        self.is_updating == other.is_updating
    }
}



#[derive(Clone)]
pub enum BookPosterItemMsg {
    PosterItem(PosterItem),

    AddOrRemoveItemFromEditing(BookId, bool),

    Ignore
}



pub struct BookPosterItem;

impl Component for BookPosterItem {
    type Message = BookPosterItemMsg;
    type Properties = BookPosterItemProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        ctx.props().callback.emit(msg);
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let &BookPosterItemProps {
            is_editing,
            is_updating,
            ref item,
            ref container_ref,
            ..
        } = ctx.props();

        let container_ref_clone = container_ref.clone();

        let book_id = item.id;

        let on_click_more = ctx.link().callback(move |e: MouseEvent| {
            e.prevent_default();
            e.stop_propagation();

            let scroll = container_ref_clone.as_ref().unwrap().cast::<HtmlElement>().unwrap().scroll_top();

            BookPosterItemMsg::PosterItem(PosterItem::ShowPopup(DisplayOverlayItem::More { book_id, mouse_pos: (e.page_x(), e.page_y() + scroll) }))
        });

        html! {
            <Link<Route> to={Route::ViewBook { book_id: item.id }} classes="book-list-item">
                <div class="poster">
                    <div class="top-left">
                        <input
                            checked={is_editing}
                            type="checkbox"
                            onclick={ctx.link().callback(move |e: MouseEvent| {
                                e.prevent_default();
                                e.stop_propagation();

                                BookPosterItemMsg::Ignore
                            })}
                            onmouseup={ctx.link().callback(move |e: MouseEvent| {
                                let input = e.target_unchecked_into::<HtmlInputElement>();

                                let value = !input.checked();

                                input.set_checked(value);

                                BookPosterItemMsg::AddOrRemoveItemFromEditing(book_id, value)
                            })}
                        />
                    </div>
                    <div class="bottom-right">
                        {
                            if container_ref.is_some() {
                                html! {
                                    <span class="material-icons" onclick={on_click_more} title="More Options">{ "more_horiz" }</span>
                                }
                            } else {
                                html! {}
                            }
                        }
                    </div>
                    <div class="bottom-left">
                        <span class="material-icons" onclick={ctx.link().callback_future(move |e: MouseEvent| {
                            e.prevent_default();
                            e.stop_propagation();

                            async move {
                                let resp = request::get_media_view(book_id).await;

                                match resp.ok() {
                                    Ok(res) => BookPosterItemMsg::PosterItem(PosterItem::ShowPopup(DisplayOverlayItem::Edit(Box::new(res)))),
                                    Err(err) => {
                                        crate::display_error(err);
                                        BookPosterItemMsg::Ignore
                                    }
                                }

                            }
                        })} title="More Options">{ "edit" }</span>
                    </div>
                    <img src={ item.get_thumb_url() } />
                    {
                        if is_updating {
                            html! {
                                <div class="changing"></div>
                            }
                        } else {
                            html! {}
                        }
                    }
                </div>
                <div class="info">
                    <div class="title" title={ item.title.clone() }>{ item.title.clone() }</div>
                    {
                        if let Some(author) = item.cached.author.as_ref() {
                            html! {
                                <div class="author" title={ author.clone() }>{ author.clone() }</div>
                            }
                        } else {
                            html! {}
                        }
                    }
                </div>
            </Link<Route>>
        }
    }
}




#[derive(Clone)]
pub enum PosterItem {
    // Poster Specific Buttons
    ShowPopup(DisplayOverlayItem),

    // Popup Events
    UpdateBookBySource(BookId),

    // Popup Events
    UpdateBookByFiles(BookId),
}

#[derive(Clone)]
pub enum DisplayOverlayItem {
    Info {
        book_id: BookId
    },

    Edit(Box<api::GetBookResponse>),

    More {
        book_id: BookId,
        mouse_pos: (i32, i32)
    },

    SearchForBook {
        book_id: BookId,
        input_value: Option<String>,
    },
}

impl PartialEq for DisplayOverlayItem {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Info { book_id: l_id }, Self::Info { book_id: r_id }) => l_id == r_id,
            (Self::More { book_id: l_id, .. }, Self::More { book_id: r_id, .. }) => l_id == r_id,
            (
                Self::SearchForBook { book_id: l_id, input_value: l_val, .. },
                Self::SearchForBook { book_id: r_id, input_value: r_val, .. }
            ) => l_id == r_id && l_val == r_val,

            _ => false
        }
    }
}
