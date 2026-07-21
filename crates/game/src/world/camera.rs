//! Smooth-follow camera with soft dead-zone, eased lookahead, capped screenshake.

use crate::math::{Dir4, Vec2};

/// Soft dead-zone half extents (px). Tuned ±30% from brief 24×16.
const DEAD_X: f32 = 24.0;
const DEAD_Y: f32 = 16.0;
const LOOKAHEAD: f32 = 16.0;
const LOOK_EASE: f32 = 0.08;
const FOLLOW: f32 = 0.15;

pub struct Camera {
    pub pos: Vec2,
    pub shake: Vec2,
    pub shake_ticks: u8,
    pub shake_mag: f32,
    look: Vec2,
    /// Optional world-space clamp rect (min, max) for dungeon rooms.
    bounds: Option<(Vec2, Vec2)>,
}

impl Camera {
    pub fn new(pos: Vec2) -> Self {
        Self {
            pos,
            shake: Vec2::ZERO,
            shake_ticks: 0,
            shake_mag: 0.0,
            look: Vec2::ZERO,
            bounds: None,
        }
    }

    pub fn set_bounds(&mut self, bounds: Option<(Vec2, Vec2)>) {
        self.bounds = bounds;
    }

    pub fn clear_bounds(&mut self) {
        self.bounds = None;
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
        let want_look = facing.unit().scale(LOOKAHEAD);
        self.look.x += (want_look.x - self.look.x) * LOOK_EASE;
        self.look.y += (want_look.y - self.look.y) * LOOK_EASE;

        // Dead-zone: only chase when target leaves box around camera center.
        let mut desired = self.pos;
        let dx = target.x + self.look.x - self.pos.x;
        let dy = target.y + self.look.y - self.pos.y;
        if dx.abs() > DEAD_X {
            desired.x = target.x + self.look.x - dx.signum() * DEAD_X;
        }
        if dy.abs() > DEAD_Y {
            desired.y = target.y + self.look.y - dy.signum() * DEAD_Y;
        }

        self.pos.x += (desired.x - self.pos.x) * FOLLOW;
        self.pos.y += (desired.y - self.pos.y) * FOLLOW;

        let half_w = 240.0;
        let half_h = 135.0;
        if let Some((min, max)) = self.bounds {
            let rw = max.x - min.x;
            let rh = max.y - min.y;
            if rw < 480.0 {
                self.pos.x = (min.x + max.x) * 0.5;
            } else {
                self.pos.x = self
                    .pos
                    .x
                    .clamp(min.x + half_w, (max.x - half_w).max(min.x + half_w));
            }
            if rh < 270.0 {
                self.pos.y = (min.y + max.y) * 0.5;
            } else {
                self.pos.y = self
                    .pos
                    .y
                    .clamp(min.y + half_h, (max.y - half_h).max(min.y + half_h));
            }
        } else {
            self.pos.x = self.pos.x.clamp(half_w, (map_w_px - half_w).max(half_w));
            self.pos.y = self.pos.y.clamp(half_h, (map_h_px - half_h).max(half_h));
        }

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
        self.look = Vec2::ZERO;
    }
}
