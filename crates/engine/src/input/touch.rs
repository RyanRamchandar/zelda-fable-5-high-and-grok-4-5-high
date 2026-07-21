use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, TouchEvent, Window};

use super::{
    SharedInput, BUTTON_ATTACK, BUTTON_CYCLE, BUTTON_DASH, BUTTON_INTERACT, BUTTON_ITEM,
    BUTTON_PAUSE,
};

pub const JOYSTICK_MAX_RADIUS: f32 = 24.0;
const HIT_GRACE: f32 = 4.0;

struct TouchButton {
    button: usize,
    cx: f32,
    cy: f32,
    r: f32,
}

const TOUCH_BUTTONS: &[TouchButton] = &[
    TouchButton {
        button: BUTTON_ATTACK,
        cx: 446.0,
        cy: 224.0,
        r: 20.0,
    },
    TouchButton {
        button: BUTTON_ITEM,
        cx: 404.0,
        cy: 246.0,
        r: 16.0,
    },
    TouchButton {
        button: BUTTON_DASH,
        cx: 462.0,
        cy: 184.0,
        r: 14.0,
    },
    TouchButton {
        button: BUTTON_INTERACT,
        cx: 412.0,
        cy: 196.0,
        r: 13.0,
    },
    TouchButton {
        button: BUTTON_CYCLE,
        cx: 376.0,
        cy: 218.0,
        r: 10.0,
    },
    // Below corner-minimap strip so it isn't covered by the 68×68 map hit target.
    TouchButton {
        button: BUTTON_PAUSE,
        cx: 468.0,
        cy: 78.0,
        r: 11.0,
    },
];

#[derive(Clone, Debug)]
pub struct TouchButtonGeom {
    pub button: usize,
    pub cx: f32,
    pub cy: f32,
    pub r: f32,
    pub held: bool,
}

#[derive(Clone, Debug, Default)]
pub struct TouchOverlay {
    pub joystick_origin: Option<(f32, f32)>,
    pub joystick_knob: Option<(f32, f32)>,
    pub buttons: Vec<TouchButtonGeom>,
}

#[derive(Clone)]
enum TouchRole {
    /// Left-half touch awaiting move; promotes to Joystick or menu_tap.
    PendingLeft { origin: (f32, f32) },
    Joystick { origin: (f32, f32) },
    Button(usize),
}

const JOYSTICK_PROMOTE: f32 = 8.0;

pub(crate) struct TouchState {
    pub touch_active: bool,
    roles: HashMap<i32, TouchRole>,
    joystick_origin: Option<(f32, f32)>,
    joystick_knob: Option<(f32, f32)>,
    pub menu_tap_pulse: Option<(f32, f32)>,
}

impl TouchState {
    pub fn new() -> Self {
        Self {
            touch_active: false,
            roles: HashMap::new(),
            joystick_origin: None,
            joystick_knob: None,
            menu_tap_pulse: None,
        }
    }

    pub fn overlay_geometry(&self) -> TouchOverlay {
        let mut buttons = Vec::with_capacity(TOUCH_BUTTONS.len());
        for b in TOUCH_BUTTONS {
            let held = self.roles.values().any(|r| match r {
                TouchRole::Button(idx) => *idx == b.button,
                TouchRole::Joystick { .. } | TouchRole::PendingLeft { .. } => false,
            });
            buttons.push(TouchButtonGeom {
                button: b.button,
                cx: b.cx,
                cy: b.cy,
                r: b.r,
                held,
            });
        }
        TouchOverlay {
            joystick_origin: self.joystick_origin,
            joystick_knob: self.joystick_knob,
            buttons,
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

fn nearest_button(x: f32, y: f32) -> Option<&'static TouchButton> {
    let mut best: Option<(&TouchButton, f32)> = None;
    for b in TOUCH_BUTTONS {
        let d = dist((x, y), (b.cx, b.cy));
        if d <= b.r + HIT_GRACE && best.map(|(_, bd)| d < bd).unwrap_or(true) {
            best = Some((b, d));
        }
    }
    best.map(|(b, _)| b)
}

fn handle_start(state: &mut SharedInput, te: &TouchEvent, canvas: &HtmlCanvasElement, scale: f64) {
    let touches = te.changed_touches();
    for i in 0..touches.length() {
        let Some(t) = touches.get(i) else {
            continue;
        };
        let id = t.identifier();
        let (x, y) = client_to_logical(canvas, t.client_x() as f64, t.client_y() as f64, scale);

        // Left half: defer joystick until the finger moves (menu taps).
        if x < crate::canvas::WIDTH as f32 * 0.5 {
            state
                .touch
                .roles
                .insert(id, TouchRole::PendingLeft { origin: (x, y) });
            continue;
        }

        if let Some(b) = nearest_button(x, y) {
            state.touch.roles.insert(id, TouchRole::Button(b.button));
            continue;
        }

        // Right-half unclaimed tap → menu_tap.
        state.touch.menu_tap_pulse = Some((x, y));
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
        match state.touch.roles.get(&id).cloned() {
            Some(TouchRole::PendingLeft { origin }) => {
                if dist((x, y), origin) >= JOYSTICK_PROMOTE {
                    state
                        .touch
                        .roles
                        .insert(id, TouchRole::Joystick { origin });
                    state.touch.joystick_origin = Some(origin);
                    let (kx, ky) = clamp_to_radius(origin, (x, y), JOYSTICK_MAX_RADIUS);
                    state.touch.joystick_knob = Some((kx, ky));
                }
            }
            Some(TouchRole::Joystick { origin }) => {
                let (kx, ky) = clamp_to_radius(origin, (x, y), JOYSTICK_MAX_RADIUS);
                state.touch.joystick_knob = Some((kx, ky));
            }
            _ => {}
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
            match role {
                TouchRole::PendingLeft { origin } => {
                    // Stationary left tap → menu_tap (title/pause rows).
                    state.touch.menu_tap_pulse = Some(origin);
                }
                TouchRole::Joystick { .. } => {
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
                TouchRole::Button(_) => {}
            }
        }
    }
}

fn recompute_outputs(state: &mut SharedInput) {
    state.touch_held = [false; super::BUTTON_COUNT];
    state.touch_move = (0.0, 0.0);

    for role in state.touch.roles.values() {
        match role {
            TouchRole::Button(btn) => state.touch_held[*btn] = true,
            TouchRole::Joystick { .. } | TouchRole::PendingLeft { .. } => {}
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
