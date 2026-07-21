//! Furnished interiors + shrine lobby stub.

use super::catalog::*;
use super::{
    EntryPoint, Loot, MapDef, MapId, SpawnDef, SpawnKind, TileLayer, TriggerDef, TriggerKind,
};
use crate::flags;
use crate::text::{NpcId, TextId};

fn room(w: u32, h: u32, exit_entry: u8) -> MapDef {
    let mut map = MapDef::new(w, h, T_INT_FLOOR);
    map.rect_border(T_INT_WALL, true);
    let door_x = w / 2;
    map.set(door_x, h - 1, TileLayer::Ground, T_CAVE_MOUTH);
    map.set_flags(door_x, h - 1, 0);
    map.entries.push(EntryPoint {
        id: 0,
        tx: door_x,
        ty: h.saturating_sub(3),
    });
    map.triggers.push(TriggerDef {
        tx: door_x,
        ty: h - 1,
        w: 1,
        h: 1,
        kind: TriggerKind::Door {
            target: MapId::Overworld,
            entry: exit_entry,
        },
    });
    map
}

pub fn house_for(n: u8) -> MapDef {
    let mut map = room(12, 10, 10 + n.min(5));
    if let Some(t) = map.triggers.first_mut() {
        if let TriggerKind::Door { entry, .. } = &mut t.kind {
            *entry = 10 + n.min(5);
        }
    }
    // Distinct furniture layouts.
    match n % 6 {
        0 => {
            map.set(2, 2, TileLayer::Detail, T_BED);
            map.set(8, 2, TileLayer::Detail, T_SHELF);
            map.set(5, 4, TileLayer::Detail, T_TABLE);
            map.set(5, 5, TileLayer::Detail, T_RUG);
        }
        1 => {
            map.set(3, 2, TileLayer::Detail, T_SHELF);
            map.set(8, 3, TileLayer::Detail, T_BED);
            map.set(2, 6, TileLayer::Detail, T_POT);
            map.set(6, 5, TileLayer::Detail, T_RUG);
        }
        2 => {
            map.set(2, 3, TileLayer::Detail, T_TABLE);
            map.set(7, 2, TileLayer::Detail, T_BED);
            map.set(9, 6, TileLayer::Detail, T_SHELF);
        }
        3 => {
            map.set(4, 2, TileLayer::Detail, T_BED);
            map.set(8, 5, TileLayer::Detail, T_TABLE);
            map.set(2, 5, TileLayer::Detail, T_POT);
            map.set(5, 6, TileLayer::Detail, T_RUG);
        }
        4 => {
            map.set(2, 2, TileLayer::Detail, T_SHELF);
            map.set(2, 4, TileLayer::Detail, T_SHELF);
            map.set(8, 2, TileLayer::Detail, T_BED);
            map.set(5, 5, TileLayer::Detail, T_TABLE);
        }
        _ => {
            map.set(6, 2, TileLayer::Detail, T_BED);
            map.set(3, 4, TileLayer::Detail, T_TABLE);
            map.set(8, 6, TileLayer::Detail, T_POT);
            map.set(5, 6, TileLayer::Detail, T_RUG);
        }
    }
    map
}

pub fn shop() -> MapDef {
    let mut map = room(16, 12, 16);
    map.entries[0].tx = 8;
    map.entries[0].ty = 9;
    map.set(4, 3, TileLayer::Detail, T_COUNTER);
    map.set(5, 3, TileLayer::Detail, T_COUNTER);
    map.set(6, 3, TileLayer::Detail, T_COUNTER);
    map.set(2, 2, TileLayer::Detail, T_SHELF);
    map.set(12, 2, TileLayer::Detail, T_SHELF);
    map.set(10, 5, TileLayer::Detail, T_POT);
    map.spawns.push(SpawnDef {
        tx: 5,
        ty: 4,
        kind: SpawnKind::Npc {
            npc: NpcId::Shopkeeper,
        },
        group: 0,
    });
    map
}

pub fn cave_grotto() -> MapDef {
    let mut map = room(14, 12, 30);
    for (fx, fy) in [(6u32, 5), (7, 5), (6, 6), (7, 6)] {
        map.set(fx, fy, TileLayer::Ground, T_FOUNTAIN);
    }
    map.set(6, 5, TileLayer::Detail, T_BASIN);
    map.spawns.push(SpawnDef {
        tx: 6,
        ty: 5,
        kind: SpawnKind::FairyFountain,
        group: 0,
    });
    map
}

pub fn cave_heart() -> MapDef {
    let mut map = room(14, 12, 31);
    // Winding passage feel.
    map.fill_rect(2, 2, 4, 8, T_INT_WALL, true);
    map.fill_rect(9, 3, 11, 9, T_INT_WALL, true);
    map.set(5, 4, TileLayer::Ground, T_INT_FLOOR);
    map.set(6, 4, TileLayer::Ground, T_INT_FLOOR);
    map.set(7, 5, TileLayer::Ground, T_INT_FLOOR);
    map.set(8, 6, TileLayer::Ground, T_INT_FLOOR);
    map.spawns.push(SpawnDef {
        tx: 7,
        ty: 3,
        kind: SpawnKind::Chest {
            flag: flags::CHEST_CLIFFS_HEART,
            loot: Loot::HeartPiece,
        },
        group: 0,
    });
    map
}

pub fn shrine_lobby() -> MapDef {
    let mut map = room(16, 12, 6);
    // Return to overworld shrine entry 6.
    if let Some(t) = map.triggers.first_mut() {
        if let TriggerKind::Door { entry, .. } = &mut t.kind {
            *entry = 6;
        }
    }
    map.set(8, 4, TileLayer::Ground, T_PEDESTAL);
    map.set(4, 3, TileLayer::Ground, T_BRAZIER);
    map.set(11, 3, TileLayer::Ground, T_BRAZIER);
    map.spawns.push(SpawnDef {
        tx: 8,
        ty: 5,
        kind: SpawnKind::Sign {
            text: TextId::ShrineLobby,
        },
        group: 0,
    });
    map
}
