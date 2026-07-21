//! Dungeon room camera lock + slide transitions.

use content::audio::sfx::SfxId;
use content::maps::dungeon::{self, DoorKind, ExitDef, RoomDef};
use content::maps::{catalog, MapId, TileLayer, TILE_PX};
use content::text::TextId;

use crate::fx::FxKind;
use crate::math::Vec2;
use crate::save_data::{has_flag, set_flag};
use crate::world::WorldEvent;
use crate::Game;

pub const SLIDE_TICKS: u8 = 24;

#[derive(Clone, Debug)]
pub struct Slide {
    pub from_rect: (f32, f32, f32, f32),
    pub to_rect: (f32, f32, f32, f32),
    pub t: u8,
    pub to_room: u8,
    pub nudge: Vec2,
}

#[derive(Clone, Debug)]
pub struct RoomsState {
    pub current: u8,
    pub slide: Option<Slide>,
    pub shutter_closed: [bool; 16],
}

impl RoomsState {
    pub fn new(entry_room: u8) -> Self {
        Self {
            current: entry_room,
            slide: None,
            shutter_closed: [false; 16],
        }
    }
}

pub fn on_enter_dungeon(game: &mut Game) {
    let room = dungeon::room_at_tile(
        (game
            .world
            .get(game.world.player_id)
            .map(|p| p.pos.x / TILE_PX)
            .unwrap_or(49.0)) as u32,
        (game
            .world
            .get(game.world.player_id)
            .map(|p| p.pos.y / TILE_PX)
            .unwrap_or(76.0)) as u32,
    )
    .map(|r| r.id)
    .unwrap_or(0);
    game.rooms = Some(RoomsState::new(room));
    set_flag(&mut game.flags, content::flags::droom_flag(room));
    apply_camera_bounds(game);
    crate::puzzle::dungeon::paint_closed(game);
    crate::puzzle::dungeon::restore(game);
    #[cfg(debug_assertions)]
    assert_exit_reciprocity();
}

pub fn clear(game: &mut Game) {
    game.rooms = None;
    game.world.camera.clear_bounds();
}

pub fn update(game: &mut Game) -> bool {
    if game.current_map != MapId::Dungeon {
        return false;
    }
    let Some(rooms) = game.rooms.as_mut() else {
        return false;
    };
    if let Some(slide) = rooms.slide.as_mut() {
        slide.t = slide.t.saturating_add(1);
        let t = (slide.t as f32 / SLIDE_TICKS as f32).clamp(0.0, 1.0);
        let s = t * t * (3.0 - 2.0 * t);
        let (fx0, fy0, fx1, fy1) = slide.from_rect;
        let (tx0, ty0, tx1, ty1) = slide.to_rect;
        let min = Vec2::new(lerp(fx0, tx0, s), lerp(fy0, ty0, s));
        let max = Vec2::new(lerp(fx1, tx1, s), lerp(fy1, ty1, s));
        game.world.camera.set_bounds(Some((min, max)));
        // Nudge player through doorway.
        if let Some(p) = game.world.get_mut(game.world.player_id) {
            p.pos = p.pos.add(slide.nudge.scale(1.0 / SLIDE_TICKS as f32));
            p.vel = Vec2::ZERO;
        }
        if slide.t >= SLIDE_TICKS {
            let to = slide.to_room;
            rooms.current = to;
            rooms.slide = None;
            set_flag(&mut game.flags, content::flags::droom_flag(to));
            apply_camera_bounds(game);
            on_enter_room(game, to);
        }
        return true; // pause sim
    }

    try_room_exit(game);
    try_locked_door(game);
    apply_camera_bounds(game);
    false
}

fn on_enter_room(game: &mut Game, room: u8) {
    // Slam shutters for uncleared combat rooms.
    let group = match room {
        dungeon::ROOM_TRIALS_1 => Some(content::flags::GRP_DNG_TRIALS_1),
        dungeon::ROOM_TRIALS_2 => Some(content::flags::GRP_DNG_TRIALS_2),
        dungeon::ROOM_TRIALS_3 => Some(content::flags::GRP_DNG_TRIALS_3),
        _ => None,
    };
    if let Some(g) = group {
        if !crate::world::spawner::group_cleared(&game.spawner, g) {
            close_room_shutters(game, room);
            game.world
                .push_event(WorldEvent::Sfx(SfxId::ShutterSlam));
        }
    }
    crate::puzzle::dungeon::on_room_enter(game, room);
}

