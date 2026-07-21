//! Particles, damage numbers, slash arcs, toasts.

mod numbers;
mod particles;
mod toasts;

use engine::render::Draw;

use crate::math::{Dir4, Vec2};
use crate::world::World;

pub use numbers::DamageNumbers;
pub use particles::Particles;
pub use toasts::{push_toast, Toasts};

#[derive(Clone, Debug)]
pub enum FxKind {
    Dust {
        pos: Vec2,
    },
    Impact {
        pos: Vec2,
    },
    KillPoof {
        pos: Vec2,
    },
    ChargeShimmer {
        pos: Vec2,
    },
    FountainSparkle {
        pos: Vec2,
    },
    DamageNumber {
        pos: Vec2,
        amount: i32,
        gold: bool,
    },
    SlashArc {
        pos: Vec2,
        facing: Dir4,
        spin: bool,
        life: u8,
    },
    Toast {
        text: &'static str,
    },
    BlockSpark {
        pos: Vec2,
    },
}

pub struct FxState {
    pub particles: Particles,
    pub numbers: DamageNumbers,
    pub toasts: Toasts,
    pub arcs: Vec<SlashArc>,
}

pub struct SlashArc {
    pub pos: Vec2,
    pub facing: Dir4,
    pub spin: bool,
    pub life: u8,
    pub max: u8,
}

impl FxState {
    pub fn new() -> Self {
        Self {
            particles: Particles::new(),
            numbers: DamageNumbers::new(),
            toasts: Toasts::new(),
            arcs: Vec::new(),
        }
    }

    pub fn handle(&mut self, kind: FxKind, rng: &mut fastrand::Rng) {
        match kind {
            FxKind::Dust { pos } => self.particles.spawn_dust(pos, rng),
            FxKind::Impact { pos } => self.particles.spawn_impact(pos, rng),
            FxKind::KillPoof { pos } => self.particles.spawn_poof(pos, rng),
            FxKind::ChargeShimmer { pos } => self.particles.spawn_shimmer(pos, rng),
            FxKind::FountainSparkle { pos } => self.particles.spawn_fountain(pos, rng),
            FxKind::DamageNumber { pos, amount, gold } => {
                self.numbers.spawn(pos, amount, gold);
            }
            FxKind::SlashArc {
                pos,
                facing,
                spin,
                life,
            } => {
                self.arcs.push(SlashArc {
                    pos,
                    facing,
                    spin,
                    life,
                    max: life,
                });
            }
            FxKind::Toast { text } => push_toast(&mut self.toasts, text),
            FxKind::BlockSpark { pos } => self.particles.spawn_impact(pos, rng),
        }
    }

    pub fn update(&mut self) {
        self.particles.update();
        self.numbers.update();
        self.toasts.update();
        for arc in &mut self.arcs {
            arc.life = arc.life.saturating_sub(1);
        }
        self.arcs.retain(|a| a.life > 0);
    }

    pub fn render_world(&self, d: &mut Draw) {
        self.particles.render(d);
        self.numbers.render(d);
        for arc in &self.arcs {
            draw_arc(d, arc);
        }
    }

    pub fn render_screen(&self, d: &mut Draw) {
        self.toasts.render(d);
    }

    pub fn particle_count(&self) -> usize {
        self.particles.count()
    }
}

impl Default for FxState {
    fn default() -> Self {
        Self::new()
    }
}

fn draw_arc(d: &mut Draw, arc: &SlashArc) {
    let t = arc.life as f32 / arc.max.max(1) as f32;
    let alpha = (t * 0.85).clamp(0.15, 0.85);
    let color = format!("rgba(220,230,255,{alpha})");
    let r = if arc.spin { 28.0 } else { 18.0 };
    if arc.spin {
        let segments = 12;
        for i in 0..segments {
            let a0 = std::f32::consts::TAU * (i as f32) / segments as f32;
            let a1 = std::f32::consts::TAU * ((i + 1) as f32) / segments as f32;
            d.line(
                arc.pos.x + a0.cos() * r,
                arc.pos.y + a0.sin() * r,
                arc.pos.x + a1.cos() * r,
                arc.pos.y + a1.sin() * r,
                2.0,
                &color,
            );
        }
    } else {
        let base = match arc.facing {
            Dir4::Right => 0.0,
            Dir4::Down => std::f32::consts::FRAC_PI_2,
            Dir4::Left => std::f32::consts::PI,
            Dir4::Up => -std::f32::consts::FRAC_PI_2,
        };
        let start = base - std::f32::consts::FRAC_PI_4;
        let segments = 4;
        for i in 0..segments {
            let a0 = start + (std::f32::consts::FRAC_PI_2) * (i as f32) / segments as f32;
            let a1 = start + (std::f32::consts::FRAC_PI_2) * ((i + 1) as f32) / segments as f32;
            d.line(
                arc.pos.x + a0.cos() * r,
                arc.pos.y + a0.sin() * r,
                arc.pos.x + a1.cos() * r,
                arc.pos.y + a1.sin() * r,
                2.5,
                &color,
            );
        }
    }
}

pub fn update(_world: &mut World, fx: &mut FxState) {
    fx.update();
}
