#![allow(clippy::let_unit_value, clippy::type_complexity)]

use std::{
    collections::HashMap,
    mem::MaybeUninit,
    rc::Rc,
    sync::{Arc, Mutex},
};

use common::{
    api::{ApiErrorResponse, WrappingResponse},
    component::popup::{Popup, PopupType},
    BookId, PersonId,
};
use common_local::{
    api,
    ws::{TaskId, TaskInfo},
    CollectionId, FileId, LibraryId, Member, Permissions,
};
use gloo_utils::body;
use lazy_static::lazy_static;
use services::open_websocket_conn;
use yew::prelude::*;
use yew_router::prelude::*;

use components::{NavbarModule, Sidebar};

mod components;
mod pages;
mod request;
mod services;
mod util;

#[derive(Debug, Clone, PartialEq)]
pub struct AppState {
    pub member: Option<Member>,

    pub is_navbar_visible: bool,
    pub update_nav_visibility: Callback<bool>,
}

lazy_static! {
    pub static ref RUNNING_TASKS: Mutex<HashMap<TaskId, TaskInfo>> = Mutex::default();
    static ref ERROR_POPUP: Arc<Mutex<Option<ApiErrorResponse>>> = Arc::new(Mutex::new(None));
}

thread_local! {
    static MAIN_MODEL: Arc<Mutex<MaybeUninit<AppHandle<Model>>>> = Arc::new(Mutex::new(MaybeUninit::uninit()));
}

pub fn request_member_self() {
    MAIN_MODEL.with(|v| unsafe {
        let lock = v.lock().unwrap();

        let scope = lock.assume_init_ref();

        scope.send_future(async { Msg::LoadMemberSelf(request::get_member_self().await) });
    });
}

pub fn display_error(value: ApiErrorResponse) {
    {
        *ERROR_POPUP.lock().unwrap() = Some(value);
    }

    MAIN_MODEL.with(|v| unsafe {
        let lock = v.lock().unwrap();

        let scope = lock.assume_init_ref();

        scope.send_message(Msg::Update);
    });
}

fn remove_error() {
    {
        *ERROR_POPUP.lock().unwrap() = None;
    }

    MAIN_MODEL.with(|v| unsafe {
        let lock = v.lock().unwrap();

        let scope = lock.assume_init_ref();

        scope.send_message(Msg::Update);
    });
}

enum Msg {
    LoadMemberSelf(WrappingResponse<api::GetMemberSelfResponse>),
    GetTasksResponse(WrappingResponse<Vec<(TaskId, TaskInfo)>>),

    UpdateNavVis(bool),

    Update,
}

struct Model {
    state: Rc<AppState>,

    has_loaded_member: bool,
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link()
            .send_future(async { Msg::LoadMemberSelf(request::get_member_self().await) });

        Self {
            state: Rc::new(AppState {
                member: None,

                is_navbar_visible: false,
                update_nav_visibility: ctx.link().callback(Msg::UpdateNavVis),
            }),
            has_loaded_member: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::LoadMemberSelf(resp) => {
                let mut await_tasks = false;

                if let WrappingResponse::Resp(resp) = resp {
                    if let Some(member) = resp.member.as_ref() {
                        if member.permissions.is_owner() {
                            ctx.link().send_future(async {
                                Msg::GetTasksResponse(request::get_tasks().await)
                            });

                            await_tasks = true;
                        }

                        open_websocket_conn();
                    }

                    let state = Rc::make_mut(&mut self.state);

                    state.is_navbar_visible = resp.member.is_some();
                    state.member = resp.member;
                }

                self.has_loaded_member = !await_tasks;
            }

            Msg::GetTasksResponse(resp) => {
                match resp.ok() {
                    Ok(resp) => {
                        *RUNNING_TASKS.lock().unwrap() = HashMap::from_iter(resp);
                    }

                    Err(e) => display_error(e),
                }

                self.has_loaded_member = true;
            }

            Msg::UpdateNavVis(value) => {
                Rc::make_mut(&mut self.state).is_navbar_visible = value;
            }

            Msg::Update => (),
        }

