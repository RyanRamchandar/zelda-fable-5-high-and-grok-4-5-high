//! Dungeon crystal / flame / seal puzzle runtime.

use content::audio::sfx::SfxId;
use content::maps::catalog;
use content::maps::dungeon;
use content::maps::{MapId, TileLayer, TILE_PX};
use content::puzzles_dungeon::{self, DungeonPuzzles};

use crate::fx::FxKind;
use crate::math::Vec2;
use crate::rooms;
use crate::save_data::{has_flag, set_flag};
use crate::world::entity::{EntityData, EntityKind};
use crate::world::{AttackKind, WorldEvent};
use crate::Game;

#[derive(Clone, Debug)]
pub struct DungeonPuzzleState {
    pub crystal_amber: Vec<bool>,
    pub flame_lit: [bool; 2],
    pub seal_hits: Option<(u32, Vec<u8>)>,
}

impl DungeonPuzzleState {
    pub fn new() -> Self {
        let def = puzzles_dungeon::def();
        Self {
            crystal_amber: def.crystals.iter().map(|c| c.amber).collect(),
            flame_lit: [false; 2],
            seal_hits: None,
        }
    }
}

impl Default for DungeonPuzzleState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn paint_closed(game: &mut Game) {
    if game.current_map != MapId::Dungeon {
        return;
    }
    let def = puzzles_dungeon::def();
    for g in def.gates {
        if room_solved(game, g.room) {
            continue;
        }
        for &(tx, ty) in g.tiles {
            let tile = if g.amber {
                catalog::D_GATE_AMBER_UP
            } else {
                catalog::D_GATE_BLUE_UP
            };
            game.world.set_tile(TileLayer::Ground, tx, ty, tile);
        }
    }
    if !has_flag(&game.flags, content::flags::DKEY_SMALL_2) {
        for &(tx, ty) in def.flame.gate {
            game.world
                .set_tile(TileLayer::Ground, tx, ty, catalog::D_GATE_BLUE_UP);
        }
        for &(tx, ty) in &def.flame.torches {
            game.world
                .set_tile(TileLayer::Ground, tx, ty, catalog::D_TORCH_UNLIT);
        }
    }
    if !has_flag(&game.flags, content::flags::SEAL_WEST) {
        paint_seal_door(game, dungeon::ROOM_SEAL_W, true);
    }
    if !has_flag(&game.flags, content::flags::SEAL_EAST) {
        paint_seal_door(game, dungeon::ROOM_SEAL_E, true);
    }
}

pub fn restore(game: &mut Game) {
    if game.current_map != MapId::Dungeon {
        return;
    }
    if has_flag(&game.flags, content::flags::SEAL_WEST) {
        paint_seal_door(game, dungeon::ROOM_SEAL_W, false);
    }
    if has_flag(&game.flags, content::flags::SEAL_EAST) {
        paint_seal_door(game, dungeon::ROOM_SEAL_E, false);
    }
    if has_flag(&game.flags, content::flags::SEAL_WEST)
        && has_flag(&game.flags, content::flags::SEAL_EAST)
    {
        open_ante_shutters(game);
    }
    if has_flag(&game.flags, content::flags::DDOOR_WING) {
        open_flagged_doors(game, content::maps::dungeon::DoorKind::SmallKey);
    }
    if has_flag(&game.flags, content::flags::DDOOR_INNER) {
        open_flagged_doors(game, content::maps::dungeon::DoorKind::InnerKey);
    }
    if has_flag(&game.flags, content::flags::DDOOR_BOSS_USED) {
        open_flagged_doors(game, content::maps::dungeon::DoorKind::BossKey);
    }
    if has_flag(&game.flags, content::flags::DKEY_SMALL_2) {
        let def = puzzles_dungeon::def();
        for &(tx, ty) in def.flame.gate {
            game.world
                .set_tile(TileLayer::Ground, tx, ty, catalog::D_GATE_BLUE_DOWN);
        }
        for &(tx, ty) in &def.flame.torches {
            game.world
                .set_tile(TileLayer::Ground, tx, ty, catalog::D_TORCH_LIT);
        }
    }
}

fn open_flagged_doors(game: &mut Game, kind: content::maps::dungeon::DoorKind) {
    for room in dungeon::rooms() {
        for exit in room.exits.iter().flatten() {
            if exit.door == kind {
                game.world.set_tile(
                    TileLayer::Ground,
                    exit.tx,
                    exit.ty,
                    catalog::D_DOOR_OPEN,
                );
            }
        }
    }
}

