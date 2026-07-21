//! Tile-based puzzle layer (chimes, plates, blocks, barricades, cranks, bomb walls).

mod barricades;
pub(crate) mod chimes;
pub(crate) mod dungeon;
mod plates;

use content::audio::sfx::SfxId;
use content::maps::catalog;
use content::maps::{flags as tile_flags, MapId, TileLayer, TriggerDef, TriggerKind, TILE_PX};
use content::puzzles::{self, OverworldPuzzles};

use crate::fx::FxKind;
use crate::items::pickups;
use crate::math::Vec2;
use crate::save_data::{has_flag, set_flag};
use crate::world::entity::{EntityData, EntityKind, PickupKind};
use crate::world::{ActiveAttack, AttackKind, World, WorldEvent};
use crate::Game;

const BLOCK_PUSH_TICKS: u16 = 6; // Phase 5: snappier push (was 8)
const TILE: f32 = TILE_PX;

#[derive(Clone, Debug)]
pub struct PuzzleState {
    #[allow(dead_code)]
    pub map: MapId,
    /// Finale chime ring times (tick) per index; 0 = not recently rung.
    pub chime_rings: [u64; 3],
    pub push_ticks: u16,
    pub push_dir: (i32, i32),
    pub push_block: Option<(u32, u32)>,
    /// `(swing_id, tx, ty)` sword-hit dedupe.
    pub hit_tiles: Vec<(u32, u32, u32)>,
    pub barricade_hp: Vec<((u32, u32), i32)>,
    pub plate_down: [bool; 2],
}

impl Default for PuzzleState {
    fn default() -> Self {
        Self::for_map(MapId::Overworld)
    }
}

impl PuzzleState {
    pub fn for_map(map: MapId) -> Self {
        let mut s = Self {
            map,
            chime_rings: [0; 3],
            push_ticks: 0,
            push_dir: (0, 0),
            push_block: None,
            hit_tiles: Vec::new(),
            barricade_hp: Vec::new(),
            plate_down: [false; 2],
        };
        if let Some(def) = puzzles::for_map(map) {
            for b in def.barricades {
                for &(tx, ty) in b.tiles {
                    s.barricade_hp
                        .push(((tx, ty), crate::combat::tuning::BARRICADE_HP));
                }
            }
        }
        s
    }
}

pub fn paint_closed(world: &mut World, map: MapId, flags: &[u16]) {
    let Some(def) = puzzles::for_map(map) else {
        return;
    };
    for g in def.chime_gates {
        if !has_flag(flags, g.flag) {
            for &(tx, ty) in g.gate {
                world.set_tile(TileLayer::Ground, tx, ty, catalog::T_GATE);
            }
        }
    }
    if !has_flag(flags, def.plate_court.flag) {
        for &(tx, ty) in def.plate_court.gate {
            world.set_tile(TileLayer::Ground, tx, ty, catalog::T_GATE);
        }
        for &(tx, ty) in &def.plate_court.plates {
            world.set_tile(TileLayer::Ground, tx, ty, catalog::T_PLATE_UP);
        }
        for &(tx, ty) in &def.plate_court.blocks {
            world.set_tile(TileLayer::Ground, tx, ty, catalog::T_BLOCK);
        }
    }
    if !has_flag(flags, def.ruins_far_switch.flag) {
        for &(tx, ty) in def.ruins_far_switch.gate {
            world.set_tile(TileLayer::Ground, tx, ty, catalog::T_GATE);
        }
        let (cx, cy) = def.ruins_far_switch.crank;
        world.set_tile(TileLayer::Ground, cx, cy, catalog::T_CRANK);
    }
    if !has_flag(flags, def.bridge_crank.flag) {
        let (cx, cy) = def.bridge_crank.crank;
        world.set_tile(TileLayer::Ground, cx, cy, catalog::T_CRANK);
    }
    for b in def.barricades {
        for &(tx, ty) in b.tiles {
            world.set_tile(TileLayer::Ground, tx, ty, catalog::T_BARRICADE);
        }
    }
}

