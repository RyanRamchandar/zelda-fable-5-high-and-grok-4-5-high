//! Interior stub rooms (Phase 2A shells — 2B furnishes).

use super::catalog::{T_CAVE_MOUTH, T_FOUNTAIN, T_INT_FLOOR, T_INT_WALL};
use super::{
    EntryPoint, MapDef, MapId, SpawnDef, SpawnKind, TileLayer, TriggerDef, TriggerKind,
};

fn room(w: u32, h: u32, exit_entry: u8) -> MapDef {
    let mut map = MapDef::new(w, h, T_INT_FLOOR);
    map.rect_border(T_INT_WALL, true);
    // Exit door centered on south wall.
    let door_x = w / 2;
    map.set(door_x, h - 1, TileLayer::Ground, T_CAVE_MOUTH);
    map.set_flags(door_x, h - 1, 0);
    // Spawn just north of the door (outside return trigger).
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
    // Fix return entry to matching overworld house door landing.
    if let Some(t) = map.triggers.first_mut() {
        if let TriggerKind::Door { entry, .. } = &mut t.kind {
            *entry = 10 + n.min(5);
        }
    }
    map
}

pub fn shop() -> MapDef {
    let mut map = room(16, 12, 16);
    map.entries[0].tx = 8;
    map.entries[0].ty = 9;
    map
}

pub fn cave_grotto() -> MapDef {
    let mut map = room(14, 12, 30);
    // Fairy fountain in center.
    for (fx, fy) in [(6u32, 5), (7, 5), (6, 6), (7, 6)] {
        map.set(fx, fy, TileLayer::Ground, T_FOUNTAIN);
    }
    map.spawns.push(SpawnDef {
        tx: 6,
        ty: 5,
        kind: SpawnKind::FairyFountain,
        group: 0,
    });
    map
}

pub fn cave_heart() -> MapDef {
    room(14, 12, 31)
}