fn close_room_shutters(game: &mut Game, room: u8) {
    let Some(def) = dungeon::room_by_id(room) else {
        return;
    };
    for exit in def.exits.iter().flatten() {
        if matches!(exit.door, DoorKind::Shutter | DoorKind::Open) {
            // Keep return path? Slam all doors including entry — player fights.
            game.world
                .set_tile(TileLayer::Ground, exit.tx, exit.ty, catalog::D_SHUTTER);
        }
    }
    if let Some(rs) = game.rooms.as_mut() {
        if (room as usize) < rs.shutter_closed.len() {
            rs.shutter_closed[room as usize] = true;
        }
    }
}

pub fn open_room_shutters(game: &mut Game, room: u8) {
    let Some(def) = dungeon::room_by_id(room) else {
        return;
    };
    for exit in def.exits.iter().flatten() {
        if matches!(exit.door, DoorKind::Shutter | DoorKind::Open) {
            let tile = match exit.door {
                DoorKind::Open => catalog::D_DOOR_OPEN,
                _ => catalog::D_DOOR_OPEN,
            };
            game.world
                .set_tile(TileLayer::Ground, exit.tx, exit.ty, tile);
        }
    }
    if let Some(rs) = game.rooms.as_mut() {
        if (room as usize) < rs.shutter_closed.len() {
            rs.shutter_closed[room as usize] = false;
        }
    }
    game.world
        .push_event(WorldEvent::Sfx(SfxId::GateOpen));
}

fn try_room_exit(game: &mut Game) {
    let (px, py, feet) = {
        let Some(p) = game.world.get(game.world.player_id) else {
            return;
        };
        let feet = Vec2::new(p.pos.x + 8.0, p.pos.y + 14.0);
        (
            (feet.x / TILE_PX).floor() as u32,
            (feet.y / TILE_PX).floor() as u32,
            feet,
        )
    };
    let rooms = game.rooms.as_ref().unwrap();
    let Some(cur) = dungeon::room_by_id(rooms.current) else {
        return;
    };
    for (dir, exit) in cur.exits.iter().enumerate() {
        let Some(exit) = exit else { continue };
        if exit.tx != px || exit.ty != py {
            continue;
        }
        if !door_is_open(game, exit) {
            continue;
        }
        start_slide(game, cur, exit, dir as u8, feet);
        return;
    }
}

fn door_is_open(game: &Game, exit: &ExitDef) -> bool {
    let tile = game.world.map.get(exit.tx, exit.ty, TileLayer::Ground);
    if tile == catalog::D_DOOR_OPEN || tile == catalog::D_SEAL_BROKEN {
        return true;
    }
    match exit.door {
        DoorKind::Open => !game.world.map.solid_at(exit.tx as i32, exit.ty as i32),
        DoorKind::SmallKey => has_flag(&game.flags, content::flags::DDOOR_WING),
        DoorKind::InnerKey => has_flag(&game.flags, content::flags::DDOOR_INNER),
        DoorKind::BossKey => has_flag(&game.flags, content::flags::DDOOR_BOSS_USED),
        DoorKind::SealWest => has_flag(&game.flags, content::flags::SEAL_WEST),
        DoorKind::SealEast => has_flag(&game.flags, content::flags::SEAL_EAST),
        DoorKind::Shutter => false,
    }
}

fn try_locked_door(game: &mut Game) {
    let (px, py) = {
        let Some(p) = game.world.get(game.world.player_id) else {
            return;
        };
        let feet = Vec2::new(p.pos.x + 8.0, p.pos.y + 14.0);
        (
            (feet.x / TILE_PX).floor() as i32,
            (feet.y / TILE_PX).floor() as i32,
        )
    };
    // Check adjacent solids for locked doors the player is pressing into.
    for (dx, dy) in [(0i32, -1), (1, 0), (0, 1), (-1, 0)] {
        let tx = (px + dx) as u32;
        let ty = (py + dy) as u32;
        let tile = game.world.map.get(tx, ty, TileLayer::Ground);
        if tile != catalog::D_DOOR_LOCKED && tile != catalog::D_DOOR_BOSS {
            continue;
        }
        let Some(rooms) = game.rooms.as_ref() else {
            return;
        };
        let Some(cur) = dungeon::room_by_id(rooms.current) else {
            return;
        };
        let Some(exit) = cur
            .exits
            .iter()
            .flatten()
            .find(|e| e.tx == tx && e.ty == ty)
        else {
            continue;
        };
        match exit.door {
            DoorKind::SmallKey => {
                if small_keys_held(&game.flags) > 0 {
                    unlock_door(game, exit, content::flags::DDOOR_WING);
                } else {
                    locked_dialog(game);
                }
            }
            DoorKind::InnerKey => {
                if small_keys_held(&game.flags) > 0 {
                    unlock_door(game, exit, content::flags::DDOOR_INNER);
                } else {
                    locked_dialog(game);
                }
            }
            DoorKind::BossKey => {
                if has_flag(&game.flags, content::flags::DKEY_BOSS)
                    && !has_flag(&game.flags, content::flags::DDOOR_BOSS_USED)
                {
                    unlock_door(game, exit, content::flags::DDOOR_BOSS_USED);
                } else {
                    locked_dialog(game);
                }
            }
            _ => {}
        }
        return;
    }
}