pub fn restore(world: &mut World, flags: &[u16]) {
    let Some(def) = puzzles::for_map(world.map_id) else {
        return;
    };
    for g in def.chime_gates {
        if has_flag(flags, g.flag) {
            open_gate_tiles(world, g.gate, g.open_tile);
        }
    }
    if has_flag(flags, def.plate_court.flag) {
        open_gate_tiles(world, def.plate_court.gate, def.plate_court.open_tile);
        for &(tx, ty) in &def.plate_court.plates {
            world.set_tile(TileLayer::Ground, tx, ty, catalog::T_PLATE_DOWN);
        }
    }
    if has_flag(flags, def.bridge_crank.flag) {
        apply_crank_swaps(world, def.bridge_crank.swaps);
    }
    if has_flag(flags, def.bomb_wall.flag) {
        open_bomb_wall(world, &def.bomb_wall);
    }
    if has_flag(flags, def.ruins_far_switch.flag) {
        open_gate_tiles(
            world,
            def.ruins_far_switch.gate,
            def.ruins_far_switch.open_tile,
        );
    }
}

pub fn update(game: &mut Game) {
    prune_hit_tiles(&mut game.puzzle);
    if game.current_map == MapId::Dungeon {
        dungeon::update(game);
        return;
    }
    let Some(def) = puzzles::for_map(game.current_map) else {
        return;
    };
    process_hits(game, def);
    plates::update_plates(game, def);
    update_block_push(game, def);
    chimes::tick_finale_expiry(game, def);
}

pub(crate) fn mark_tile_hit_pub(puzzle: &mut PuzzleState, swing_id: u32, tx: u32, ty: u32) -> bool {
    mark_tile_hit(puzzle, swing_id, tx, ty)
}

fn prune_hit_tiles(puzzle: &mut PuzzleState) {
    let live: Vec<u32> = puzzle
        .hit_tiles
        .iter()
        .map(|h| h.0)
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();
    // Keep only recent swing ids still referenced; cap growth.
    if puzzle.hit_tiles.len() > 64 {
        puzzle.hit_tiles.retain(|h| live.contains(&h.0));
        if puzzle.hit_tiles.len() > 64 {
            puzzle.hit_tiles.drain(0..puzzle.hit_tiles.len() - 32);
        }
    }
}

