//! Ruins pressure-plate court.

use content::audio::sfx::SfxId;
use content::maps::catalog;
use content::maps::TileLayer;
use content::puzzles::OverworldPuzzles;

use crate::fx::FxKind;
use crate::save_data::{has_flag, set_flag};
use crate::world::WorldEvent;
use crate::Game;

use super::open_gate_tiles;

pub fn update_plates(game: &mut Game, def: &OverworldPuzzles) {
    if has_flag(&game.flags, def.plate_court.flag) {
        // Keep plates visually down once solved.
        for &(tx, ty) in &def.plate_court.plates {
            if game.world.map.get(tx, ty, TileLayer::Ground) != catalog::T_PLATE_DOWN {
                game.world
                    .set_tile(TileLayer::Ground, tx, ty, catalog::T_PLATE_DOWN);
            }
        }
        return;
    }

    let feet = {
        let Some(p) = game.world.get(game.world.player_id) else {
            return;
        };
        let c = p.center();
        (
            (c.x / 16.0).floor() as u32,
            ((c.y + 6.0) / 16.0).floor() as u32,
        )
    };

    let mut pressed = [false; 2];
    for (i, &(px, py)) in def.plate_court.plates.iter().enumerate() {
        let block_on = game.world.map.get(px, py, TileLayer::Ground) == catalog::T_BLOCK;
        let player_on = feet.0 == px && feet.1 == py;
        // If a block occupies the plate tile, the ground id is T_BLOCK — treat as pressed.
        // Otherwise check T_PLATE_* under player / leave plate tiles as plate when empty.
        pressed[i] = block_on || player_on;
        // Visual: when block is on plate, block sprite wins (ground is block).
        // When only player: show plate down underfoot; when empty: plate up.
        if !block_on {
            let want = if player_on {
                catalog::T_PLATE_DOWN
            } else {
                catalog::T_PLATE_UP
            };
            let cur = game.world.map.get(px, py, TileLayer::Ground);
            if cur != want && (cur == catalog::T_PLATE_UP || cur == catalog::T_PLATE_DOWN) {
                game.world.set_tile(TileLayer::Ground, px, py, want);
                if want != (if game.puzzle.plate_down[i] {
                    catalog::T_PLATE_DOWN
                } else {
                    catalog::T_PLATE_UP
                }) {
                    game.world
                        .push_event(WorldEvent::Sfx(SfxId::PlateClick));
                }
            }
        }
        game.puzzle.plate_down[i] = pressed[i];
    }

    if pressed[0] && pressed[1] {
        set_flag(&mut game.flags, def.plate_court.flag);
        open_gate_tiles(
            &mut game.world,
            def.plate_court.gate,
            def.plate_court.open_tile,
        );
        game.world
            .push_event(WorldEvent::Sfx(SfxId::GateOpen));
        game.world.camera.add_shake(2.0, 10);
        game.world.push_event(WorldEvent::FxRequest(FxKind::Toast {
            text: "GATE OPENS",
        }));
        if let Some(json) = crate::state::save_from_game(game).to_json() {
            game.pending_save = Some(json);
        }
    } else {
        // Re-close gates while unsolved if any plate released.
        for &(tx, ty) in def.plate_court.gate {
            let cur = game.world.map.get(tx, ty, TileLayer::Ground);
            if cur != catalog::T_GATE {
                game.world
                    .set_tile(TileLayer::Ground, tx, ty, catalog::T_GATE);
            }
        }
    }
}
