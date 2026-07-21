//! Dungeon minimap: discovered rooms + exit reciprocity from `RoomDef` table.

use content::maps::dungeon::{self, DoorKind};
use content::maps::MapId;
use engine::render::Draw;

use crate::save_data::has_flag;
use crate::Game;

const OX: f32 = 360.0;
const OY: f32 = 8.0;
const CELL: f32 = 10.0;

pub fn render_corner(d: &mut Draw, game: &Game) {
    if game.current_map != MapId::Dungeon {
        return;
    }
    let Some(rooms) = game.rooms.as_ref() else {
        return;
    };
    let cur = rooms.current;
    // Compact chip: current + explored neighbors.
    let blink = (game.world.tick / 20).is_multiple_of(2);
    for room in dungeon::rooms() {
        if !has_flag(&game.flags, content::flags::droom_flag(room.id)) && room.id != cur {
            continue;
        }
        let (gx, gy) = grid_pos(room.id);
        let x = OX + gx as f32 * CELL;
        let y = OY + gy as f32 * CELL;
        let col = if room.id == cur && blink {
            "#ffff80"
        } else if room.id == cur {
            "#e0e040"
        } else {
            "#606878"
        };
        d.rect(x, y, CELL - 1.0, CELL - 1.0, col);
        for exit in room.exits.iter().flatten() {
            let mark = match exit.door {
                DoorKind::BossKey => "#ff6040",
                DoorKind::SealWest | DoorKind::SealEast => "#80e0ff",
                DoorKind::SmallKey | DoorKind::InnerKey => "#c0a040",
                _ => "#a0a8b0",
            };
            d.rect(x + 3.0, y + 3.0, 2.0, 2.0, mark);
        }
    }
    if has_flag(&game.flags, content::flags::ITEM_BOOMERANG) {
        d.text("B", OX - 12.0, OY + 2.0, "#40e0c0");
    }
    let keys = crate::rooms::small_keys_held(&game.flags);
    if keys > 0 {
        d.text(&format!("K{keys}"), OX - 12.0, OY + 14.0, "#e8d060");
    }
}

pub fn render_pause(d: &mut Draw, game: &Game) {
    if game.current_map != MapId::Dungeon {
        return;
    }
    d.rect(40.0, 30.0, 400.0, 210.0, "rgba(10,14,22,0.92)");
    d.text("DUNGEON MAP", 180.0, 48.0, "#e8e8e8");
    let Some(rooms) = game.rooms.as_ref() else {
        return;
    };
    for room in dungeon::rooms() {
        let known = has_flag(&game.flags, content::flags::droom_flag(room.id));
        if !known && room.id != rooms.current {
            continue;
        }
        let (gx, gy) = grid_pos(room.id);
        let x = 80.0 + gx as f32 * 36.0;
        let y = 70.0 + gy as f32 * 28.0;
        let col = if room.id == rooms.current {
            "#e0e040"
        } else {
            "#505868"
        };
        d.rect(x, y, 30.0, 22.0, col);
        // Exit pips from the same table as physical doors.
        for (i, exit) in room.exits.iter().enumerate() {
            let Some(exit) = exit else { continue };
            let (px, py) = match i {
                0 => (x + 12.0, y - 3.0),
                1 => (x + 28.0, y + 8.0),
                2 => (x + 12.0, y + 20.0),
                _ => (x - 3.0, y + 8.0),
            };
            let col = match exit.door {
                DoorKind::BossKey => "#ff6040",
                DoorKind::SealWest | DoorKind::SealEast => "#80e0ff",
                DoorKind::SmallKey | DoorKind::InnerKey => "#c0a040",
                DoorKind::Shutter => "#a07070",
                DoorKind::Open => "#c0c8d0",
            };
            d.rect(px, py, 4.0, 4.0, col);
        }
    }
}

fn grid_pos(id: u8) -> (i32, i32) {
    match id {
        dungeon::ROOM_SANCTUM => (2, 0),
        dungeon::ROOM_ARENA => (3, 0),
        dungeon::ROOM_SEAL_W => (1, 1),
        dungeon::ROOM_ANTECHAMBER => (2, 1),
        dungeon::ROOM_SEAL_E => (3, 1),
        dungeon::ROOM_CURRENTS_LINES => (3, 2),
        dungeon::ROOM_MULTI => (4, 2),
        dungeon::ROOM_TRIALS_3 => (0, 3),
        dungeon::ROOM_TRIALS_2 => (1, 3),
        dungeon::ROOM_CURRENTS_TEACH => (3, 3),
        dungeon::ROOM_CURRENTS_RANGE => (4, 3),
        dungeon::ROOM_BOOMERANG => (0, 4),
        dungeon::ROOM_TRIALS_1 => (1, 4),
        dungeon::ROOM_VESTIBULE => (2, 4),
        dungeon::ROOM_CURRENTS_HUB => (3, 4),
        dungeon::ROOM_FLAME => (4, 4),
        _ => (0, 0),
    }
}
