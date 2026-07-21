//! Bottom dialogue panel with typewriter + blip.

use content::audio::sfx::SfxId;
use content::text::{self, TextId};
use engine::input::{InputState, BUTTON_ATTACK, BUTTON_INTERACT};
use engine::render::Draw;

use crate::assets::SpriteMap;
use crate::world::{World, WorldEvent};

const CHARS_PER_TICK: u16 = 2; // typewriter: reveal every 2 ticks
const PANEL_Y: f32 = 198.0;
const PANEL_H: f32 = 68.0;

pub struct DialogState {
    pub open: bool,
    pages: &'static [&'static str],
    page: usize,
    chars_shown: usize,
    timer: u16,
    blip_accum: u8,
    /// Edge latch so held Interact doesn't skip pages.
    advance_armed: bool,
}

impl DialogState {
    pub fn new() -> Self {
        Self {
            open: false,
            pages: &[],
            page: 0,
            chars_shown: 0,
            timer: 0,
            blip_accum: 0,
            advance_armed: false,
        }
    }

    pub fn open_text(&mut self, id: TextId) {
        self.pages = text::text(id);
        self.page = 0;
        self.chars_shown = 0;
        self.timer = 0;
        self.blip_accum = 0;
        self.open = !self.pages.is_empty();
        self.advance_armed = false;
    }

    pub fn update(&mut self, input: &InputState, world: &mut World) {
        if !self.open {
            return;
        }
        let page = self.pages.get(self.page).copied().unwrap_or("");
        let full = page.len();

        self.timer = self.timer.wrapping_add(1);
        if self.chars_shown < full && self.timer.is_multiple_of(CHARS_PER_TICK) {
            let next = (self.chars_shown + 1).min(full);
            let added = next - self.chars_shown;
            self.chars_shown = next;
            self.blip_accum = self.blip_accum.wrapping_add(added as u8);
            if self.blip_accum >= 2 {
                self.blip_accum = 0;
                world.push_event(WorldEvent::Sfx(SfxId::TextBlip));
            }
        }

        let attack = input.buttons[BUTTON_ATTACK].pressed;
        let interact = input.buttons[BUTTON_INTERACT].pressed;
        let advance = attack || interact;

        if !advance {
            self.advance_armed = true;
            return;
        }
        if !self.advance_armed {
            return;
        }
        self.advance_armed = false;

        if self.chars_shown < full {
            self.chars_shown = full;
            return;
        }
        if self.page + 1 < self.pages.len() {
            self.page += 1;
            self.chars_shown = 0;
            self.timer = 0;
            self.blip_accum = 0;
        } else {
            self.open = false;
        }
    }

    pub fn render(&self, d: &mut Draw, sprites: &SpriteMap) {
        if !self.open {
            return;
        }
        let _ = sprites;
        d.rect(40.0, PANEL_Y, 400.0, PANEL_H, "#202830");
        d.rect(42.0, PANEL_Y + 2.0, 396.0, PANEL_H - 4.0, "#304050");
        d.rect(40.0, PANEL_Y, 400.0, 2.0, "#6080a0");
        let page = self.pages.get(self.page).copied().unwrap_or("");
        let shown: String = page.chars().take(self.chars_shown).collect();
        // Wrap manually into up to 3 visual lines (~38 chars).
        let mut y = PANEL_Y + 12.0;
        for chunk in wrap_lines(&shown, 38).iter().take(3) {
            d.text(chunk, 56.0, y, "#f0f0e8");
            y += 14.0;
        }
        if self.chars_shown >= page.len() {
            d.text("▼", 420.0, PANEL_Y + PANEL_H - 16.0, "#e0e080");
        }
    }
}

impl Default for DialogState {
    fn default() -> Self {
        Self::new()
    }
}

fn wrap_lines(s: &str, width: usize) -> Vec<String> {
    if s.is_empty() {
        return vec![String::new()];
    }
    let mut lines = Vec::new();
    let mut cur = String::new();
    for word in s.split_whitespace() {
        if cur.is_empty() {
            cur.push_str(word);
        } else if cur.len() + 1 + word.len() <= width {
            cur.push(' ');
            cur.push_str(word);
        } else {
            lines.push(cur);
            cur = word.to_string();
        }
    }
    if !cur.is_empty() {
        lines.push(cur);
    }
    lines
}
