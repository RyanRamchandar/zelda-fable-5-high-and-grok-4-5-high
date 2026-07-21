//! F2 sprite-motion viewer — pauses world, cycles sheets at gameplay timing.

use content::art::SpriteBake;
use engine::input::{InputState, BUTTON_ATTACK, DEBUG_VIEWER};
use engine::render::Draw;

use crate::assets::{self, SpriteMap};

pub struct SpriteViewer {
    pub active: bool,
    pub index: usize,
    pub frame: u16,
    pub timer: u16,
    /// ticks per frame advance (Attack cycles)
    pub rate: u16,
    pub move_cool: u16,
    pub entries: Vec<SpriteBake>,
}

impl SpriteViewer {
    pub fn new() -> Self {
        Self {
            active: false,
            index: 0,
            frame: 0,
            timer: 0,
            rate: 8,
            move_cool: 0,
            entries: assets::viewer_entries(),
        }
    }

    pub fn update(&mut self, input: &InputState) {
        if input.debug[DEBUG_VIEWER].pressed {
            self.active = !self.active;
            self.frame = 0;
            self.timer = 0;
        }
        if !self.active || self.entries.is_empty() {
            return;
        }

        if self.move_cool > 0 {
            self.move_cool -= 1;
        } else if input.move_vec.0 > 0.7 {
            self.index = (self.index + 1) % self.entries.len();
            self.frame = 0;
            self.move_cool = 14;
        } else if input.move_vec.0 < -0.7 {
            self.index = if self.index == 0 {
                self.entries.len() - 1
            } else {
                self.index - 1
            };
            self.frame = 0;
            self.move_cool = 14;
        }

        self.timer = self.timer.wrapping_add(1);
        if self.timer >= self.rate {
            self.timer = 0;
            let frames = self.entries[self.index].def.frames.max(1);
            self.frame = (self.frame + 1) % frames as u16;
        }

        if input.buttons[BUTTON_ATTACK].pressed {
            self.rate = match self.rate {
                4 => 8,
                8 => 12,
                12 => 16,
                _ => 4,
            };
        }
    }

    pub fn render(&self, d: &mut Draw, sprites: &SpriteMap) {
        if !self.active || self.entries.is_empty() {
            return;
        }
        d.clear("#0a0c12");
        d.set_offset(0.0, 0.0);
        let entry = self.entries[self.index];
        let handle = sprites.must(entry.key);
        d.text(
            &format!(
                "F2 {}  f{}/{}  rate{}",
                entry.key,
                self.frame,
                entry.def.frames.saturating_sub(1),
                self.rate
            ),
            8.0,
            12.0,
            "#c0ffc0",
        );
        d.text("move:sheets  J:rate  F2:exit", 8.0, 24.0, "#808080");

        if let Some(fa) = sprites.get("floor_a") {
            for i in 0..4 {
                d.sprite(fa, 0, 40.0 + i as f32 * 16.0, 60.0, false);
            }
        }
        if let Some(fb) = sprites.get("floor_b") {
            for i in 0..4 {
                d.sprite(fb, 0, 40.0 + i as f32 * 16.0, 76.0, false);
            }
        }

        let y1 = 100.0;
        d.sprite(handle, self.frame as u32, 80.0, y1, false);
        d.text("1x", 80.0, y1 + entry.def.h as f32 + 12.0, "#a0a0a0");

        d.sprite_scaled(handle, self.frame as u32, 180.0, 80.0, 3.0, false);
        d.text(
            "3x",
            180.0,
            80.0 + entry.def.h as f32 * 3.0 + 12.0,
            "#a0a0a0",
        );

        d.sprite_scaled(handle, self.frame as u32, 320.0, 80.0, 3.0, true);
        d.text(
            "3x flip",
            320.0,
            80.0 + entry.def.h as f32 * 3.0 + 12.0,
            "#a0a0a0",
        );
    }
}

impl Default for SpriteViewer {
    fn default() -> Self {
        Self::new()
    }
}
