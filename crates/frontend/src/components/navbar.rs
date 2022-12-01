use std::sync::{Arc, Mutex};

use common::{api::WrappingResponse, util::does_parent_contain_class};
use common_local::{api::GetBookListResponse, filter::FilterContainer, ThumbnailStoreExt};
use gloo_utils::{body, document};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_router::components::Link;

use crate::{request, BaseRoute};

#[derive(PartialEq, Eq, Properties)]
pub struct Property {
    pub visible: bool,
}

pub enum Msg {
    Close,
    SearchFor(String),
    SearchResults(WrappingResponse<GetBookListResponse>),
}

pub struct NavbarModule {
    left_items: Vec<(BaseRoute, DisplayType)>,
    right_items: Vec<(BaseRoute, DisplayType)>,

    search_results: Option<GetBookListResponse>,
    #[allow(clippy::type_complexity)]
    closure: Arc<Mutex<Option<Closure<dyn FnMut(MouseEvent)>>>>,
}

impl Component for NavbarModule {
    type Message = Msg;
    type Properties = Property;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            left_items: vec![
                (BaseRoute::Dashboard, DisplayType::Icon("home", "Home")),
                (BaseRoute::People, DisplayType::Icon("person", "Authors")),
            ],
            right_items: vec![(BaseRoute::Settings, DisplayType::Icon("settings", "Settings"))],

            search_results: None,
            closure: Arc::new(Mutex::new(None)),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Close => {
                self.search_results = None;
            }

            Msg::SearchFor(value) => {
                self.search_results = None;

                ctx.link().send_future(async move {
                    Msg::SearchResults(
                        request::get_books(None, Some(0), Some(20), {
                            let mut search = FilterContainer::default();
                            search.add_query_filter(value);
                            Some(search)
                        })
                        .await,
                    )
                });
            }

            Msg::SearchResults(resp) => match resp.ok() {
                Ok(res) => self.search_results = Some(res),
                Err(err) => crate::display_error(err),
            },
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let input_id = "book-search-input";

        html! {
            <div class={ classes!("navbar-module", (!ctx.props().visible).then_some("hidden")) }>
                <div class="left-content">
                {
                    for self.left_items.iter().map(|item| Self::render_item(item.0.clone(), &item.1))
                }
                </div>
                <div class="center-content">
                    <form class="search-bar row">
                        <input id={input_id} placeholder="Search" class="alternate" />
                        <button for={input_id} class="alternate" onclick={
                            ctx.link().callback(move |e: MouseEvent| {
                                e.prevent_default();

                                let input = document().get_element_by_id(input_id).unwrap().unchecked_into::<HtmlInputElement>();

                                Msg::SearchFor(input.value())
                            })
                        }>{ "Search" }</button>
                    </form>

                    { self.render_dropdown_results() }
                </div>
                <div class="right-content">
                {
                    for self.right_items.iter().map(|item| Self::render_item(item.0.clone(), &item.1))
                }
                </div>
            </div>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, _first_render: bool) {
        if let Some(func) = (*self.closure.lock().unwrap()).take() {
            let _ =
                body().remove_event_listener_with_callback("click", func.as_ref().unchecked_ref());
        }

        let closure = Arc::new(Mutex::default());

        let link = ctx.link().clone();
        let on_click = Closure::wrap(Box::new(move |event: MouseEvent| {
            if let Some(target) = event.target() {
                if !does_parent_contain_class(&target.unchecked_into(), "search-bar") {
                    link.send_message(Msg::Close);
                }
            }
        }) as Box<dyn FnMut(MouseEvent)>);

        let _ = body().add_event_listener_with_callback("click", on_click.as_ref().unchecked_ref());

        *closure.lock().unwrap() = Some(on_click);

        self.closure = closure;
    }

    fn destroy(&mut self, _ctx: &Context<Self>) {
        let func = (*self.closure.lock().unwrap()).take().unwrap();
        let _ = body().remove_event_listener_with_callback("click", func.as_ref().unchecked_ref());
    }
}

impl NavbarModule {
    fn render_item(route: BaseRoute, name: &DisplayType) -> Html {
        match name {
            DisplayType::Text(v) => html! {
                <Link<BaseRoute> to={route}>{ v }</Link<BaseRoute>>
            },
            DisplayType::Icon(icon, title) => html! {
                <Link<BaseRoute> to={route}>
                    <span class="material-icons" title={ *title }>{ icon }</span>
                </Link<BaseRoute>>
            },
        }
    }

    fn render_dropdown_results(&self) -> Html {
        if let Some(resp) = self.search_results.as_ref() {
            html! {
                <div class="search-dropdown">
                    {
                        for resp.items.iter().map(|item| {
                            html_nested! {
                                <Link<BaseRoute> to={BaseRoute::ViewBook { book_id: item.id }} classes={ classes!("search-item") }>
                                    <div class="poster max-vertical">
                                        <img src={ item.thumb_path.get_book_http_path().into_owned() } />
                                    </div>
                                    <div class="info">
                                        <h5 class="book-name" title={ item.title.clone() }>{ item.title.clone() }</h5>
                                    </div>
                                </Link<BaseRoute>>
                            }
                        })
                    }
                </div>
            }
        } else {
            html! {}
        }
    }
}

pub enum DisplayType {
    #[allow(dead_code)]
    Text(&'static str),
    Icon(&'static str, &'static str),
}