fn process_hits(game: &mut Game, def: &OverworldPuzzles) {
    let attacks: Vec<ActiveAttack> = game.world.active_attacks.clone();
    for atk in &attacks {
        try_hit_tiles(game, def, atk.center, atk.half, atk.use_radius, atk.radius, atk.swing_id, atk.kind);
    }

    let ids = game.world.alive_ids();
    for id in ids {
        let (kind, center, from_player, hit, swing_id) = {
            let Some(e) = game.world.get(id) else {
                continue;
            };
            match (&e.kind, &e.data) {
                (EntityKind::SwordBeam, EntityData::Beam(b)) => {
                    (EntityKind::SwordBeam, e.center(), b.from_player, b.hit, b.swing_id)
                }
                (EntityKind::OctorokRock, EntityData::Rock(r)) => {
                    (EntityKind::OctorokRock, e.center(), r.from_player, r.hit, 0)
                }
                (EntityKind::Boomerang, EntityData::Boomerang(b)) => {
                    (EntityKind::Boomerang, e.center(), true, false, b.throw_id)
                }
                _ => continue,
            }
        };
        if !from_player || hit {
            continue;
        }
        let half = Vec2::new(2.0, 2.0);
        let ak = if kind == EntityKind::Boomerang {
            AttackKind::Boomerang
        } else {
            AttackKind::Beam
        };
        let sid = if kind == EntityKind::Boomerang {
            swing_id
        } else {
            swing_id.wrapping_add(0xB0B0)
        };
        let marked = try_hit_tiles(game, def, center, half, false, 0.0, sid, ak);
        if marked && kind != EntityKind::Boomerang {
            if let Some(e) = game.world.get_mut(id) {
                match &mut e.data {
                    EntityData::Beam(b) => b.hit = true,
                    EntityData::Rock(r) => r.hit = true,
                    _ => {}
                }
                if kind == EntityKind::SwordBeam {
                    e.alive = false;
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn try_hit_tiles(
    game: &mut Game,
    def: &OverworldPuzzles,
    center: Vec2,
    half: Vec2,
    use_radius: bool,
    radius: f32,
    swing_id: u32,
    kind: AttackKind,
) -> bool {
    let mut any = false;
    // Chime gates
    for g in def.chime_gates {
        let (tx, ty) = g.chime;
        if tile_hit(center, half, use_radius, radius, tx, ty)
            && mark_tile_hit(&mut game.puzzle, swing_id, tx, ty)
        {
            chimes::ring_gate_chime(game, g);
            any = true;
        }
    }
    // Finale chimes
    for (i, &(tx, ty)) in def.chime_finale.chimes.iter().enumerate() {
        if tile_hit(center, half, use_radius, radius, tx, ty)
            && mark_tile_hit(&mut game.puzzle, swing_id, tx, ty)
        {
            chimes::ring_finale_chime(game, def, i);
            any = true;
        }
    }
    // Barricades
    let dmg = barricade_damage(kind);
    if dmg > 0 {
        let tiles: Vec<(u32, u32)> = game
            .puzzle
            .barricade_hp
            .iter()
            .map(|(t, _)| *t)
            .collect();
        for (tx, ty) in tiles {
            if tile_hit(center, half, use_radius, radius, tx, ty)
                && mark_tile_hit(&mut game.puzzle, swing_id, tx, ty)
            {
                barricades::damage_barricade(game, def, tx, ty, dmg);
                any = true;
            }
        }
    }
    // Bridge crank
    let (cx, cy) = def.bridge_crank.crank;
    if !has_flag(&game.flags, def.bridge_crank.flag)
        && tile_hit(center, half, use_radius, radius, cx, cy)
        && mark_tile_hit(&mut game.puzzle, swing_id, cx, cy)
    {
        turn_bridge_crank(game, def);
        any = true;
    }
    // Far switch crank
    let (fx, fy) = def.ruins_far_switch.crank;
    if !has_flag(&game.flags, def.ruins_far_switch.flag)
        && tile_hit(center, half, use_radius, radius, fx, fy)
        && mark_tile_hit(&mut game.puzzle, swing_id, fx, fy)
    {
        turn_far_switch(game, def);
        any = true;
    }
    any
}

fn barricade_damage(kind: AttackKind) -> i32 {
    match kind {
        AttackKind::Finisher | AttackKind::Spin => 2,
        AttackKind::Bomb => 99,
        AttackKind::Slash
        | AttackKind::Backslash
        | AttackKind::Beam
        | AttackKind::DebugShot
        | AttackKind::Boomerang => 1,
    }
}

fn tile_hit(
    center: Vec2,
    half: Vec2,
    use_radius: bool,
    radius: f32,
    tx: u32,
    ty: u32,
) -> bool {
    let tl = Vec2::new(tx as f32 * TILE, ty as f32 * TILE);
    let tr = Vec2::new(TILE * 0.5, TILE * 0.5);
    let tc = tl.add(tr);
    if use_radius {
        let dx = center.x - tc.x;
        let dy = center.y - tc.y;
        dx * dx + dy * dy <= radius * radius
    } else {
        aabb_overlap(center, half, tc, tr)
    }
}

fn aabb_overlap(c0: Vec2, h0: Vec2, c1: Vec2, h1: Vec2) -> bool {
    (c0.x - h0.x) < (c1.x + h1.x)
        && (c0.x + h0.x) > (c1.x - h1.x)
        && (c0.y - h0.y) < (c1.y + h1.y)
        && (c0.y + h0.y) > (c1.y - h1.y)
}

fn mark_tile_hit(puzzle: &mut PuzzleState, swing_id: u32, tx: u32, ty: u32) -> bool {
    let key = (swing_id, tx, ty);
    if puzzle.hit_tiles.contains(&key) {
        return false;
    }
    puzzle.hit_tiles.push(key);
    true
}

fn open_gate_tiles(world: &mut World, tiles: &[(u32, u32)], open: u16) {
    for &(tx, ty) in tiles {
        world.set_tile(TileLayer::Ground, tx, ty, open);
    }
}

fn apply_crank_swaps(world: &mut World, swaps: &[(u32, u32, u16)]) {
    for &(tx, ty, id) in swaps {
        world.set_tile(TileLayer::Ground, tx, ty, id);
    }
}

pub(crate) fn open_bomb_wall(world: &mut World, wall: &content::puzzles::BombWallDef) {
    let (tx, ty) = wall.wall;
    world.set_tile(TileLayer::Ground, tx, ty, wall.open_tile);
    let (target, entry) = wall.door;
    if !world.map.triggers.iter().any(|t| {
        matches!(
            &t.kind,
            TriggerKind::Door {
                target: tmap,
                entry: e
            } if *tmap == target && *e == entry
        ) && t.tx == tx
            && t.ty == ty
    }) {
        world.map.triggers.push(TriggerDef {
            tx,
            ty,
            w: 1,
            h: 1,
            kind: TriggerKind::Door { target, entry },
        });
    }
}

fn turn_bridge_crank(game: &mut Game, def: &OverworldPuzzles) {
    set_flag(&mut game.flags, def.bridge_crank.flag);
    apply_crank_swaps(&mut game.world, def.bridge_crank.swaps);
    game.world.push_event(WorldEvent::Sfx(SfxId::CrankTurn));
    game.world.push_event(WorldEvent::Sfx(SfxId::SealOpen));
    game.world.camera.add_shake(2.5, 12);
    game.world.push_event(WorldEvent::FxRequest(FxKind::Toast {
        text: "SHORTCUT OPEN!",
    }));
    if let Some(json) = crate::state::save_from_game(game).to_json() {
        game.pending_save = Some(json);
    }
}

fn turn_far_switch(game: &mut Game, def: &OverworldPuzzles) {
    set_flag(&mut game.flags, def.ruins_far_switch.flag);
    open_gate_tiles(
        &mut game.world,
        def.ruins_far_switch.gate,
        def.ruins_far_switch.open_tile,
    );
    game.world.push_event(WorldEvent::Sfx(SfxId::CrankTurn));
    game.world.push_event(WorldEvent::FxRequest(FxKind::Toast {
        text: "GATE OPENS",
    }));
    if let Some(json) = crate::state::save_from_game(game).to_json() {
        game.pending_save = Some(json);
    }
}

fn update_block_push(game: &mut Game, def: &OverworldPuzzles) {
    if has_flag(&game.flags, def.plate_court.flag) {
        return;
    }
    let floor = def.plate_court.floor_tile;
    let (ppos, pvel, feet) = {
        let Some(p) = game.world.get(game.world.player_id) else {
            return;
        };
        let c = p.center();
        let feet = (
            (c.x / TILE).floor() as i32,
            ((c.y + 6.0) / TILE).floor() as i32,
        );
        (c, p.vel, feet)
    };
    if pvel.len_sq() < 0.2 {
        game.puzzle.push_ticks = 0;
        game.puzzle.push_block = None;
        return;
    }
    let (dx, dy) = if pvel.x.abs() >= pvel.y.abs() {
        (pvel.x.signum() as i32, 0)
    } else {
        (0, pvel.y.signum() as i32)
    };
    if dx == 0 && dy == 0 {
        game.puzzle.push_ticks = 0;
        return;
    }
    let bx = feet.0 + dx;
    let by = feet.1 + dy;
    if bx < 0 || by < 0 {
        game.puzzle.push_ticks = 0;
        return;
    }
    let (bx, by) = (bx as u32, by as u32);
    let tile = game.world.map.get(bx, by, TileLayer::Ground);
    if tile != catalog::T_BLOCK {
        game.puzzle.push_ticks = 0;
        game.puzzle.push_block = None;
        return;
    }
    // Contact sustained.
    let same = game.puzzle.push_block == Some((bx, by))
        && game.puzzle.push_dir == (dx, dy);
    if same {
        game.puzzle.push_ticks = game.puzzle.push_ticks.saturating_add(1);
    } else {
        game.puzzle.push_block = Some((bx, by));
        game.puzzle.push_dir = (dx, dy);
        game.puzzle.push_ticks = 1;
    }
    if game.puzzle.push_ticks < BLOCK_PUSH_TICKS {
        return;
    }
    let nx = bx as i32 + dx;
    let ny = by as i32 + dy;
    if nx < 0 || ny < 0 {
        return;
    }
    let (nx, ny) = (nx as u32, ny as u32);
    if !can_block_land(&game.world, nx, ny) {
        return;
    }
    game.world
        .set_tile(TileLayer::Ground, bx, by, floor);
    game.world
        .set_tile(TileLayer::Ground, nx, ny, catalog::T_BLOCK);
    game.puzzle.push_ticks = 0;
    game.puzzle.push_block = Some((nx, ny));
    let pos = Vec2::new(nx as f32 * TILE + 8.0, ny as f32 * TILE + 8.0);
    game.world
        .push_event(WorldEvent::Sfx(SfxId::BlockSlide));
    game.world
        .push_event(WorldEvent::FxRequest(FxKind::Dust { pos }));
    let _ = ppos;
}

fn can_block_land(world: &World, tx: u32, ty: u32) -> bool {
    if tx >= world.map.width || ty >= world.map.height {
        return false;
    }
    let id = world.map.get(tx, ty, TileLayer::Ground);
    if id == catalog::T_BLOCK {
        return false;
    }
    let info = catalog::tile_info(id);
    let f = info.flags;
    if f & tile_flags::SOLID != 0 || f & tile_flags::WATER != 0 {
        return false;
    }
    true
}

/// Called by bombs when a cracked wall is in blast radius.
pub fn try_open_bomb_wall(game: &mut Game, blast_center: Vec2, radius: f32) -> bool {
    let Some(def) = puzzles::for_map(game.current_map) else {
        return false;
    };
    if has_flag(&game.flags, def.bomb_wall.flag) {
        return false;
    }
    let (tx, ty) = def.bomb_wall.wall;
    let tc = Vec2::new(tx as f32 * TILE + 8.0, ty as f32 * TILE + 8.0);
    let dx = blast_center.x - tc.x;
    let dy = blast_center.y - tc.y;
    if dx * dx + dy * dy > radius * radius {
        return false;
    }
    let ground = game.world.map.get(tx, ty, TileLayer::Ground);
    if ground != catalog::T_CRACKED_WALL {
        return false;
    }
    set_flag(&mut game.flags, def.bomb_wall.flag);
    set_flag(&mut game.flags, content::flags::SECRET_GROVE_BOMB);
    open_bomb_wall(&mut game.world, &def.bomb_wall);
    game.world
        .push_event(WorldEvent::Sfx(SfxId::SecretChime));
    game.world.push_event(WorldEvent::FxRequest(FxKind::Toast {
        text: "SECRET!",
    }));
    game.ui.minimap.mark_discovered_secret();
    if let Some(json) = crate::state::save_from_game(game).to_json() {
        game.pending_save = Some(json);
    }
    true
}

/// Bomb blast vs barricades in radius.
pub fn bomb_break_barricades(game: &mut Game, center: Vec2, radius: f32) {
    let Some(def) = puzzles::for_map(game.current_map) else {
        return;
    };
    let tiles: Vec<(u32, u32)> = game
        .puzzle
        .barricade_hp
        .iter()
        .map(|(t, _)| *t)
        .collect();
    for (tx, ty) in tiles {
        let tc = Vec2::new(tx as f32 * TILE + 8.0, ty as f32 * TILE + 8.0);
        let dx = center.x - tc.x;
        let dy = center.y - tc.y;
        if dx * dx + dy * dy <= radius * radius {
            barricades::damage_barricade(game, def, tx, ty, 99);
        }
    }
}

pub fn spawn_rupee_chance(game: &mut Game, pos: Vec2) {
    if game.world.rng.f32() < 0.30 {
        pickups::spawn_one(&mut game.world, pos, PickupKind::Rupee);
    }
}
