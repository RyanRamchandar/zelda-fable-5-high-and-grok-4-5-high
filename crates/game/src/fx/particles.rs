use engine::render::Draw;

use crate::math::Vec2;

const CAP: usize = 256;

#[derive(Clone, Copy, Debug)]
pub enum ParticleKind {
    Dust,
    Impact,
    Poof,
    Shimmer,
    Fountain,
}

struct Particle {
    pos: Vec2,
    vel: Vec2,
    life: u16,
    max: u16,
    size: f32,
    kind: ParticleKind,
}

pub struct Particles {
    list: Vec<Particle>,
}

impl Particles {
    pub fn new() -> Self {
        Self {
            list: Vec::with_capacity(64),
        }
    }

    pub fn count(&self) -> usize {
        self.list.len()
    }

    pub fn spawn_dust(&mut self, pos: Vec2, rng: &mut fastrand::Rng) {
        for _ in 0..3 {
            self.push(
                pos,
                Vec2::new(rng.f32() * 1.2 - 0.6, rng.f32() * 0.4 + 0.2),
                18,
                1.5,
                ParticleKind::Dust,
            );
        }
    }

    pub fn spawn_impact(&mut self, pos: Vec2, rng: &mut fastrand::Rng) {
        for _ in 0..6 {
            let a = rng.f32() * std::f32::consts::TAU;
            let sp = 0.8 + rng.f32() * 1.5;
            self.push(
                pos,
                Vec2::new(a.cos() * sp, a.sin() * sp),
                14,
                2.0,
                ParticleKind::Impact,
            );
        }
    }

    pub fn spawn_poof(&mut self, pos: Vec2, rng: &mut fastrand::Rng) {
        for _ in 0..10 {
            let a = rng.f32() * std::f32::consts::TAU;
            let sp = 0.5 + rng.f32() * 1.8;
            self.push(
                pos,
                Vec2::new(a.cos() * sp, a.sin() * sp),
                22,
                2.5,
                ParticleKind::Poof,
            );
        }
    }

    pub fn spawn_shimmer(&mut self, pos: Vec2, rng: &mut fastrand::Rng) {
        self.push(
            pos.add(Vec2::new(rng.f32() * 16.0 - 8.0, rng.f32() * 16.0 - 8.0)),
            Vec2::new(0.0, -0.4),
            12,
            1.5,
            ParticleKind::Shimmer,
        );
    }

    pub fn spawn_fountain(&mut self, pos: Vec2, rng: &mut fastrand::Rng) {
        self.push(
            pos.add(Vec2::new(rng.f32() * 12.0 - 6.0, rng.f32() * 4.0)),
            Vec2::new(rng.f32() * 0.4 - 0.2, -0.6 - rng.f32() * 0.4),
            20,
            1.5,
            ParticleKind::Fountain,
        );
    }

    fn push(&mut self, pos: Vec2, vel: Vec2, life: u16, size: f32, kind: ParticleKind) {
        if self.list.len() >= CAP {
            self.list.remove(0);
        }
        self.list.push(Particle {
            pos,
            vel,
            life,
            max: life,
            size,
            kind,
        });
    }

    pub fn update(&mut self) {
        for p in &mut self.list {
            p.pos = p.pos.add(p.vel);
            p.vel = p.vel.scale(0.92);
            p.life = p.life.saturating_sub(1);
        }
        self.list.retain(|p| p.life > 0);
    }

    pub fn render(&self, d: &mut Draw) {
        for p in &self.list {
            let t = p.life as f32 / p.max.max(1) as f32;
            let color = match p.kind {
                ParticleKind::Dust => format!("rgba(180,170,140,{t})"),
                ParticleKind::Impact => format!("rgba(255,230,120,{t})"),
                ParticleKind::Poof => format!("rgba(200,200,210,{t})"),
                ParticleKind::Shimmer => format!("rgba(180,220,255,{t})"),
                ParticleKind::Fountain => format!("rgba(120,220,200,{t})"),
            };
            d.rect(p.pos.x, p.pos.y, p.size, p.size, &color);
        }
    }
}

impl Default for Particles {
    fn default() -> Self {
        Self::new()
    }
}