fn paint_seal_door(game: &mut Game, room: u8, closed: bool) {
    let Some(def) = dungeon::room_by_id(room) else {
        return;
    };
    let tile = if closed {
        catalog::D_SEAL_DOOR
    } else {
        catalog::D_SEAL_BROKEN
    };
    for exit in def.exits.iter().flatten() {
        if matches!(
            exit.door,
            content::maps::dungeon::DoorKind::SealWest | content::maps::dungeon::DoorKind::SealEast
        ) {
            game.world.set_tile(TileLayer::Ground, exit.tx, exit.ty, tile);
        }
    }
    // Also open reciprocal on antechamber.
    if let Some(ante) = dungeon::room_by_id(dungeon::ROOM_ANTECHAMBER) {
        for exit in ante.exits.iter().flatten() {
            if exit.to_room == room {
                game.world.set_tile(TileLayer::Ground, exit.tx, exit.ty, tile);
            }
        }
    }
}

fn open_ante_shutters(game: &mut Game) {
    if let Some(vest) = dungeon::room_by_id(dungeon::ROOM_VESTIBULE) {
        if let Some(ex) = vest.exits[0] {
            game.world
                .set_tile(TileLayer::Ground, ex.tx, ex.ty, catalog::D_DOOR_OPEN);
        }
    }
    if let Some(ante) = dungeon::room_by_id(dungeon::ROOM_ANTECHAMBER) {
        if let Some(ex) = ante.exits[2] {
            game.world
                .set_tile(TileLayer::Ground, ex.tx, ex.ty, catalog::D_DOOR_OPEN);
        }
        if let Some(ex) = ante.exits[0] {
            game.world
                .set_tile(TileLayer::Ground, ex.tx, ex.ty, catalog::D_DOOR_OPEN);
        }
    }
    if let Some(san) = dungeon::room_by_id(dungeon::ROOM_SANCTUM) {
        if let Some(ex) = san.exits[2] {
            game.world
                .set_tile(TileLayer::Ground, ex.tx, ex.ty, catalog::D_DOOR_OPEN);
        }
    }
}

fn room_solved(game: &Game, room: u8) -> bool {
    match room {
        dungeon::ROOM_FLAME => has_flag(&game.flags, content::flags::DKEY_SMALL_2),
        dungeon::ROOM_SEAL_W => has_flag(&game.flags, content::flags::SEAL_WEST),
        dungeon::ROOM_SEAL_E => has_flag(&game.flags, content::flags::SEAL_EAST),
        _ => false,
    }
}

pub fn on_room_enter(game: &mut Game, room: u8) {
    if !puzzles_dungeon::for_room(room) {
        return;
    }
    // Reset unsolved room-local flame state.
    if room == dungeon::ROOM_FLAME && !has_flag(&game.flags, content::flags::DKEY_SMALL_2) {
        if let Some(dp) = game.dungeon_puzzle.as_mut() {
            dp.flame_lit = [false; 2];
        }
        let def = puzzles_dungeon::def();
        for &(tx, ty) in &def.flame.torches {
            game.world
                .set_tile(TileLayer::Ground, tx, ty, catalog::D_TORCH_UNLIT);
        }
        for &(tx, ty) in def.flame.gate {
            game.world
                .set_tile(TileLayer::Ground, tx, ty, catalog::D_GATE_BLUE_UP);
        }
    }
}

pub fn on_group_cleared(game: &mut Game, group: u16) {
    let room = match group {
        content::flags::GRP_DNG_TRIALS_1 => dungeon::ROOM_TRIALS_1,
        content::flags::GRP_DNG_TRIALS_2 => dungeon::ROOM_TRIALS_2,
        content::flags::GRP_DNG_TRIALS_3 => dungeon::ROOM_TRIALS_3,
        _ => return,
    };
    rooms::open_room_shutters(game, room);
}

pub fn update(game: &mut Game) {
    if game.current_map != MapId::Dungeon {
        return;
    }
    if game.dungeon_puzzle.is_none() {
        game.dungeon_puzzle = Some(DungeonPuzzleState::new());
    }
    process_hits(game);
    update_flame(game);
    update_multi(game);
}

