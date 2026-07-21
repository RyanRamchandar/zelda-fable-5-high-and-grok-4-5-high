mod save_data;

pub use save_data::{SaveGame, SAVE_KEY};

use engine::canvas::{HEIGHT, WIDTH};
use engine::input::{InputState, BUTTON_ATTACK};
use engine::render::Draw;

const PLAYER_W: f32 = 16.0;
const PLAYER_H: f32 = 24.0;
const MOVE_SPEED: f32 = 1.5;
const SAVE_INTERVAL_TICKS: u32 = 60;

#[derive(Clone, Debug)]
pub enum GameEvent {
    Beep,
    Save(String),
}

pub struct Game {
    pub x: f32,
    pub y: f32,
    ticks: u32,
    touch_active: bool,
    touch_overlay: engine::input::TouchOverlay,
}

impl Game {
    pub fn new(save: SaveGame) -> Self {
        Self {
            x: save.x,
            y: save.y,
            ticks: 0,
            touch_active: false,
            touch_overlay: engine::input::TouchOverlay::default(),
        }
    }

    pub fn from_storage_json(json: Option<String>) -> Self {
        let save = match json {
            Some(s) => SaveGame::from_json(&s),
            None => SaveGame::default_spawn(),
        };
        Self::new(save)
    }

    pub fn update(&mut self, input: &InputState) -> Vec<GameEvent> {
        self.touch_active = input.touch_active;
        self.touch_overlay = input.touch_overlay.clone();

        let (mx, my) = input.move_vec;
        let len = (mx * mx + my * my).sqrt();
        let (nx, ny) = if len > 0.01 {
            (mx / len, my / len)
        } else {
            (0.0, 0.0)
        };

        self.x += nx * MOVE_SPEED;
        self.y += ny * MOVE_SPEED;
        self.x = self.x.clamp(0.0, WIDTH as f32 - PLAYER_W);
        self.y = self.y.clamp(0.0, HEIGHT as f32 - PLAYER_H);

        let mut events = Vec::new();
        if input.buttons[BUTTON_ATTACK].pressed {
            events.push(GameEvent::Beep);
        }

        self.ticks = self.ticks.wrapping_add(1);
        if self.ticks.is_multiple_of(SAVE_INTERVAL_TICKS) {
            let save = SaveGame {
                x: self.x,
                y: self.y,
            };
            if let Some(json) = save.to_json() {
                events.push(GameEvent::Save(json));
            }
        }

        events
    }

    pub fn render(&self, d: &mut Draw) {
        d.clear("#1a3b2a");
        d.rect(self.x, self.y, PLAYER_W, PLAYER_H, "#ffffff");
        d.text("Shard of the Triforce — Phase 0", 8.0, 14.0, "#c8e6c9");

        if self.touch_active {
            let o = &self.touch_overlay;
            if let Some((ox, oy)) = o.joystick_origin {
                d.circle(ox, oy, engine::input::JOYSTICK_MAX_RADIUS, "rgba(255,255,255,0.25)");
            }
            if let Some((kx, ky)) = o.joystick_knob {
                d.circle(kx, ky, 8.0, "rgba(255,255,255,0.45)");
            }
            d.circle(
                o.attack_pos.0,
                o.attack_pos.1,
                o.button_radius,
                "rgba(255,80,80,0.35)",
            );
            d.circle(
                o.dash_pos.0,
                o.dash_pos.1,
                o.button_radius,
                "rgba(80,160,255,0.35)",
            );
        }
    }
}
