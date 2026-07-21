use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, TouchEvent, Window};

use super::{SharedInput, BUTTON_ATTACK, BUTTON_DASH};

pub const JOYSTICK_MAX_RADIUS: f32 = 24.0;
const BUTTON_RADIUS: f32 = 18.0;
const ATTACK_POS: (f32, f32) = (430.0, 230.0);
const DASH_POS: (f32, f32) = (400.0, 250.0);

#[derive(Clone, Debug, Default)]
pub struct TouchOverlay {
    pub joystick_origin: Option<(f32, f32)>,
    pub joystick_knob: Option<(f32, f32)>,
    pub attack_pos: (f32, f32),
    pub dash_pos: (f32, f32),
    pub button_radius: f32,
}

enum TouchRole {
    Joystick { origin: (f32, f32) },
    Attack,
    Dash,
}

pub(crate) struct TouchState {
    pub touch_active: bool,
    roles: HashMap<i32, TouchRole>,
    joystick_origin: Option<(f32, f32)>,
    joystick_knob: Option<(f32, f32)>,
}

impl TouchState {
    pub fn new() -> Self {
        Self {
            touch_active: false,
            roles: HashMap::new(),
            joystick_origin: None,
            joystick_knob: None,
        }
    }

    pub fn overlay_geometry(&self) -> TouchOverlay {
        TouchOverlay {
            joystick_origin: self.joystick_origin,
            joystick_knob: self.joystick_knob,
            attack_pos: ATTACK_POS,
            dash_pos: DASH_POS,
            button_radius: BUTTON_RADIUS,
        }
    }
}

pub fn attach(
    window: &Window,
    canvas: &HtmlCanvasElement,
    shared: Rc<RefCell<SharedInput>>,
    listeners: &mut Vec<Closure<dyn FnMut(web_sys::Event)>>,
) -> Result<(), String> {
    let canvas_el = canvas.clone();

    let on_start = make_handler(shared.clone(), canvas_el.clone(), TouchPhase::Start);
    let on_move = make_handler(shared.clone(), canvas_el.clone(), TouchPhase::Move);
    let on_end = make_handler(shared.clone(), canvas_el.clone(), TouchPhase::End);
    let on_cancel = make_handler(shared, canvas_el.clone(), TouchPhase::End);

    for (name, closure) in [
        ("touchstart", &on_start),
        ("touchmove", &on_move),
        ("touchend", &on_end),
        ("touchcancel", &on_cancel),
    ] {
        canvas_el
            .add_event_listener_with_callback(name, closure.as_ref().unchecked_ref())
            .map_err(|_| format!("{name} listener"))?;
    }

    window
        .add_event_listener_with_callback("touchend", on_end.as_ref().unchecked_ref())
        .map_err(|_| "window touchend")?;

    listeners.push(on_start);
    listeners.push(on_move);
    listeners.push(on_end);
    listeners.push(on_cancel);
    Ok(())
}

#[derive(Clone, Copy)]
enum TouchPhase {
    Start,
    Move,
    End,
}

fn make_handler(
    shared: Rc<RefCell<SharedInput>>,
    canvas_el: HtmlCanvasElement,
    phase: TouchPhase,
) -> Closure<dyn FnMut(web_sys::Event)> {
    Closure::wrap(Box::new(move |event: web_sys::Event| {
        let Ok(te) = event.dyn_into::<TouchEvent>() else {
            return;
        };
        te.prevent_default();
        let mut state = shared.borrow_mut();
        state.touch.touch_active = true;

        let scale = canvas_css_scale(&canvas_el);
        match phase {
            TouchPhase::Start => handle_start(&mut state, &te, &canvas_el, scale),
            TouchPhase::Move => handle_move(&mut state, &te, &canvas_el, scale),
            TouchPhase::End => handle_end(&mut state, &te),
        }
        recompute_outputs(&mut state);
    }) as Box<dyn FnMut(web_sys::Event)>)
}

fn canvas_css_scale(canvas: &HtmlCanvasElement) -> f64 {
    let rect = canvas.get_bounding_client_rect();
    let w = rect.width();
    if w <= 0.0 {
        1.0
    } else {
        w / crate::canvas::WIDTH
    }
}

