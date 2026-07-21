//! Breakable barricade tiles.

use content::audio::sfx::SfxId;
use content::maps::TileLayer;
use content::puzzles::OverworldPuzzles;

use crate::fx::FxKind;
use crate::math::Vec2;
use crate::world::WorldEvent;
use crate::Game;

pub fn damage_barricade(
    game: &mut Game,
    def: &OverworldPuzzles,
    tx: u32,
    ty: u32,
    dmg: i32,
) {
    let Some(slot) = game
        .puzzle
        .barricade_hp
        .iter_mut()
        .find(|(t, _)| *t == (tx, ty))
    else {
        return;
    };
    slot.1 -= dmg;
    if slot.1 > 0 {
        return;
    }
    let floor = def
        .barricades
        .iter()
        .find(|b| b.tiles.contains(&(tx, ty)))
        .map(|b| b.floor_tile)
        .unwrap_or(content::maps::catalog::T_DIRT_ASH);
    game.world.set_tile(TileLayer::Ground, tx, ty, floor);
    let pos = Vec2::new(tx as f32 * 16.0 + 8.0, ty as f32 * 16.0 + 8.0);
    game.world
        .push_event(WorldEvent::FxRequest(FxKind::KillPoof { pos }));
    game.world
        .push_event(WorldEvent::Sfx(SfxId::BarricadeBreak));
    super::spawn_rupee_chance(game, pos);
    game.puzzle.barricade_hp.retain(|(t, _)| *t != (tx, ty));
}

