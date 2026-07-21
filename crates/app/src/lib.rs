use std::cell::RefCell;
use std::rc::Rc;

use content::audio::music::{self as content_music, TrackId};
use content::audio::sfx::{self, OscKind as ContentOsc};
use wasm_bindgen::prelude::*;

use engine::audio::{MusicTrackParams, Note, OscKind, SfxParams};
use engine::Platform;
use game::{Game, GameEvent, SAVE_KEY};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[wasm_bindgen(start)]
pub fn start() {
    log("shard: build phase5 (Gate C)");

    let platform = match Platform::create() {
        Ok(p) => p,
        Err(e) => {
            error(&format!("platform init failed: {e}"));
            return;
        }
    };

    let baked = match game::bake_assets() {
        Ok(b) => b,
        Err(e) => {
            error(&format!("atlas bake failed: {e}"));
            return;
        }
    };

    {
        let mut p = platform.borrow_mut();
        p.draw.set_atlas(baked.atlas);
    }

    let saved = engine::save::load(SAVE_KEY);
    let game = Rc::new(RefCell::new(Game::from_storage_json(
        saved,
        baked.sprites,
    )));

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
            let mut input = input;
            for step in 0..steps {
                if step > 0 {
                    // Edge pulses are one-shot per snapshot; don't re-fire on catch-up.
                    for b in input.buttons.iter_mut() {
                        b.pressed = false;
                    }
                    for b in input.debug.iter_mut() {
                        b.pressed = false;
                    }
                    input.minimap_toggle = false;
                    input.menu_tap = None;
                }
                let events = g.update(&input);
                for event in events {
                    match event {
                        GameEvent::Sfx(id) => {
                            let spec = sfx::spec(id);
                            p.audio.play(&adapt_sfx(&spec));
                        }
                        GameEvent::Save(json) => {
                            let _ = engine::save::save(SAVE_KEY, &json);
                        }
                        GameEvent::SetMuted(b) => {
                            p.audio.set_muted(b);
                        }
                        GameEvent::SetMusic(id) => {
                            p.audio.set_music(Some(adapt_track(id)));
                        }
                    }
                }
            }
            g.render(&mut p.draw);
        }

        p.audio.tick_music();
        p.input.end_frame();
    });
}

fn adapt_sfx(spec: &sfx::SfxSpec) -> SfxParams {
    SfxParams {
        osc: match spec.osc {
            ContentOsc::Square => OscKind::Square,
            ContentOsc::Triangle => OscKind::Triangle,
            ContentOsc::Saw => OscKind::Saw,
            ContentOsc::Sine => OscKind::Sine,
            ContentOsc::Noise => OscKind::Noise,
        },
        freq_start: spec.freq_start,
        freq_end: spec.freq_end,
        attack_s: spec.attack_s,
        decay_s: spec.decay_s,
        // Slight lift so cues cut through music bus (~0.35).
        gain: (spec.gain * 1.15).min(0.55),
        noise_mix: spec.noise_mix,
    }
}

fn adapt_track(id: TrackId) -> MusicTrackParams {
    let t = content_music::track(id);
    MusicTrackParams {
        bpm: t.bpm,
        loop_len: t.loop_len,
        channels: [
            adapt_notes(t.channels[0]),
            adapt_notes(t.channels[1]),
            adapt_notes(t.channels[2]),
            adapt_notes(t.channels[3]),
        ],
    }
}

fn adapt_notes(notes: &[content_music::Note]) -> Vec<Note> {
    notes
        .iter()
        .map(|n| Note {
            start: n.start,
            len: n.len,
            midi: n.midi,
            vol: n.vol,
        })
        .collect()
}
