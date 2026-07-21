use wasm_bindgen::JsCast;
use web_sys::Window;

use super::{
    SharedInput, BUTTON_ATTACK, BUTTON_DASH, BUTTON_INTERACT, BUTTON_ITEM, BUTTON_PAUSE,
};

const DEADZONE: f32 = 0.25;

pub fn poll(window: &Window, state: &mut SharedInput) {
    let navigator = window.navigator();
    let Ok(pads) = navigator.get_gamepads() else {
        state.pad_move = (0.0, 0.0);
        state.pad_held = [false; super::BUTTON_COUNT];
        return;
    };

    let mut move_x = 0.0_f32;
    let mut move_y = 0.0_f32;
    let mut any = false;
    let mut pad_buttons = [false; super::BUTTON_COUNT];

    for i in 0..pads.length() {
        let pad_val = pads.get(i);
        if pad_val.is_null() || pad_val.is_undefined() {
            continue;
        }
        let Ok(pad) = pad_val.dyn_into::<web_sys::Gamepad>() else {
            continue;
        };
        any = true;

        let axes = pad.axes();
        if axes.length() >= 2 {
            let ax = axes.get(0).as_f64().unwrap_or(0.0) as f32;
            let ay = axes.get(1).as_f64().unwrap_or(0.0) as f32;
            move_x += apply_deadzone(ax);
            move_y += apply_deadzone(ay);
        }

        let buttons = pad.buttons();
        // D-pad: 12 up, 13 down, 14 left, 15 right
        if button_pressed(&buttons, 12) {
            move_y -= 1.0;
        }
        if button_pressed(&buttons, 13) {
            move_y += 1.0;
        }
        if button_pressed(&buttons, 14) {
            move_x -= 1.0;
        }
        if button_pressed(&buttons, 15) {
            move_x += 1.0;
        }

        // Standard mapping: 0=Attack, 2=Item, 1=Dash, 3=Interact, 9=Pause
        if button_pressed(&buttons, 0) {
            pad_buttons[BUTTON_ATTACK] = true;
        }
        if button_pressed(&buttons, 2) {
            pad_buttons[BUTTON_ITEM] = true;
        }
        if button_pressed(&buttons, 1) {
            pad_buttons[BUTTON_DASH] = true;
        }
        if button_pressed(&buttons, 3) {
            pad_buttons[BUTTON_INTERACT] = true;
        }
        if button_pressed(&buttons, 9) {
            pad_buttons[BUTTON_PAUSE] = true;
        }
    }

    if !any {
        state.pad_move = (0.0, 0.0);
        state.pad_held = [false; super::BUTTON_COUNT];
        return;
    }

    let len = (move_x * move_x + move_y * move_y).sqrt();
    if len > 1.0 {
        move_x /= len;
        move_y /= len;
    }
    state.pad_move = (move_x, move_y);
    state.pad_held = pad_buttons;
}

fn apply_deadzone(v: f32) -> f32 {
    if v.abs() < DEADZONE {
        0.0
    } else {
        v
    }
}

fn button_pressed(buttons: &js_sys::Array, index: u32) -> bool {
    let Some(val) = buttons
        .get(index)
        .dyn_into::<web_sys::GamepadButton>()
        .ok()
    else {
        return false;
    };
    val.pressed()
}