fn process_hits(game: &mut Game) {
    let def = puzzles_dungeon::def();
    let attacks = game.world.active_attacks.clone();
    for atk in &attacks {
        try_crystal_hit(game, &def, atk.center, atk.swing_id, atk.kind);
        try_seal_hit(game, &def, atk.center, atk.swing_id);
    }
    let ids = game.world.alive_ids();
    for id in ids {
        let (center, swing_id, kind, mark_hit) = {
            let Some(e) = game.world.get(id) else {
                continue;
            };
            match (&e.kind, &e.data) {
                (EntityKind::SwordBeam, EntityData::Beam(b)) if b.from_player && !b.hit => {
                    (e.center(), b.swing_id, AttackKind::Beam, true)
                }
                (EntityKind::OctorokRock, EntityData::Rock(r)) if r.from_player && !r.hit => {
                    (e.center(), r.swing_id, AttackKind::Beam, true)
                }
                (EntityKind::Boomerang, EntityData::Boomerang(b)) => {
                    (e.center(), b.throw_id, AttackKind::Boomerang, false)
                }
                _ => continue,
            }
        };
        let hit = try_crystal_hit(game, &def, center, swing_id, kind);
        if hit && mark_hit {
            if let Some(e) = game.world.get_mut(id) {
                match &mut e.data {
                    EntityData::Beam(b) => {
                        b.hit = true;
                        e.alive = false;
                    }
                    EntityData::Rock(r) => r.hit = true,
                    _ => {}
                }
            }
        }
        try_seal_hit(game, &def, center, swing_id);
        try_flame_boomerang(game, &def, id, center);
    }
}

fn try_crystal_hit(
    game: &mut Game,
    def: &DungeonPuzzles,
    center: Vec2,
    swing_id: u32,
    _kind: AttackKind,
) -> bool {
    let mut any = false;
    for (i, c) in def.crystals.iter().enumerate() {
        if c.room == dungeon::ROOM_SEAL_W || c.room == dungeon::ROOM_SEAL_E {
            continue; // seals use ordered path
        }
        if !tile_hit(center, c.tx, c.ty) {
            continue;
        }
        if !crate::puzzle::mark_tile_hit_pub(&mut game.puzzle, swing_id, c.tx, c.ty) {
            continue;
        }
        toggle_crystal(game, def, i);
        any = true;
    }
    any
}

fn toggle_crystal(game: &mut Game, def: &DungeonPuzzles, idx: usize) {
    let Some(dp) = game.dungeon_puzzle.as_mut() else {
        return;
    };
    if idx >= dp.crystal_amber.len() {
        return;
    }
    dp.crystal_amber[idx] = !dp.crystal_amber[idx];
    let amber = dp.crystal_amber[idx];
    let c = def.crystals[idx];
    let sprite = if amber {
        catalog::D_CRYSTAL_AMBER
    } else {
        catalog::D_CRYSTAL_BLUE
    };
    game.world
        .set_tile(TileLayer::Ground, c.tx, c.ty, sprite);
    game.world.push_event(WorldEvent::Sfx(if amber {
        SfxId::CrystalAmber
    } else {
        SfxId::CrystalBlue
    }));
    // Toggle same-family gates in this room.
    for g in def.gates {
        if g.room != c.room || g.amber != c.amber {
            continue;
        }
        for &(tx, ty) in g.tiles {
            let cur = game.world.map.get(tx, ty, TileLayer::Ground);
            let next = match cur {
                catalog::D_GATE_BLUE_UP => catalog::D_GATE_BLUE_DOWN,
                catalog::D_GATE_BLUE_DOWN => catalog::D_GATE_BLUE_UP,
                catalog::D_GATE_AMBER_UP => catalog::D_GATE_AMBER_DOWN,
                catalog::D_GATE_AMBER_DOWN => catalog::D_GATE_AMBER_UP,
                _ => continue,
            };
            game.world.set_tile(TileLayer::Ground, tx, ty, next);
        }
    }
}

