//! Grove chime gates + three-chime finale window.

use content::audio::sfx::SfxId;
use content::puzzles::{ChimeGateDef, OverworldPuzzles};

use crate::fx::FxKind;
use crate::math::Vec2;
use crate::save_data::{has_flag, set_flag};
use crate::world::entity::{EntityData, EntityKind};
use crate::world::WorldEvent;
use crate::Game;

use super::open_gate_tiles;

pub fn ring_gate_chime(game: &mut Game, gate: &ChimeGateDef) {
    let (tx, ty) = gate.chime;
    let pos = Vec2::new(tx as f32 * 16.0 + 8.0, ty as f32 * 16.0 + 8.0);
    game.world
        .push_event(WorldEvent::Sfx(SfxId::ChimeRing));
    game.world
        .push_event(WorldEvent::FxRequest(FxKind::FountainSparkle { pos }));
    if has_flag(&game.flags, gate.flag) {
        return;
    }
    set_flag(&mut game.flags, gate.flag);
    open_gate_tiles(&mut game.world, gate.gate, gate.open_tile);
    game.world
        .push_event(WorldEvent::Sfx(SfxId::GateOpen));
    game.world.camera.add_shake(1.5, 8);
    if let Some(json) = crate::state::save_from_game(game).to_json() {
        game.pending_save = Some(json);
    }
}

pub fn ring_finale_chime(game: &mut Game, def: &OverworldPuzzles, idx: usize) {
    if has_flag(&game.flags, def.chime_finale.flag) {
        return;
    }
    let (tx, ty) = def.chime_finale.chimes[idx];
    let pos = Vec2::new(tx as f32 * 16.0 + 8.0, ty as f32 * 16.0 + 8.0);
    game.world
        .push_event(WorldEvent::Sfx(SfxId::ChimeRing));
    game.world
        .push_event(WorldEvent::FxRequest(FxKind::FountainSparkle { pos }));
    let now = game.world.tick;
    game.puzzle.chime_rings[idx] = now;

    let window = def.chime_finale.window_ticks as u64;
    let all = game.puzzle.chime_rings.iter().all(|&t| t > 0 && now.saturating_sub(t) <= window);
    if !all {
        return;
    }
    // Ensure the oldest is still inside the window from the newest.
    let min_t = game.puzzle.chime_rings.iter().copied().min().unwrap_or(0);
    let max_t = game.puzzle.chime_rings.iter().copied().max().unwrap_or(0);
    if max_t.saturating_sub(min_t) > window {
        return;
    }
    set_flag(&mut game.flags, def.chime_finale.flag);
    clear_courage_seal(game);
    game.world
        .push_event(WorldEvent::Sfx(SfxId::SealOpen));
    game.world.push_event(WorldEvent::FxRequest(FxKind::Toast {
        text: "THE SEAL FADES",
    }));
    game.world.camera.add_shake(2.0, 12);
    if let Some(json) = crate::state::save_from_game(game).to_json() {
        game.pending_save = Some(json);
    }
}

pub fn tick_finale_expiry(game: &mut Game, def: &OverworldPuzzles) {
    if has_flag(&game.flags, def.chime_finale.flag) {
        return;
    }
    let now = game.world.tick;
    let window = def.chime_finale.window_ticks as u64;
    for i in 0..3 {
        let t = game.puzzle.chime_rings[i];
        if t > 0 && now.saturating_sub(t) > window {
            game.puzzle.chime_rings[i] = 0;
            game.world
                .push_event(WorldEvent::Sfx(SfxId::ChimeMiss));
        }
    }
}

pub fn clear_courage_seal(game: &mut Game) {
    let ids = game.world.alive_ids();
    for id in ids {
        if let Some(e) = game.world.get_mut(id) {
            if e.kind == EntityKind::Gem {
                if let EntityData::Gem(g) = &mut e.data {
                    if g.id == 0 {
                        g.sealed = false;
                    }
                }
            }
        }
    }
}

pub fn apply_courage_seal_from_flags(world: &mut crate::world::World, flags: &[u16]) {
    let sealed = !has_flag(flags, content::flags::PUZZLE_CHIMES_DONE);
    let ids = world.alive_ids();
    for id in ids {
        if let Some(e) = world.get_mut(id) {
            if e.kind == EntityKind::Gem {
                if let EntityData::Gem(g) = &mut e.data {
                    if g.id == 0 {
                        g.sealed = sealed;
                    }
                }
            }
        }
    }
}
