use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{KeyboardEvent, Window};

use super::{
    SharedInput, BUTTON_ATTACK, BUTTON_CONFIRM, BUTTON_CYCLE, BUTTON_DASH, BUTTON_INTERACT,
    BUTTON_ITEM, BUTTON_PAUSE, DEBUG_ACTION, DEBUG_MAP, DEBUG_OVERLAY, DEBUG_TELEPORT,
    DEBUG_VIEWER,
};

pub fn attach(
    window: &Window,
    shared: Rc<RefCell<SharedInput>>,
    listeners: &mut Vec<Closure<dyn FnMut(web_sys::Event)>>,
) -> Result<(), String> {
    let down_shared = shared.clone();
    let on_down = Closure::wrap(Box::new(move |event: web_sys::Event| {
        let Ok(ke) = event.dyn_into::<KeyboardEvent>() else {
            return;
        };
        if apply_key(&mut down_shared.borrow_mut(), &ke.code(), true) {
            ke.prevent_default();
        }
    }) as Box<dyn FnMut(web_sys::Event)>);

    let up_shared = shared;
    let on_up = Closure::wrap(Box::new(move |event: web_sys::Event| {
        let Ok(ke) = event.dyn_into::<KeyboardEvent>() else {
            return;
        };
        if apply_key(&mut up_shared.borrow_mut(), &ke.code(), false) {
            ke.prevent_default();
        }
    }) as Box<dyn FnMut(web_sys::Event)>);

    window
        .add_event_listener_with_callback("keydown", on_down.as_ref().unchecked_ref())
        .map_err(|_| "keydown listener")?;
    window
        .add_event_listener_with_callback("keyup", on_up.as_ref().unchecked_ref())
        .map_err(|_| "keyup listener")?;

    listeners.push(on_down);
    listeners.push(on_up);
    Ok(())
}

fn apply_key(state: &mut SharedInput, code: &str, down: bool) -> bool {
    let mut handled = true;
    match code {
        "KeyW" | "ArrowUp" => {
            state.key_move.1 = if down {
                -1.0
            } else {
                axis_release(state.key_move.1, -1.0)
            }
        }
        "KeyS" | "ArrowDown" => {
            state.key_move.1 = if down {
                1.0
            } else {
                axis_release(state.key_move.1, 1.0)
            }
        }
        "KeyA" | "ArrowLeft" => {
            state.key_move.0 = if down {
                -1.0
            } else {
                axis_release(state.key_move.0, -1.0)
            }
        }
        "KeyD" | "ArrowRight" => {
            state.key_move.0 = if down {
                1.0
            } else {
                axis_release(state.key_move.0, 1.0)
            }
        }
        "KeyJ" | "Space" => state.key_held[BUTTON_ATTACK] = down,
        "KeyK" => state.key_held[BUTTON_ITEM] = down,
        "KeyL" | "ShiftLeft" | "ShiftRight" => state.key_held[BUTTON_DASH] = down,
        "KeyE" => state.key_held[BUTTON_INTERACT] = down,
        "Escape" => state.key_held[BUTTON_PAUSE] = down,
        "Enter" => state.key_held[BUTTON_CONFIRM] = down,
        "KeyQ" => state.key_held[BUTTON_CYCLE] = down,
        "F1" => {
            state.debug_held[DEBUG_OVERLAY] = down;
            if down {
                state.debug_pulse[DEBUG_OVERLAY] = true;
            }
        }
        "F2" => {
            state.debug_held[DEBUG_VIEWER] = down;
            if down {
                state.debug_pulse[DEBUG_VIEWER] = true;
            }
        }
        "KeyH" => {
            state.debug_held[DEBUG_ACTION] = down;
            if down {
                state.debug_pulse[DEBUG_ACTION] = true;
            }
        }
        "F3" => {
            state.debug_held[DEBUG_MAP] = down;
            if down {
                state.debug_pulse[DEBUG_MAP] = true;
            }
        }
        "F4" => {
            state.debug_held[DEBUG_TELEPORT] = down;
            if down {
                state.debug_pulse[DEBUG_TELEPORT] = true;
            }
        }
        "KeyM" => {
            if down {
                state.minimap_pulse = true;
            }
        }
        _ => handled = false,
    }
    handled
}

fn axis_release(current: f32, direction: f32) -> f32 {
    if (current - direction).abs() < f32::EPSILON {
        0.0
    } else {
        current
    }
}
