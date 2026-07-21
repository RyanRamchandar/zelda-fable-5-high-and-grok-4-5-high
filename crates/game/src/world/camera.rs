//! Smooth-follow camera with soft lookahead and capped screenshake.

use crate::math::{Dir4, Vec2};
pub struct Camera {
    pub pos: Vec2,
    pub shake: Vec2,
    pub shake_ticks: u8,
    pub shake_mag: f32,
}

impl Camera {
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            shake: Vec2::ZERO,
            shake_ticks: 0,
            shake_mag: 0.0,
        }
    }

    pub fn add_shake(&mut self, mag: f32, ticks: u8) {
        let mag = mag.min(3.0);
        if mag >= self.shake_mag || ticks >= self.shake_ticks {
            self.shake_mag = mag;
            self.shake_ticks = ticks;
        }
    }

    pub fn update(
        &mut self,
        map_w_px: f32,
        map_h_px: f32,
        rng: &mut fastrand::Rng,
        target: Vec2,
        facing: Dir4,
    ) {
        let look = facing.unit().scale(12.0);
        let desired = target.add(look);
        self.pos.x += (desired.x - self.pos.x) * 0.15;
        self.pos.y += (desired.y - self.pos.y) * 0.15;

        let half_w = 240.0;
        let half_h = 135.0;
        self.pos.x = self.pos.x.clamp(half_w, (map_w_px - half_w).max(half_w));
        self.pos.y = self.pos.y.clamp(half_h, (map_h_px - half_h).max(half_h));

        if self.shake_ticks > 0 {
            let m = self.shake_mag;
            self.shake = Vec2::new((rng.f32() * 2.0 - 1.0) * m, (rng.f32() * 2.0 - 1.0) * m);
            self.shake_ticks -= 1;
            self.shake_mag *= 0.85;
            if self.shake_ticks == 0 {
                self.shake = Vec2::ZERO;
                self.shake_mag = 0.0;
            }
        } else {
            self.shake = Vec2::ZERO;
        }
    }

    /// Top-left of the view in world space (apply as `Draw::set_offset(-ox, -oy)`).
    pub fn offset(&self) -> Vec2 {
        Vec2::new(
            self.pos.x - 240.0 + self.shake.x,
            self.pos.y - 135.0 + self.shake.y,
        )
    }

    pub fn snap_to(&mut self, target: Vec2) {
        self.pos = target;
    }
}
