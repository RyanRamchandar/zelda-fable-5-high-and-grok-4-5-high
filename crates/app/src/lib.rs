use std::cell::RefCell;
use std::rc::Rc;

use wasm_bindgen::prelude::*;

use engine::Platform;
use game::{Game, GameEvent, SAVE_KEY};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &str);
}

#[wasm_bindgen(start)]
pub fn start() {
    let platform = match Platform::create() {
        Ok(p) => p,
        Err(e) => {
            error(&format!("platform init failed: {e}"));
            return;
        }
    };

    let saved = engine::save::load(SAVE_KEY);
    let game = Rc::new(RefCell::new(Game::from_storage_json(saved)));

    let platform_loop = platform.clone();
    let game_loop = game;
    let window = platform.borrow().window.clone();

    engine::time::run_loop(&window, move |steps| {
        let mut p = platform_loop.borrow_mut();
        let window = p.window.clone();
        p.input.poll_gamepad(&window);
        let input = p.input.snapshot();

        {
            let mut g = game_loop.borrow_mut();
            for _ in 0..steps {
                let events = g.update(&input);
                for event in events {
                    match event {
                        GameEvent::Beep => p.audio.beep(880.0, 0.08),
                        GameEvent::Save(json) => {
                            let _ = engine::save::save(SAVE_KEY, &json);
                        }
                    }
                }
            }
            g.render(&mut p.draw);
        }

        p.input.end_frame();
    });
}
