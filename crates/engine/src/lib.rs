pub mod atlas;
pub mod audio;
pub mod canvas;
pub mod chunks;
pub mod input;
pub mod render;
pub mod save;
pub mod time;

use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::Window;

use audio::Audio;
use canvas::Canvas;
use input::Input;
use render::Draw;

/// Platform facade: canvas, input, audio, draw. Created via [`Platform::create`].
pub struct Platform {
    pub window: Window,
    pub canvas: Canvas,
    pub draw: Draw,
    pub input: Input,
    pub audio: Audio,
}

impl Platform {
    pub fn create() -> Result<Rc<RefCell<Self>>, String> {
        console_error_panic_hook::set_once();

        let window = web_sys::window().ok_or("no window")?;
        let canvas = Canvas::new(&window)?;
        let draw = Draw::new(canvas.element())?;
        let input = Input::new(&window, canvas.element())?;
        let audio = Audio::new();

        let platform = Rc::new(RefCell::new(Self {
            window: window.clone(),
            canvas,
            draw,
            input,
            audio,
        }));

        {
            let mut p = platform.borrow_mut();
            let portrait = viewport_is_portrait(&p.window);
            p.input.set_viewport_portrait(portrait);
        }

        let resize_target = platform.clone();
        let resize_listener = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            let mut p = resize_target.borrow_mut();
            let window = p.window.clone();
            let _ = p.canvas.resize(&window);
            let portrait = viewport_is_portrait(&window);
            p.input.set_viewport_portrait(portrait);
        }) as Box<dyn FnMut(web_sys::Event)>);
        window
            .add_event_listener_with_callback("resize", resize_listener.as_ref().unchecked_ref())
            .map_err(|_| "resize listener")?;
        resize_listener.forget();

        let audio_target = platform.clone();
        let unlock = Closure::wrap(Box::new(move |_event: web_sys::Event| {
            audio_target.borrow_mut().audio.resume();
        }) as Box<dyn FnMut(web_sys::Event)>);
        for evt in ["keydown", "pointerdown", "touchstart"] {
            window
                .add_event_listener_with_callback(evt, unlock.as_ref().unchecked_ref())
                .map_err(|_| "gesture listener")?;
        }
        unlock.forget();

        Ok(platform)
    }
}

fn viewport_is_portrait(window: &Window) -> bool {
    let w = window
        .inner_width()
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(480.0);
    let h = window
        .inner_height()
        .ok()
        .and_then(|v| v.as_f64())
        .unwrap_or(270.0);
    w < h
}
