//! Wind crystal prime / gale / core timers for Granite Warden.

use content::audio::sfx::SfxId;

use crate::combat::tuning;
use crate::fx::FxKind;
use crate::math::Vec2;
use crate::world::entity::{EntityData, EntityId, EntityKind, WardenAttack, WardenPhase};
use crate::world::WorldEvent;
use crate::Game;

use super::granite_warden::{arena_cx, arena_cy, perch_e, perch_w};

pub(crate) fn update_crystals(game: &mut Game, crystals: [EntityId; 2], warden: EntityId) {
    let phase = game
        .world
        .get(warden)
        .and_then(|e| match &e.data {
            EntityData::GraniteWarden(d) => Some(d.phase),
            _ => None,
        })
        .unwrap_or(WardenPhase::One);

    // Boomerang prime (throw_id dedupe).
    let boom_hits: Vec<(u32, Vec2)> = game
        .world
        .alive_ids()
        .into_iter()
        .filter_map(|id| {
            let e = game.world.get(id)?;
            if e.kind != EntityKind::Boomerang {
                return None;
            }
            let EntityData::Boomerang(b) = &e.data else {
                return None;
            };
            Some((b.throw_id, e.center()))
        })
        .collect();

    for cid in crystals {
        let (mut perch, mut primed, seen, mut orbit) = {
            let Some(e) = game.world.get(cid) else {
                continue;
            };
            let EntityData::WindCrystal(d) = &e.data else {
                continue;
            };
            (d.perch, d.primed, d.throw_id_seen, d.orbit_angle)
        };
        if primed > 0 {
            primed = primed.saturating_sub(1);
        }
        if matches!(phase, WardenPhase::Two) {
            orbit += 0.012;
        }
        let pos = match phase {
            WardenPhase::One | WardenPhase::Three => {
                let p = if perch == 0 { perch_w() } else { perch_e() };
                Vec2::new(p.x - 8.0, p.y - 8.0)
            }
            WardenPhase::Two => {
                let r = 88.0;
                Vec2::new(
                    arena_cx() + orbit.cos() * r - 8.0,
                    arena_cy() + orbit.sin() * r * 0.55 - 8.0,
                )
            }
        };
        let cpos = Vec2::new(pos.x + 8.0, pos.y + 8.0);
        let mut prime_tid = None;
        for (tid, bpos) in &boom_hits {
            if *tid == seen {
                continue;
            }
            if bpos.sub(cpos).len() < 14.0 {
                prime_tid = Some(*tid);
                break;
            }
        }
        let mut new_seen = seen;
        if let Some(tid) = prime_tid {
            let window = match phase {
                WardenPhase::One => tuning::WARDEN_PRIME_WINDOW,
                _ => tuning::WARDEN_PRIME_WINDOW_P2,
            };
            primed = window;
            new_seen = tid;
            game.world
                .push_event(WorldEvent::Sfx(SfxId::CrystalPrime));
            game.world
                .push_event(WorldEvent::FxRequest(FxKind::Impact { pos: cpos }));
        }
        if let Some(e) = game.world.get_mut(cid) {
            e.pos = pos;
            e.anim.frame = if primed > 0 { 1 } else { 0 };
            if let EntityData::WindCrystal(d) = &mut e.data {
                d.perch = perch;
                d.primed = primed;
                d.throw_id_seen = new_seen;
                d.orbit_angle = orbit;
            }
        }
        let _ = &mut perch;
    }
}

pub(crate) fn maybe_gale(game: &mut Game, crystals: [EntityId; 2], warden: EntityId) {
    let both = crystals.iter().all(|&cid| {
        matches!(
            game.world.get(cid).map(|e| &e.data),
            Some(EntityData::WindCrystal(d)) if d.primed > 0
        )
    });
    if !both {
        // Soft fizzle when one expires alone.
        return;
    }
    // Clear primes, expose core.
    for cid in crystals {
        if let Some(e) = game.world.get_mut(cid) {
            if let EntityData::WindCrystal(d) = &mut e.data {
                d.primed = 0;
            }
        }
    }
    if let Some(e) = game.world.get_mut(warden) {
        if let EntityData::GraniteWarden(d) = &mut e.data {
            d.core_exposed = tuning::WARDEN_CORE_EXPOSE;
            d.fake_armed = true;
            d.attack = WardenAttack::Idle;
            d.timer = 40;
            e.anim.frame = 1; // core glow
        }
    }
    game.world.hitstop = 6;
    game.world.camera.add_shake(2.5, 12);
    game.world
        .push_event(WorldEvent::Sfx(SfxId::GaleStagger));
    game.world.push_event(WorldEvent::FxRequest(FxKind::Toast {
        text: "GALE!",
    }));
    // Wind streaks (particles via impact burst).
    for i in 0..6 {
        let pos = Vec2::new(arena_cx() - 80.0 + i as f32 * 28.0, arena_cy() + (i % 3) as f32 * 10.0);
        game.world
            .push_event(WorldEvent::FxRequest(FxKind::Impact { pos }));
    }

    // P3: swap crystal perches after gale.
    let phase = game
        .world
        .get(warden)
        .and_then(|e| match &e.data {
            EntityData::GraniteWarden(d) => Some(d.phase),
            _ => None,
        })
        .unwrap_or(WardenPhase::One);
    if phase == WardenPhase::Three {
        for cid in crystals {
            if let Some(e) = game.world.get_mut(cid) {
                if let EntityData::WindCrystal(d) = &mut e.data {
                    d.perch = 1 - d.perch;
                }
            }
        }
        game.world
            .push_event(WorldEvent::Sfx(SfxId::CrystalAmber));
    }
}

pub(crate) fn tick_core_timer(game: &mut Game, warden: EntityId) {
    if let Some(e) = game.world.get_mut(warden) {
        if let EntityData::GraniteWarden(d) = &mut e.data {
            if d.core_exposed > 0 {
                d.core_exposed = d.core_exposed.saturating_sub(1);
                e.anim.frame = 1;
                if d.core_exposed == 0 {
                    e.anim.frame = 0;
                }
            }
        }
    }
}

pub(crate) fn tick_fake_flash(game: &mut Game, warden: EntityId) {
    let (phase, exposed, fake_armed, attack, timer) = {
        let Some(e) = game.world.get(warden) else {
            return;
        };
        let EntityData::GraniteWarden(d) = &e.data else {
            return;
        };
        (d.phase, d.core_exposed, d.fake_armed, d.attack, d.timer)
    };
    if phase != WardenPhase::Three || !fake_armed || exposed > 0 {
        return;
    }
    if attack == WardenAttack::Idle && timer == 75 {
        if let Some(e) = game.world.get_mut(warden) {
            if let EntityData::GraniteWarden(d) = &mut e.data {
                d.attack = WardenAttack::FakeFlash;
                d.timer = 0;
                d.fake_armed = false;
                e.anim.frame = 1; // local glint — not gale
            }
        }
        game.world.push_event(WorldEvent::Sfx(SfxId::FakeFlash));
        game.world.push_event(WorldEvent::FxRequest(FxKind::Toast {
            text: "FALSE CORE!",
        }));
        let pos = game
            .world
            .get(warden)
            .map(|e| e.center())
            .unwrap_or(Vec2::ZERO);
        game.world
            .push_event(WorldEvent::FxRequest(FxKind::BlockSpark { pos }));
    }
}

