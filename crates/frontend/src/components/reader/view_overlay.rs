use std::rc::Rc;

use web_sys::MouseEvent;
use yew::{function_component, Properties, html, use_node_ref, use_effect_with_deps, Callback, use_state_eq, use_state, NodeRef, UseStateHandle};
use yew_hooks::{use_swipe, UseSwipeDirection, use_event};




pub struct OverlayEvent {
    pub type_of: DragType,
    pub dragging: bool,
}

#[derive(Debug)]
pub enum DragType {
    Up(usize),
    Right(usize),
    Down(usize),
    Left(usize),
    None,
}



#[derive(PartialEq, Properties)]
pub struct ViewOverlayProps {
    pub event: Callback<OverlayEvent>,
}


#[function_component(ViewOverlay)]
pub fn _view_overlay(props: &ViewOverlayProps) -> Html {
    let node =  use_node_ref();
    let state = use_swipe(node.clone());
    let state2 = use_mouse(node.clone());

    { // Swipe
        use_effect_with_deps(move |direction| {
            log::info!("Swipe: {:?}", **direction);

            match **direction {
                UseSwipeDirection::Left => (),
                UseSwipeDirection::Right => (),
                UseSwipeDirection::Up => (),
                UseSwipeDirection::Down => (),
                _ => (),
            }
            || ()
        }, state.direction);
    }

    { // Mouse
        let event = props.event.clone();
        let curr_event_state = use_state_eq(|| false);

        use_effect_with_deps(move |handle| {
            let distance = match *handle.direction {
                UseSwipeDirection::Left => handle.length_x.abs(),
                UseSwipeDirection::Right => handle.length_x.abs(),
                UseSwipeDirection::Up => handle.length_y.abs(),
                UseSwipeDirection::Down => handle.length_y.abs(),
                UseSwipeDirection::None => 0,
            } as usize;

            let direction = match *handle.direction {
                UseSwipeDirection::Left => DragType::Left(distance),
                UseSwipeDirection::Right => DragType::Right(distance),
                UseSwipeDirection::Up => DragType::Up(distance),
                UseSwipeDirection::Down => DragType::Down(distance),
                UseSwipeDirection::None => DragType::None,
            };

            // If we're dragging the mouse down and it's registered as moving.
            if *handle.dragging && *handle.moving {
                curr_event_state.set(true);

                event.emit(OverlayEvent {
                    type_of: direction,
                    dragging: true,
                });
            } else if !*handle.dragging && *curr_event_state {
                curr_event_state.set(false);

                event.emit(OverlayEvent {
                    type_of: direction,
                    dragging: false,
                });
            }

            || ()
        }, state2);
    }

    html! {
        <div class="view-overlay" ref={ node } style="user-select: none;"></div>
    }
}





// Based off Swipe

#[derive(Debug, PartialEq)]
pub struct UseMouseHandle {
    pub dragging: UseStateHandle<bool>,
    pub moving: UseStateHandle<bool>,

    pub direction: UseStateHandle<UseSwipeDirection>,

    pub coords_start: UseStateHandle<(i32, i32)>,
    pub coords_end: UseStateHandle<Option<(i32, i32)>>,

    pub length_x: UseStateHandle<i32>,
    pub length_y: UseStateHandle<i32>,
}

impl Clone for UseMouseHandle {
    fn clone(&self) -> Self {
        Self {
            dragging: self.dragging.clone(),
            moving: self.moving.clone(),
            direction: self.direction.clone(),
            coords_start: self.coords_start.clone(),
            coords_end: self.coords_end.clone(),
            length_x: self.length_x.clone(),
            length_y: self.length_y.clone(),
        }
    }
}

pub fn use_mouse(node: NodeRef) -> UseMouseHandle {
    let dragging = use_state_eq(|| false);
    let moving = use_state_eq(|| false);
    let direction = use_state_eq(|| UseSwipeDirection::None);
    let coords_start = use_state(|| (0, 0));
    let coords_end = use_state(|| Option::<(i32, i32)>::None);
    let length_x = use_state(|| 0);
    let length_y = use_state(|| 0);

    let threshold = 5;

    let diff_x = {
        let coords_start = coords_start.clone();
        let coords_end = coords_end.clone();

        Rc::new(move || {
            if let Some(coords_end) = *coords_end {
                ((*coords_start).0 - coords_end.0) as i32
            } else {
                0
            }
        })
    };

    let diff_y = {
        let coords_start = coords_start.clone();
        let coords_end = coords_end.clone();

        Rc::new(move || {
            if let Some(coords_end) = *coords_end {
                ((*coords_start).1 - coords_end.1) as i32
            } else {
                0
            }
        })
    };

    let threshold_exceeded = {
        let diff_x = diff_x.clone();
        let diff_y = diff_y.clone();

        Rc::new(move || diff_x().abs().max(diff_y().abs()) >= (threshold as i32))
    };

    {
        let coords_start = coords_start.clone();
        let coords_end = coords_end.clone();
        let dragging = dragging.clone();

        use_event(node.clone(), "mousedown", move |e: MouseEvent| {
            let x = e.x();
            let y = e.y();

            coords_start.set((x, y));
            coords_end.set(None);
            dragging.set(true);
        });
    }

    {
        let coords_end = coords_end.clone();
        let moving = moving.clone();
        let length_x = length_x.clone();
        let length_y = length_y.clone();
        let direction = direction.clone();
        let dragging = dragging.clone();

        use_event(node.clone(), "mousemove", move |e: MouseEvent| {
            // TODO: Should I keep. This prevents one-time clicks from changing the page.
            if !*dragging {
                return;
            }

            let x = e.x();
            let y = e.y();

            coords_end.set(Some((x, y)));

            length_x.set(diff_x());
            length_y.set(diff_y());

            if !*moving && threshold_exceeded() {
                moving.set(true);
            }

            if !threshold_exceeded() {
                direction.set(UseSwipeDirection::None);
            } else if diff_x().abs() > diff_y().abs() {
                if diff_x() > 0 {
                    direction.set(UseSwipeDirection::Left);
                } else {
                    direction.set(UseSwipeDirection::Right);
                }
            } else if diff_y() > 0 {
                direction.set(UseSwipeDirection::Up);
            } else {
                direction.set(UseSwipeDirection::Down);
            }
        });
    }

    {
        let moving = moving.clone();
        let direction = direction.clone();
        let dragging = dragging.clone();

        use_event(node, "mouseup", move |_: MouseEvent| {
            moving.set(false);
            dragging.set(false);
            direction.set(UseSwipeDirection::None);
        });

    }

    UseMouseHandle {
        dragging,
        moving,
        direction,
        coords_start,
        coords_end,
        length_x,
        length_y,
    }
}
