use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use web_sys::Window;

const FIXED_DT: f64 = 1.0 / 60.0;
const MAX_FRAME_DT: f64 = 0.1;

/// Self-rescheduling rAF loop. Invokes `frame(steps)` once per animation frame,
/// where `steps` is how many fixed 60 Hz updates to run this frame (may be 0).
pub fn run_loop(window: &Window, frame: impl FnMut(u32) + 'static) {
    let frame = Rc::new(RefCell::new(frame));
    let accumulator = Rc::new(RefCell::new(0.0_f64));
    let last_ts = Rc::new(RefCell::new(None::<f64>));

    type RafCallback = Closure<dyn FnMut(f64)>;
    let callback: Rc<RefCell<Option<RafCallback>>> = Rc::new(RefCell::new(None));
    let callback_clone = callback.clone();
    let window_for_cb = window.clone();

    let closure = Closure::wrap(Box::new(move |timestamp_ms: f64| {
        let timestamp = timestamp_ms / 1000.0;
        let mut steps = 0_u32;
        {
            let mut last = last_ts.borrow_mut();
            let mut acc = accumulator.borrow_mut();
            if let Some(prev) = *last {
                let mut dt = timestamp - prev;
                if dt > MAX_FRAME_DT {
                    dt = MAX_FRAME_DT;
                }
                *acc += dt;
            }
            *last = Some(timestamp);

            while *acc >= FIXED_DT {
                steps += 1;
                *acc -= FIXED_DT;
            }
        }

        (frame.borrow_mut())(steps);

        if let Some(cb) = callback_clone.borrow().as_ref() {
            let _ = window_for_cb.request_animation_frame(cb.as_ref().unchecked_ref());
        }
    }) as Box<dyn FnMut(f64)>);

    let _ = window.request_animation_frame(closure.as_ref().unchecked_ref());
    *callback.borrow_mut() = Some(closure);
}
