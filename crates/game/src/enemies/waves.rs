//! Arena wave director (Phase 1 scaffolding).

use content::audio::sfx::SfxId;

use crate::combat::tuning;
use crate::fx::FxKind;
use crate::math::Vec2;
use crate::world::entity::EntityData;
use crate::world::{World, WorldEvent};

use super::{ai, bat, octorok, slime};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Kind {
    Slime,
    Bat,
    Octorok,
}

#[derive(Clone, Debug)]
pub struct WaveDirector {
    pub wave: u32,
    pub lull: u16,
    pub active: bool,
    spawned_this_wave: bool,
}

impl WaveDirector {
    pub fn new() -> Self {
        Self {
            wave: 0,
            lull: 30, // short intro before wave 1
            active: true,
            spawned_this_wave: false,
        }
    }

    pub fn update(&mut self, world: &mut World) {
        if !self.active {
            return;
        }
        let alive = ai::count_alive_enemies(world);
        if self.lull > 0 {
            self.lull -= 1;
            if self.lull == 0 && !self.spawned_this_wave {
                self.wave += 1;
                self.spawn_wave(world);
                self.spawned_this_wave = true;
                world.push_event(WorldEvent::FxRequest(FxKind::Toast {
                    text: wave_toast(self.wave),
                }));
                world.push_event(WorldEvent::Sfx(SfxId::WaveCue));
            }
            return;
        }

        if alive == 0 && self.spawned_this_wave {
            // clear bonus
            let bonus = clear_bonus(world, self.wave);
            if let Some(p) = world.get_mut(world.player_id) {
                if let EntityData::Player(pd) = &mut p.data {
                    pd.rupees = pd.rupees.saturating_add(bonus);
                }
            }
            world.push_event(WorldEvent::FxRequest(FxKind::Toast { text: "CLEAR!" }));
            self.lull = tuning::WAVE_LULL;
            self.spawned_this_wave = false;
        }
    }

    fn spawn_wave(&mut self, world: &mut World) {
        let roster = roster(self.wave);
        let points = ai::edge_spawn_points(world);
        let base = (self.wave as usize).wrapping_mul(3);
        for (i, kind) in roster.into_iter().enumerate() {
            if ai::count_alive_enemies(world) >= tuning::WAVE_ALIVE_CAP {
                break;
            }
            let pos = ai::spawn_clear_of_player(world, &points);
            let jitter = points[(base + i) % points.len()];
            let spawn_at = Vec2::new((pos.x + jitter.x) * 0.5, (pos.y + jitter.y) * 0.5);
            let spawn_at = ai::spawn_clear_of_player(world, &[spawn_at, pos, jitter]);
            match kind {
                Kind::Slime => {
                    slime::spawn(world, spawn_at);
                }
                Kind::Bat => {
                    bat::spawn(world, spawn_at);
                }
                Kind::Octorok => {
                    octorok::spawn(world, spawn_at);
                }
            }
        }
    }
}

impl Default for WaveDirector {
    fn default() -> Self {
        Self::new()
    }
}

fn roster(wave: u32) -> Vec<Kind> {
    let mut list = match wave {
        0 | 1 => vec![Kind::Slime, Kind::Slime, Kind::Slime],
        2 => vec![Kind::Slime, Kind::Slime, Kind::Bat, Kind::Bat],
        3 => vec![
            Kind::Octorok,
            Kind::Octorok,
            Kind::Slime,
            Kind::Slime,
            Kind::Bat,
        ],
        n => {
            // escalate: wave 3 pattern + (n-3) extras, cap composition
            let mut v = vec![
                Kind::Octorok,
                Kind::Octorok,
                Kind::Slime,
                Kind::Slime,
                Kind::Bat,
            ];
            let extra = (n - 3).min(5) as usize;
            for i in 0..extra {
                v.push(match i % 3 {
                    0 => Kind::Slime,
                    1 => Kind::Bat,
                    _ => Kind::Octorok,
                });
            }
            v
        }
    };
    while list.len() > tuning::WAVE_ALIVE_CAP {
        list.pop();
    }
    list
}

fn wave_toast(wave: u32) -> &'static str {
    match wave {
        1 => "WAVE 1",
        2 => "WAVE 2",
        3 => "WAVE 3",
        4 => "WAVE 4",
        5 => "WAVE 5",
        6 => "WAVE 6",
        7 => "WAVE 7",
        8 => "WAVE 8",
        9 => "WAVE 9",
        _ => "WAVE N",
    }
}

fn clear_bonus(world: &World, wave: u32) -> u32 {
    let rank = world
        .get(world.player_id)
        .and_then(|p| match &p.data {
            EntityData::Player(pd) => Some(pd.style_rank),
            _ => None,
        })
        .unwrap_or(0);
    let base = 3 + wave;
    base + rank as u32 * 2
}
