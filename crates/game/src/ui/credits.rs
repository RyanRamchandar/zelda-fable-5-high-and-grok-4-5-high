//! Act 1 credits — auto-scroll; R/Confirm or hold Attack to skip.

use content::text::{self, TextId};
use engine::input::{InputState, BUTTON_ATTACK, BUTTON_CONFIRM};
use engine::render::Draw;

const SCROLL_SPEED: f32 = 0.4;
const SKIP_HOLD: u16 = 60;

pub struct CreditsState {
    pub active: bool,
    pub scroll: f32,
    pub hold: u16,
}

impl CreditsState {
    pub fn new() -> Self {
        Self {
            active: false,
            scroll: 0.0,
            hold: 0,
        }
    }

    pub fn begin(&mut self) {
        self.active = true;
        self.scroll = 0.0;
        self.hold = 0;
    }

    /// Returns true when credits should end (skip or finished).
    pub fn update(&mut self, input: &InputState) -> bool {
        if !self.active {
            return false;
        }
        self.scroll += SCROLL_SPEED;
        if input.buttons[BUTTON_ATTACK].held {
            self.hold = self.hold.saturating_add(1);
        } else {
            self.hold = 0;
        }
        let confirm_skip = input.buttons[BUTTON_CONFIRM].pressed;
        if confirm_skip || self.hold >= SKIP_HOLD || self.scroll > 420.0 {
            self.active = false;
            return true;
        }
        false
    }

    pub fn draw(&self, d: &mut Draw) {
        if !self.active {
            return;
        }
        d.rect(0.0, 0.0, 480.0, 270.0, "#0a0a12");
        let lines = content::text::credits_lines();
        let mut y = 280.0 - self.scroll;
        for line in lines {
            if y > -16.0 && y < 280.0 {
                let color = if line.is_empty() {
                    "#0a0a12"
                } else if line.starts_with("ACT") || line.starts_with("SHARD") {
                    "#e8d080"
                } else {
                    "#c8c0b0"
                };
                d.text(line, 140.0, y, color);
            }
            y += 16.0;
        }
        d.text(text::line(TextId::CreditsSkipHint), 140.0, 252.0, "#606070");
    }
}

impl Default for CreditsState {
    fn default() -> Self {
        Self::new()
    }
}