fn unlock_door(game: &mut Game, exit: &ExitDef, flag: u16) {
    set_flag(&mut game.flags, flag);
    game.world
        .set_tile(TileLayer::Ground, exit.tx, exit.ty, catalog::D_DOOR_OPEN);
    // Reciprocal door tile.
    if let Some(other) = dungeon::room_by_id(exit.to_room) {
        for e in other.exits.iter().flatten() {
            if e.to_room
                == game
                    .rooms
                    .as_ref()
                    .map(|r| r.current)
                    .unwrap_or(0)
            {
                game.world
                    .set_tile(TileLayer::Ground, e.tx, e.ty, catalog::D_DOOR_OPEN);
            }
        }
    }
    game.world
        .push_event(WorldEvent::Sfx(SfxId::DoorUnlock));
    game.world.camera.add_shake(1.5, 6);
    game.world.push_event(WorldEvent::FxRequest(FxKind::Toast {
        text: "DOOR UNLOCKED",
    }));
}

fn locked_dialog(game: &mut Game) {
    game.ui.dialog.open_text(TextId::DoorLocked);
}

fn start_slide(game: &mut Game, from: &RoomDef, exit: &ExitDef, dir: u8, _feet: Vec2) {
    let Some(to) = dungeon::room_by_id(exit.to_room) else {
        return;
    };
    let nudge = match dir {
        0 => Vec2::new(0.0, -20.0),
        1 => Vec2::new(20.0, 0.0),
        2 => Vec2::new(0.0, 20.0),
        _ => Vec2::new(-20.0, 0.0),
    };
    let slide = Slide {
        from_rect: rect_px(from.rect),
        to_rect: rect_px(to.rect),
        t: 0,
        to_room: exit.to_room,
        nudge,
    };
    if let Some(rs) = game.rooms.as_mut() {
        rs.slide = Some(slide);
    }
    game.world
        .push_event(WorldEvent::Sfx(SfxId::RoomSlide));
}

fn apply_camera_bounds(game: &mut Game) {
    let Some(rooms) = game.rooms.as_ref() else {
        return;
    };
    if rooms.slide.is_some() {
        return;
    }
    let Some(room) = dungeon::room_by_id(rooms.current) else {
        return;
    };
    let (x0, y0, x1, y1) = rect_px(room.rect);
    game.world
        .camera
        .set_bounds(Some((Vec2::new(x0, y0), Vec2::new(x1, y1))));
}

fn rect_px(r: (u32, u32, u32, u32)) -> (f32, f32, f32, f32) {
    (
        r.0 as f32 * TILE_PX,
        r.1 as f32 * TILE_PX,
        (r.2 + 1) as f32 * TILE_PX,
        (r.3 + 1) as f32 * TILE_PX,
    )
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

/// Small keys held = chest flags set minus opened-door flags.
pub fn small_keys_held(flags: &[u16]) -> u8 {
    let mut n = 0u8;
    if has_flag(flags, content::flags::DKEY_SMALL_1) {
        n += 1;
    }
    if has_flag(flags, content::flags::DKEY_SMALL_2) {
        n += 1;
    }
    if has_flag(flags, content::flags::DDOOR_WING) {
        n = n.saturating_sub(1);
    }
    if has_flag(flags, content::flags::DDOOR_INNER) {
        n = n.saturating_sub(1);
    }
    n
}

#[cfg(debug_assertions)]
fn assert_exit_reciprocity() {
    for room in dungeon::rooms() {
        for exit in room.exits.iter().flatten() {
            let Some(other) = dungeon::room_by_id(exit.to_room) else {
                panic!("dungeon exit to missing room {}", exit.to_room);
            };
            let ok = other
                .exits
                .iter()
                .flatten()
                .any(|e| e.to_room == room.id);
            debug_assert!(ok, "exit reciprocity {} → {}", room.id, exit.to_room);
        }
    }
}
