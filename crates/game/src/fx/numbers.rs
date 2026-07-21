use engine::render::Draw;

use crate::math::Vec2;

struct DmgNum {
    pos: Vec2,
    base_y: f32,
    amount: i32,
    gold: bool,
    life: u16,
}

pub struct DamageNumbers {
    list: Vec<DmgNum>,
}

impl DamageNumbers {
    pub fn new() -> Self {
        Self { list: Vec::new() }
    }

    pub fn spawn(&mut self, pos: Vec2, amount: i32, gold: bool) {
        self.list.push(DmgNum {
            pos,
            base_y: pos.y,
            amount,
            gold,
            life: 40,
        });
    }

    pub fn update(&mut self) {
        for n in &mut self.list {
            let age = 40 - n.life;
            if age < 10 {
                n.pos.y = n.base_y - (age as f32 / 10.0) * 6.0;
            }
            n.life = n.life.saturating_sub(1);
        }
        self.list.retain(|n| n.life > 0);
    }

    pub fn render(&self, d: &mut Draw) {
        for n in &self.list {
            let t = n.life as f32 / 40.0;
            let color = if n.gold {
                format!("rgba(255,215,80,{t})")
            } else {
                format!("rgba(255,255,255,{t})")
            };
            d.text(&format!("{}", n.amount), n.pos.x - 3.0, n.pos.y, &color);
        }
    }
}

impl Default for DamageNumbers {
    fn default() -> Self {
        Self::new()
    }
}
