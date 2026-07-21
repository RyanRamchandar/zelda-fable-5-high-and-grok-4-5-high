mod gamepad;
mod keyboard;
mod touch;

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::closure::Closure;
use web_sys::{HtmlCanvasElement, Window};

pub use touch::{TouchOverlay, JOYSTICK_MAX_RADIUS};

pub const BUTTON_ATTACK: usize = 0;
pub const BUTTON_ITEM: usize = 1;
pub const BUTTON_DASH: usize = 2;
pub const BUTTON_INTERACT: usize = 3;
pub const BUTTON_PAUSE: usize = 4;
pub const BUTTON_CONFIRM: usize = 5;
pub const BUTTON_COUNT: usize = 6;

pub const DEBUG_OVERLAY: usize = 0;
pub const DEBUG_VIEWER: usize = 1;
pub const DEBUG_ACTION: usize = 2;
pub const DEBUG_COUNT: usize = 3;

#[derive(Clone, Copy, Debug, Default)]
pub struct Button {
    pub held: bool,
    pub pressed: bool,
}

#[derive(Clone, Debug)]
pub struct InputState {
    pub move_vec: (f32, f32),
    pub buttons: [Button; BUTTON_COUNT],
    pub debug: [Button; DEBUG_COUNT],
    pub touch_active: bool,
    pub touch_overlay: TouchOverlay,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            move_vec: (0.0, 0.0),
            buttons: [Button::default(); BUTTON_COUNT],
            debug: [Button::default(); DEBUG_COUNT],
            touch_active: false,
            touch_overlay: TouchOverlay::default(),
        }
    }
}

pub struct Input {
    shared: Rc<RefCell<SharedInput>>,
    #[allow(dead_code)]
    listeners: Vec<Closure<dyn FnMut(web_sys::Event)>>,
}

pub(crate) struct SharedInput {
    pub key_move: (f32, f32),
    pub pad_move: (f32, f32),
    pub touch_move: (f32, f32),
    pub key_held: [bool; BUTTON_COUNT],
    pub pad_held: [bool; BUTTON_COUNT],
    pub touch_held: [bool; BUTTON_COUNT],
    pub prev_held: [bool; BUTTON_COUNT],
    pub debug_held: [bool; DEBUG_COUNT],
    pub prev_debug: [bool; DEBUG_COUNT],
    /// Latched on keydown so brief F1/F2/H taps still register as pressed.
    pub debug_pulse: [bool; DEBUG_COUNT],
    pub touch: touch::TouchState,
}

impl SharedInput {
    fn new() -> Self {
        Self {
            key_move: (0.0, 0.0),
            pad_move: (0.0, 0.0),
            touch_move: (0.0, 0.0),
            key_held: [false; BUTTON_COUNT],
            pad_held: [false; BUTTON_COUNT],
            touch_held: [false; BUTTON_COUNT],
            prev_held: [false; BUTTON_COUNT],
            debug_held: [false; DEBUG_COUNT],
            prev_debug: [false; DEBUG_COUNT],
            debug_pulse: [false; DEBUG_COUNT],
            touch: touch::TouchState::new(),
        }
    }

    fn held(&self) -> [bool; BUTTON_COUNT] {
        let mut out = [false; BUTTON_COUNT];
        for ((slot, key), (pad, touch)) in out
            .iter_mut()
            .zip(self.key_held.iter())
            .zip(self.pad_held.iter().zip(self.touch_held.iter()))
        {
            *slot = *key || *pad || *touch;
        }
        out
    }

    fn merged_move(&self) -> (f32, f32) {
        let (mut x, mut y) = (0.0_f32, 0.0_f32);
        for src in [self.key_move, self.pad_move, self.touch_move] {
            x += src.0;
            y += src.1;
        }
        let len = (x * x + y * y).sqrt();
        if len > 1.0 {
            x /= len;
            y /= len;
        } else if len < 0.01 {
            x = 0.0;
            y = 0.0;
        }
        (x, y)
    }
}

impl Input {
    pub fn new(window: &Window, canvas: &HtmlCanvasElement) -> Result<Self, String> {
        let shared = Rc::new(RefCell::new(SharedInput::new()));
        let mut listeners = Vec::new();

        keyboard::attach(window, shared.clone(), &mut listeners)?;
        touch::attach(window, canvas, shared.clone(), &mut listeners)?;

        Ok(Self { shared, listeners })
    }

    pub fn poll_gamepad(&self, window: &Window) {
        gamepad::poll(window, &mut self.shared.borrow_mut());
    }

    pub fn snapshot(&mut self) -> InputState {
        let s = self.shared.borrow_mut();
        let held = s.held();
        let mut buttons = [Button::default(); BUTTON_COUNT];
        for i in 0..BUTTON_COUNT {
            buttons[i] = Button {
                held: held[i],
                pressed: held[i] && !s.prev_held[i],
            };
        }
        let mut debug = [Button::default(); DEBUG_COUNT];
        for ((slot, (&held, &pulse)), &prev) in debug
            .iter_mut()
            .zip(s.debug_held.iter().zip(s.debug_pulse.iter()))
            .zip(s.prev_debug.iter())
        {
            *slot = Button {
                held,
                pressed: pulse || (held && !prev),
            };
        }
        InputState {
            move_vec: s.merged_move(),
            buttons,
            debug,
            touch_active: s.touch.touch_active,
            touch_overlay: s.touch.overlay_geometry(),
        }
    }

    pub fn end_frame(&mut self) {
        let mut s = self.shared.borrow_mut();
        s.prev_held = s.held();
        s.prev_debug = s.debug_held;
        s.debug_pulse = [false; DEBUG_COUNT];
    }
}
