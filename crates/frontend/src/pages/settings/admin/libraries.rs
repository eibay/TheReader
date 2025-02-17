use common::{
    api::WrappingResponse,
    component::popup::{Popup, PopupType},
};
use common_local::{api, BasicLibrary, LibraryId};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew_hooks::use_list;

use crate::{components::edit::library::LibraryEdit, request};

pub enum Msg {
    // Request Results
    OptionsResults(Box<WrappingResponse<api::GetOptionsResponse>>),

    // Events
    DisplayPopup(usize, LibraryId),
    ClosePopup,

    RequestUpdateOptions(bool, api::ModifyOptionsBody),
}

pub struct AdminLibrariesPage {
    resp: Option<api::GetOptionsResponse>,
    visible_popup: Option<(usize, LibraryId)>,
}

impl Component for AdminLibrariesPage {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            resp: None,
            visible_popup: None,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::OptionsResults(resp) => match resp.ok() {
                Ok(resp) => {
                    self.resp = Some(resp);
                    self.visible_popup = None;
                }
                Err(err) => crate::display_error(err),
            },

            Msg::DisplayPopup(popup, index) => {
                self.visible_popup = Some((popup, index));
            }

            Msg::ClosePopup => {
                self.visible_popup = None;
            }

            Msg::RequestUpdateOptions(is_adding, options) => {
                ctx.link().send_future(async move {
                    if is_adding {
                        request::update_options_add(options).await;
                    } else {
                        request::update_options_remove(options).await;
                    }

                    Msg::OptionsResults(Box::new(request::get_options().await))
                });
            }
        }

        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let render = if let Some(resp) = self.resp.as_ref() {
            html! {
                // We use a empty div to prevent the buttons' widths from fully expanding.
                <div>
                    <h2>{ "Libraries" }</h2>
                    {
                        for resp.libraries.iter()
                            .map(|v| {
                                let id = v.id;

                                html! {
                                    <div>
                                        <button class="btn btn-secondary btn-sm" onclick={ ctx.link().callback(move |_| Msg::DisplayPopup(1, id)) }>
                                            { v.name.clone() }
                                        </button>
                                    </div>
                                }
                            })
                    }
                    <button class="btn btn-success btn-sm" onclick={ctx.link().callback(|_| Msg::DisplayPopup(0, LibraryId::none()))}>{ "Add Library" }</button>

                    { self.render_popup(ctx) }
                </div>
            }
        } else {
            html! {
                <h1>{ "Loading..." }</h1>
            }
        };

        html! {
            <div class="view-container">
                { render }
            </div>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            ctx.link()
                .send_future(async { Msg::OptionsResults(Box::new(request::get_options().await)) });
        }
    }
}

impl AdminLibrariesPage {
    fn render_popup(&self, ctx: &Context<Self>) -> Html {
        if let Some((popup_id, library_id)) = self.visible_popup {
            // TODO: Make popup component for this.

            match popup_id {
                // Add Library
                0 => html! {
                    <NewLibrary callback={ ctx.link().callback(|v| v) } />
                },

                1 => html! {
                    <Popup type_of={ PopupType::FullOverlay } on_close={ ctx.link().callback(|_| Msg::ClosePopup) }>
                        <LibraryEdit
                            id={ library_id }
                            on_change={ ctx.link().callback_future(move |v| async move {
                                request::update_library(library_id, &v).await;

                                Msg::ClosePopup
                            }) }
                        />
                    </Popup>
                },

                _ => html! {},
            }
        } else {
            html! {}
        }
    }
}

#[derive(Properties, PartialEq)]
pub struct NewLibraryProps {
    pub callback: Callback<Msg>,
}

#[function_component(NewLibrary)]
fn new_library(props: &NewLibraryProps) -> Html {
    let library_name = use_state(String::new);
    let directories = use_list(Vec::<String>::new());

    let on_create = {
        let dirs = directories.clone();
        let name = library_name.clone();

        props.callback.reform(move |_| {
            Msg::RequestUpdateOptions(
                true,
                api::ModifyOptionsBody {
                    library: Some(BasicLibrary {
                        id: None,
                        name: Some(name.to_string()),
                        directories: Some(dirs.current().to_vec()),
                    }),
                    ..Default::default()
                },
            )
        })
    };

    let on_new_dir_path = {
        let dirs = directories.clone();
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                let input = e.target_unchecked_into::<HtmlInputElement>();

                dirs.push(input.value());

                input.set_value("");
            }
        })
    };

    let on_change_lib_name = {
        let name = library_name.setter();

        Callback::from(move |e: Event| {
            name.set(e.target_unchecked_into::<HtmlInputElement>().value());
        })
    };

    html! {
        <Popup
            // classes=""
            type_of={ PopupType::FullOverlay }
            on_close={ props.callback.reform(|_| Msg::ClosePopup) }
        >
            <div class="modal-header">
                <h2 class="modal-title">{ "New Library" }</h2>
            </div>

            <div class="modal-body">
                <div class="mb-3 input-group">
                    <input class="form-control" type="text" name="library-name" placeholder="Library Name" onchange={ on_change_lib_name } />
                    <button class="btn btn-success btn-sm" onclick={ on_create }>{"Create"}</button>
                </div>

                <h5>{ "Directories" }</h5>

                <div class="directories">
                    {
                        for directories.current()
                            .iter()
                            .enumerate()
                            .map(|(index, path)| {
                                let dirs = directories.clone();

                                let onclick = Callback::from(move |_| { dirs.remove(index); });

                                html! {
                                    <div class="mb-3">
                                        <button class="btn btn-danger btn-sm" {onclick}>{ "X" }</button>
                                        <span>{ path.clone() }</span>
                                    </div>
                                }
                            }
                        )
                    }

                    <input class="form-control" onkeypress={ on_new_dir_path } />
                </div>
            </div>

        </Popup>
    }
}
