//! Region name banners (minimal — 2B skins).

use engine::render::Draw;

const FADE_IN: u16 = 20;
const HOLD: u16 = 90;
const FADE_OUT: u16 = 30;
const TOTAL: u16 = FADE_IN + HOLD + FADE_OUT;
const COOLDOWN: u64 = 1800;

pub struct BannerState {
    pub active: Option<ActiveBanner>,
    pub last_shown: [u64; 16],
}

pub struct ActiveBanner {
    pub name: &'static str,
    pub tick: u16,
}

impl BannerState {
    pub fn new() -> Self {
        Self {
            active: None,
            last_shown: [0; 16],
        }
    }

    pub fn on_region(&mut self, region: u8, name: &'static str, tick: u64) {
        let idx = region as usize;
        if idx >= self.last_shown.len() {
            return;
        }
        if tick.saturating_sub(self.last_shown[idx]) < COOLDOWN {
            return;
        }
        self.last_shown[idx] = tick;
        self.active = Some(ActiveBanner { name, tick: 0 });
    }

    pub fn update(&mut self) {
        if let Some(b) = &mut self.active {
            b.tick = b.tick.saturating_add(1);
            if b.tick >= TOTAL {
                self.active = None;
            }
        }
    }

    pub fn render(&self, d: &mut Draw) {
        let Some(b) = &self.active else {
            return;
        };
        let alpha = if b.tick < FADE_IN {
            b.tick as f32 / FADE_IN as f32
        } else if b.tick < FADE_IN + HOLD {
            1.0
        } else {
            1.0 - (b.tick - FADE_IN - HOLD) as f32 / FADE_OUT as f32
        };
        let a = (alpha * 180.0) as u8;
        let under = format!("rgba(0,0,0,{:.2})", alpha * 0.55);
        let text = format!("rgba(240,240,220,{:.2})", alpha);
        let _ = a;
        let w = (b.name.len() as f32) * 6.5 + 16.0;
        let x = (480.0 - w) * 0.5;
        d.rect(x, 18.0, w, 16.0, &under);
        d.text(b.name, x + 8.0, 30.0, &text);
    }
}

impl Default for BannerState {
    fn default() -> Self {
        Self::new()
    }
}
