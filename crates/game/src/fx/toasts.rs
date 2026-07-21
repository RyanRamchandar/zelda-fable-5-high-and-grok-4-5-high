//! Toast queue API. 1B may restyle rendering into `game::ui` — keep [`push_toast`].

use engine::render::Draw;

const MAX_STACK: usize = 2;
const LIFE: u16 = 90;

pub struct Toast {
    pub text: &'static str,
    pub ticks: u16,
}

pub struct Toasts {
    pub queue: Vec<Toast>,
}

impl Toasts {
    pub fn new() -> Self {
        Self { queue: Vec::new() }
    }

    pub fn update(&mut self) {
        for t in &mut self.queue {
            t.ticks = t.ticks.saturating_sub(1);
        }
        self.queue.retain(|t| t.ticks > 0);
    }

    pub fn render(&self, d: &mut Draw) {
        for (i, t) in self.queue.iter().enumerate() {
            let fade = if t.ticks < 20 {
                t.ticks as f32 / 20.0
            } else if t.ticks > LIFE - 8 {
                (LIFE - t.ticks) as f32 / 8.0
            } else {
                1.0
            };
            let y = 28.0 + i as f32 * 14.0;
            let color = format!("rgba(255,240,200,{fade})");
            let x = 240.0 - t.text.len() as f32 * 2.8;
            d.text(t.text, x, y, &color);
        }
    }
}

impl Default for Toasts {
    fn default() -> Self {
        Self::new()
    }
}

pub fn push_toast(toasts: &mut Toasts, text: &'static str) {
    if toasts.queue.len() >= MAX_STACK {
        toasts.queue.remove(0);
    }
    toasts.queue.push(Toast {
        text,
        ticks: LIFE,
    });
}