        true
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <>
                <BrowserRouter>
                    <ContextProvider<Rc<AppState>> context={ self.state.clone() }>
                    {
                        if self.has_loaded_member {
                            let member_permissions = self.state.member.as_ref().map(|v| v.permissions);

                            html! {
                                    <>
                                        <Sidebar visible={ self.state.is_navbar_visible } />
                                        <div class="outer-view-container flex-column">
                                            <NavbarModule visible={ self.state.is_navbar_visible } />
                                            <Switch<BaseRoute> render={ move |route| switch_base(route, member_permissions) } />
                                        </div>
                                    </>
                                }
                            } else {
                                html! {
                                    <div>
                                        <h1>{ "Loading..." }</h1>
                                    </div>
                                }
                            }
                        }
                    </ContextProvider<Rc<AppState>>>
                </BrowserRouter>

                {
                    if let Some(error) = ERROR_POPUP.lock().unwrap().as_ref() {
                        html! {
                            <Popup
                                type_of={ PopupType::FullOverlay }
                                on_close={ Callback::from(|_| remove_error()) }
                            >
                                <p>{ error.description.clone() }</p>
                            </Popup>
                        }
                    } else {
                        html! {}
                    }
                }
            </>
        }
    }
}

#[derive(Routable, PartialEq, Eq, Clone, Debug)]
pub enum BaseRoute {
    #[at("/login")]
    Login,

    #[at("/logout")]
    Logout,

    #[at("/library/:id")]
    ViewLibrary { id: LibraryId },

    #[at("/view/:book_id")]
    ViewBook { book_id: BookId },

    #[at("/read/:book_id")]
    ReadBook { book_id: FileId },

    #[at("/people")]
    People,

    #[at("/person/:person_id")]
    ViewPerson { person_id: PersonId },

    #[at("/collections")]
    Collections,

    #[at("/collection/:id")]
    ViewCollection { id: CollectionId },

    #[at("/settings/*")]
    Settings,

    #[at("/setup")]
    Setup,

    #[at("/")]
    Dashboard,
}

fn switch_base(route: BaseRoute, permissions: Option<Permissions>) -> Html {
    log::info!("{:?}", route);

    if permissions.is_none() && route != BaseRoute::Setup {
        return html! { <pages::LoginPage /> };
    }

    match route {
        BaseRoute::Login => {
            html! { <pages::LoginPage /> }
        }

        BaseRoute::Logout => {
            html! { <pages::LogoutPage /> }
        }

        BaseRoute::ViewLibrary { id } => {
            html! { <pages::LibraryPage id={id} /> }
        }

        BaseRoute::ViewBook { book_id } => {
            html! { <pages::BookPage id={book_id} /> }
        }

        BaseRoute::ReadBook { book_id } => {
            html! { <pages::ReadingBook id={book_id} /> }
        }

        BaseRoute::People => {
            html! { <pages::AuthorListPage /> }
        }

        BaseRoute::ViewPerson { person_id } => {
            html! { <pages::AuthorView id={person_id} /> }
        }

        BaseRoute::Collections => {
            html! { <pages::CollectionListPage /> }
        }

        BaseRoute::ViewCollection { id } => {
            html! { <pages::CollectionItemPage {id} /> }
        }

        BaseRoute::Settings => {
            html! { <Switch<pages::settings::SettingsRoute> render={ move |route: pages::settings::SettingsRoute| {
                if route.is_admin() && !permissions.unwrap().is_owner() {
                    return html_container("No Permissions");
                }

                pages::settings::switch_settings(route)
            } } /> }
        }

        BaseRoute::Setup => {
            html! { <pages::SetupPage /> }
        }

        BaseRoute::Dashboard => {
            html! { <pages::HomePage /> }
        }
    }
}

fn html_container(value: &'static str) -> Html {
    html! {
        <div class="view-container">
            <h1>{ value }</h1>
        </div>
    }
}


fn main() {
    wasm_logger::init(wasm_logger::Config::default());

    log::debug!("starting...");

    body().set_class_name("text-light d-flex");

    let handle = yew::Renderer::<Model>::new().render();

    MAIN_MODEL.with(move |v| *v.lock().unwrap() = MaybeUninit::new(handle));
}