fn client_to_logical(
    canvas: &HtmlCanvasElement,
    client_x: f64,
    client_y: f64,
    scale: f64,
) -> (f32, f32) {
    let rect = canvas.get_bounding_client_rect();
    let x = ((client_x - rect.left()) / scale) as f32;
    let y = ((client_y - rect.top()) / scale) as f32;
    (x, y)
}

fn handle_start(state: &mut SharedInput, te: &TouchEvent, canvas: &HtmlCanvasElement, scale: f64) {
    let touches = te.changed_touches();
    for i in 0..touches.length() {
        let Some(t) = touches.get(i) else {
            continue;
        };
        let id = t.identifier();
        let (x, y) = client_to_logical(canvas, t.client_x() as f64, t.client_y() as f64, scale);

        if dist((x, y), ATTACK_POS) <= BUTTON_RADIUS {
            state.touch.roles.insert(id, TouchRole::Attack);
            continue;
        }
        if dist((x, y), DASH_POS) <= BUTTON_RADIUS {
            state.touch.roles.insert(id, TouchRole::Dash);
            continue;
        }
        if x < crate::canvas::WIDTH as f32 * 0.5 {
            state
                .touch
                .roles
                .insert(id, TouchRole::Joystick { origin: (x, y) });
            state.touch.joystick_origin = Some((x, y));
            state.touch.joystick_knob = Some((x, y));
        }
    }
}

fn handle_move(state: &mut SharedInput, te: &TouchEvent, canvas: &HtmlCanvasElement, scale: f64) {
    let touches = te.changed_touches();
    for i in 0..touches.length() {
        let Some(t) = touches.get(i) else {
            continue;
        };
        let id = t.identifier();
        let (x, y) = client_to_logical(canvas, t.client_x() as f64, t.client_y() as f64, scale);
        if let Some(TouchRole::Joystick { origin }) = state.touch.roles.get(&id) {
            let origin = *origin;
            let (kx, ky) = clamp_to_radius(origin, (x, y), JOYSTICK_MAX_RADIUS);
            state.touch.joystick_knob = Some((kx, ky));
        }
    }
}

fn handle_end(state: &mut SharedInput, te: &TouchEvent) {
    let touches = te.changed_touches();
    for i in 0..touches.length() {
        let Some(t) = touches.get(i) else {
            continue;
        };
        let id = t.identifier();
        if let Some(role) = state.touch.roles.remove(&id) {
            if matches!(role, TouchRole::Joystick { .. }) {
                state.touch.joystick_origin = None;
                state.touch.joystick_knob = None;
                for role in state.touch.roles.values() {
                    if let TouchRole::Joystick { origin } = role {
                        state.touch.joystick_origin = Some(*origin);
                        state.touch.joystick_knob = Some(*origin);
                        break;
                    }
                }
            }
        }
    }
}

fn recompute_outputs(state: &mut SharedInput) {
    state.touch_held = [false; super::BUTTON_COUNT];
    state.touch_move = (0.0, 0.0);

    for role in state.touch.roles.values() {
        match role {
            TouchRole::Attack => state.touch_held[BUTTON_ATTACK] = true,
            TouchRole::Dash => state.touch_held[BUTTON_DASH] = true,
            TouchRole::Joystick { .. } => {}
        }
    }

    if let (Some(origin), Some(knob)) = (state.touch.joystick_origin, state.touch.joystick_knob) {
        let dx = (knob.0 - origin.0) / JOYSTICK_MAX_RADIUS;
        let dy = (knob.1 - origin.1) / JOYSTICK_MAX_RADIUS;
        let len = (dx * dx + dy * dy).sqrt();
        if len > 1.0 {
            state.touch_move = (dx / len, dy / len);
        } else if len > 0.05 {
            state.touch_move = (dx, dy);
        }
    }
}

fn dist(a: (f32, f32), b: (f32, f32)) -> f32 {
    let dx = a.0 - b.0;
    let dy = a.1 - b.1;
    (dx * dx + dy * dy).sqrt()
}

fn clamp_to_radius(origin: (f32, f32), point: (f32, f32), radius: f32) -> (f32, f32) {
    let dx = point.0 - origin.0;
    let dy = point.1 - origin.1;
    let len = (dx * dx + dy * dy).sqrt();
    if len <= radius || len < f32::EPSILON {
        (point.0, point.1)
    } else {
        (
            origin.0 + dx / len * radius,
            origin.1 + dy / len * radius,
        )
    }
}