fn try_seal_hit(game: &mut Game, def: &DungeonPuzzles, center: Vec2, throw_id: u32) {
    for seal in [&def.seal_w, &def.seal_e] {
        if has_flag(&game.flags, seal.flag) {
            continue;
        }
        for (i, &(tx, ty)) in seal.crystals.iter().enumerate() {
            if !tile_hit(center, tx, ty) {
                continue;
            }
            if !crate::puzzle::mark_tile_hit_pub(&mut game.puzzle, throw_id, tx, ty) {
                continue;
            }
            let Some(dp) = game.dungeon_puzzle.as_mut() else {
                return;
            };
            let entry = dp.seal_hits.get_or_insert_with(|| (throw_id, Vec::new()));
            if entry.0 != throw_id {
                *entry = (throw_id, Vec::new());
            }
            if entry.1.contains(&(i as u8)) {
                continue;
            }
            let expected = entry.1.len() as u8;
            if i as u8 != expected {
                game.world.push_event(WorldEvent::Sfx(SfxId::RuneBad));
                dp.seal_hits = None;
                continue;
            }
            entry.1.push(i as u8);
            game.world.push_event(WorldEvent::Sfx(SfxId::RuneGood));
            if entry.1.len() == 3 {
                set_flag(&mut game.flags, seal.flag);
                paint_seal_door(game, seal.room, false);
                game.world
                    .push_event(WorldEvent::Sfx(SfxId::SealBreak));
                game.world.camera.add_shake(2.5, 10);
                game.world.push_event(WorldEvent::FxRequest(FxKind::Toast {
                    text: "SEAL BREAKS",
                }));
                if has_flag(&game.flags, content::flags::SEAL_WEST)
                    && has_flag(&game.flags, content::flags::SEAL_EAST)
                {
                    open_ante_shutters(game);
                    game.world.push_event(WorldEvent::FxRequest(FxKind::Toast {
                        text: "THE WAY OPENS",
                    }));
                }
            }
        }
    }
}

fn try_flame_boomerang(
    game: &mut Game,
    def: &DungeonPuzzles,
    id: crate::world::EntityId,
    center: Vec2,
) {
    let EntityKind::Boomerang = game.world.get(id).map(|e| e.kind).unwrap_or(EntityKind::Dummy)
    else {
        return;
    };
    let (bx, by) = def.flame.brazier;
    let near_brazier = tile_near(center, bx, by);
    if near_brazier {
        if let Some(e) = game.world.get_mut(id) {
            if let EntityData::Boomerang(b) = &mut e.data {
                if !b.flame {
                    b.flame = true;
                    game.world
                        .push_event(WorldEvent::Sfx(SfxId::TorchLight));
                }
            }
        }
    }
    let flame = game
        .world
        .get(id)
        .and_then(|e| match &e.data {
            EntityData::Boomerang(b) => Some(b.flame),
            _ => None,
        })
        .unwrap_or(false);
    for (i, &(tx, ty)) in def.flame.torches.iter().enumerate() {
        if !tile_near(center, tx, ty) {
            continue;
        }
        let Some(dp) = game.dungeon_puzzle.as_mut() else {
            return;
        };
        if flame {
            if !dp.flame_lit[i] {
                dp.flame_lit[i] = true;
                game.world
                    .set_tile(TileLayer::Ground, tx, ty, catalog::D_TORCH_LIT);
                game.world
                    .push_event(WorldEvent::Sfx(SfxId::TorchLight));
            }
        } else if dp.flame_lit[i] {
            dp.flame_lit[i] = false;
            game.world
                .set_tile(TileLayer::Ground, tx, ty, catalog::D_TORCH_UNLIT);
            game.world
                .push_event(WorldEvent::Sfx(SfxId::TorchSnuff));
        }
    }
}

fn update_flame(game: &mut Game) {
    if has_flag(&game.flags, content::flags::DKEY_SMALL_2) {
        return;
    }
    let def = puzzles_dungeon::def();
    let Some(dp) = game.dungeon_puzzle.as_ref() else {
        return;
    };
    if dp.flame_lit[0] && dp.flame_lit[1] {
        for &(tx, ty) in def.flame.gate {
            game.world
                .set_tile(TileLayer::Ground, tx, ty, catalog::D_GATE_BLUE_DOWN);
        }
    }
}

fn update_multi(game: &mut Game) {
    let def = puzzles_dungeon::def();
    let Some(dp) = game.dungeon_puzzle.as_ref() else {
        return;
    };
    // Crystals at indices 4 and 5 are multi room.
    if dp.crystal_amber.len() < 6 {
        return;
    }
    let same = dp.crystal_amber[4] == dp.crystal_amber[5];
    for &(tx, ty) in def.multi_gate {
        let tile = if same {
            catalog::D_GATE_BLUE_DOWN
        } else {
            catalog::D_GATE_BLUE_UP
        };
        game.world.set_tile(TileLayer::Ground, tx, ty, tile);
    }
}

fn tile_hit(center: Vec2, tx: u32, ty: u32) -> bool {
    let tc = Vec2::new(tx as f32 * TILE_PX + 8.0, ty as f32 * TILE_PX + 8.0);
    center.sub(tc).len() < 14.0
}

fn tile_near(center: Vec2, tx: u32, ty: u32) -> bool {
    let tc = Vec2::new(tx as f32 * TILE_PX + 8.0, ty as f32 * TILE_PX + 8.0);
    center.sub(tc).len() < 20.0
}
