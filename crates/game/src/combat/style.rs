//! Style / momentum ranks (economy only — never modifies damage).

use crate::combat::tuning;
use crate::fx::FxKind;
use crate::world::entity::PlayerData;
use crate::world::WorldEvent;
use content::audio::sfx::SfxId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StyleVerb {
    Slash = 0,
    Finisher = 1,
    ChargeSpin = 2,
    DashThrough = 3,
    PerfectBlock = 4,
    /// Reserved for Phase 2 boomerang.
    #[allow(dead_code)]
    BoomerangStun = 5,
}

impl StyleVerb {
    pub fn index(self) -> usize {
        self as usize
    }

    pub fn base_points(self) -> f32 {
        match self {
            Self::Slash => 6.0,
            Self::Finisher => 14.0,
            Self::ChargeSpin => 16.0,
            Self::DashThrough => 10.0,
            Self::PerfectBlock => 12.0,
            Self::BoomerangStun => 10.0,
        }
    }
}

pub fn rank_from_points(p: f32) -> u8 {
    if p >= tuning::STYLE_S {
        4
    } else if p >= tuning::STYLE_A {
        3
    } else if p >= tuning::STYLE_B {
        2
    } else if p >= tuning::STYLE_C {
        1
    } else {
        0
    }
}

pub fn rank_letter(rank: u8) -> &'static str {
    match rank {
        0 => "D",
        1 => "C",
        2 => "B",
        3 => "A",
        _ => "S",
    }
}

fn rank_floor(rank: u8) -> f32 {
    match rank {
        0 => 0.0,
        1 => tuning::STYLE_C,
        2 => tuning::STYLE_B,
        3 => tuning::STYLE_A,
        _ => tuning::STYLE_S,
    }
}

/// Apply a style verb; returns events to push after the player borrow ends.
pub fn apply_verb(pd: &mut PlayerData, verb: StyleVerb) -> Vec<WorldEvent> {
    let idx = verb.index();
    let cd = pd.verb_cooldowns[idx];
    let decay = if cd == 0 {
        1.0
    } else {
        (1.0 - (cd as f32 / tuning::STYLE_VERB_WINDOW as f32)).clamp(0.25, 1.0)
    };
    pd.style_points = (pd.style_points + verb.base_points() * decay).min(200.0);
    pd.verb_cooldowns[idx] = tuning::STYLE_VERB_WINDOW;
    pd.combat_timer = tuning::STYLE_OUT_OF_COMBAT;
    pd.out_of_combat = 0;
    sync_rank(pd)
}

pub fn on_player_damaged(pd: &mut PlayerData) -> Vec<WorldEvent> {
    pd.no_dmg_streak = 0;
    let mut ev = Vec::new();
    if pd.style_rank > 0 {
        pd.style_rank -= 1;
        pd.style_points = rank_floor(pd.style_rank);
        pd.style_pulse = 12;
        ev.push(WorldEvent::FxRequest(FxKind::Toast {
            text: "RANK DOWN",
        }));
        ev.push(WorldEvent::Sfx(SfxId::RankDown));
    }
    ev
}

fn sync_rank(pd: &mut PlayerData) -> Vec<WorldEvent> {
    let next = rank_from_points(pd.style_points);
    if next == pd.style_rank {
        return Vec::new();
    }
    let up = next > pd.style_rank;
    pd.style_rank = next;
    pd.style_pulse = 14;
    vec![
        WorldEvent::FxRequest(FxKind::Toast {
            text: if up { "RANK UP" } else { "RANK DOWN" },
        }),
        WorldEvent::Sfx(if up {
            SfxId::RankUp
        } else {
            SfxId::RankDown
        }),
    ]
}

pub fn tick(pd: &mut PlayerData) -> Vec<WorldEvent> {
    for cd in &mut pd.verb_cooldowns {
        *cd = cd.saturating_sub(1);
    }
    if pd.style_pulse > 0 {
        pd.style_pulse -= 1;
    }

    if pd.combat_timer > 0 {
        pd.combat_timer -= 1;
        pd.out_of_combat = 0;
        pd.no_dmg_streak = pd.no_dmg_streak.saturating_add(1);
        if pd.no_dmg_streak > 0 && pd.no_dmg_streak.is_multiple_of(tuning::STYLE_STREAK_INTERVAL)
        {
            pd.style_points = (pd.style_points + 1.0).min(200.0);
            return sync_rank(pd);
        }
    } else {
        pd.out_of_combat = pd.out_of_combat.saturating_add(1);
        if pd.out_of_combat >= tuning::STYLE_OUT_OF_COMBAT
            && pd
                .out_of_combat
                .is_multiple_of(tuning::STYLE_DECAY_INTERVAL)
        {
            pd.style_points = (pd.style_points - 1.0).max(0.0);
            return sync_rank(pd);
        }
    }
    Vec::new()
}

pub fn dash_energy_cost(pd: &PlayerData) -> f32 {
    if pd.style_rank >= 3 {
        tuning::DASH_ENERGY_A_PLUS
    } else {
        tuning::DASH_ENERGY
    }
}

pub fn energy_regen_mult(pd: &PlayerData) -> f32 {
    if pd.style_rank >= 2 {
        1.5
    } else {
        1.0
    }
}

pub fn s_rank_bonus_drops(pd: &PlayerData) -> bool {
    pd.style_rank >= 4
}
