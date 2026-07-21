//! Triforce Shrine dungeon — one contiguous MapDef with camera-locked rooms.

use super::catalog::*;
use super::dungeon_rooms;
use super::{
    EntryPoint, Loot, MapDef, MapId, SpawnDef, SpawnKind, TileLayer, TriggerDef, TriggerKind,
};
use crate::flags;
use crate::text::TextId;

pub use dungeon_rooms::*;

const W: u32 = 100;
const H: u32 = 80;

pub fn build() -> MapDef {
    let mut map = MapDef::new(W, H, D_WALL);
    // Carve rooms.
    for room in rooms() {
        let (x0, y0, x1, y1) = room.rect;
        fill_room(&mut map, x0, y0, x1, y1);
        paint_exits(&mut map, room);
    }
    // Vestibule south door → shrine lobby.
    map.set(49, 79, TileLayer::Ground, D_DOOR_OPEN);
    map.set_flags(49, 79, 0);
    map.set(50, 79, TileLayer::Ground, D_DOOR_OPEN);
    map.set_flags(50, 79, 0);

    map.entries.push(EntryPoint {
        id: 0,
        tx: 49,
        ty: 76,
    });
    map.entries.push(EntryPoint {
        id: 7,
        tx: 49,
        ty: 72,
    });
    map.entries.push(EntryPoint {
        id: 8,
        tx: 49,
        ty: 40,
    });
    // Pre-boss checkpoint (antechamber, just south of Sanctum shutter).
    map.entries.push(EntryPoint {
        id: 9,
        tx: 49,
        ty: 20,
    });

    map.triggers.push(TriggerDef {
        tx: 49,
        ty: 79,
        w: 2,
        h: 1,
        kind: TriggerKind::Door {
            target: MapId::ShrineLobby,
            entry: 1,
        },
    });
    map.triggers.push(TriggerDef {
        tx: 48,
        ty: 70,
        w: 4,
        h: 2,
        kind: TriggerKind::Checkpoint { id: 7 },
    });
    map.triggers.push(TriggerDef {
        tx: 48,
        ty: 38,
        w: 4,
        h: 2,
        kind: TriggerKind::Checkpoint { id: 8 },
    });
    map.triggers.push(TriggerDef {
        tx: 48,
        ty: 18,
        w: 4,
        h: 2,
        kind: TriggerKind::Checkpoint { id: 9 },
    });

    dress_vestibule(&mut map);
    dress_trials(&mut map);
    dress_boomerang(&mut map);
    dress_currents(&mut map);
    dress_flame(&mut map);
    dress_seals(&mut map);
    dress_antechamber(&mut map);
    dress_sanctum_arena(&mut map);

    map.regions.push(super::RegionDef {
        name: "Triforce Shrine",
        rect: (0, 0, W - 1, H - 1),
    });
    map
}

fn fill_room(map: &mut MapDef, x0: u32, y0: u32, x1: u32, y1: u32) {
    for y in y0..=y1 {
        for x in x0..=x1 {
            let edge = x == x0 || x == x1 || y == y0 || y == y1;
            if edge {
                map.set(x, y, TileLayer::Ground, D_WALL);
            } else if (x + y) % 5 == 0 {
                map.set(x, y, TileLayer::Ground, D_FLOOR_B);
            } else {
                map.set(x, y, TileLayer::Ground, D_FLOOR_A);
            }
        }
    }
}

fn paint_exits(map: &mut MapDef, room: &RoomDef) {
    for exit in room.exits.iter().flatten() {
        let tile = match exit.door {
            DoorKind::Open => D_DOOR_OPEN,
            DoorKind::SmallKey | DoorKind::InnerKey => D_DOOR_LOCKED,
            DoorKind::BossKey => D_DOOR_BOSS,
            DoorKind::SealWest | DoorKind::SealEast => D_SEAL_DOOR,
            DoorKind::Shutter => D_SHUTTER,
        };
        map.set(exit.tx, exit.ty, TileLayer::Ground, tile);
        if matches!(exit.door, DoorKind::Open) {
            map.set_flags(exit.tx, exit.ty, 0);
        }
    }
}

fn dress_vestibule(map: &mut MapDef) {
    map.set(45, 68, TileLayer::Detail, D_BRAZIER_ETERNAL);
    map.set(54, 68, TileLayer::Detail, D_BRAZIER_ETERNAL);
    map.spawns.push(SpawnDef {
        tx: 49,
        ty: 69,
        kind: SpawnKind::Sign {
            text: TextId::DungeonLore,
        },
        group: 0,
    });
}

fn dress_trials(map: &mut MapDef) {
    // Trials 1: slime + skeleton
    map.spawns.push(SpawnDef {
        tx: 26,
        ty: 70,
        kind: SpawnKind::Slime,
        group: flags::GRP_DNG_TRIALS_1,
    });
    map.spawns.push(SpawnDef {
        tx: 32,
        ty: 72,
        kind: SpawnKind::Skeleton,
        group: flags::GRP_DNG_TRIALS_1,
    });
    // Trials 2: raiders + plate
    map.set(29, 55, TileLayer::Ground, T_PLATE_UP);
    map.spawns.push(SpawnDef {
        tx: 24,
        ty: 52,
        kind: SpawnKind::RaiderSpear,
        group: flags::GRP_DNG_TRIALS_2,
    });
    map.spawns.push(SpawnDef {
        tx: 34,
        ty: 54,
        kind: SpawnKind::RaiderTorch,
        group: flags::GRP_DNG_TRIALS_2,
    });
    // Trials 3: 1 wisp + octorok (Phase 5: drop second wisp for shutter density).
    map.spawns.push(SpawnDef {
        tx: 6,
        ty: 52,
        kind: SpawnKind::Wisp,
        group: flags::GRP_DNG_TRIALS_3,
    });
    map.spawns.push(SpawnDef {
        tx: 10,
        ty: 54,
        kind: SpawnKind::Octorok,
        group: flags::GRP_DNG_TRIALS_3,
    });
    map.spawns.push(SpawnDef {
        tx: 8,
        ty: 58,
        kind: SpawnKind::Chest {
            flag: flags::DCHEST_KEY1,
            loot: Loot::SmallKey,
        },
        group: 0,
    });
}

fn dress_boomerang(map: &mut MapDef) {
    map.set(9, 70, TileLayer::Ground, D_FLOOR_RUNE);
    map.spawns.push(SpawnDef {
        tx: 9,
        ty: 69,
        kind: SpawnKind::Chest {
            flag: flags::DCHEST_BOOMERANG,
            loot: Loot::Boomerang,
        },
        group: 0,
    });
    map.spawns.push(SpawnDef {
        tx: 12,
        ty: 72,
        kind: SpawnKind::Chest {
            flag: flags::DCHEST_RUPEE_A,
            loot: Loot::Rupees(50),
        },
        group: 0,
    });
}

fn dress_currents(map: &mut MapDef) {
    map.spawns.push(SpawnDef {
        tx: 69,
        ty: 70,
        kind: SpawnKind::Sign {
            text: TextId::CurrentsSign,
        },
        group: 0,
    });
    // Teach: crystal beside gate
    map.set(72, 54, TileLayer::Ground, D_CRYSTAL_BLUE);
    map.set(66, 55, TileLayer::Ground, D_GATE_BLUE_UP);
    map.set(66, 56, TileLayer::Ground, D_GATE_BLUE_UP);
    // Range: water channel + crystal across
    for x in 84..96 {
        map.set(x, 55, TileLayer::Ground, D_WATER);
        map.set(x, 56, TileLayer::Ground, D_WATER);
    }
    map.set(85, 52, TileLayer::Ground, D_CRYSTAL_AMBER);
    map.set(92, 58, TileLayer::Ground, D_GATE_AMBER_UP);
    map.set(93, 58, TileLayer::Ground, D_GATE_AMBER_UP);
    // Lines: interleaved gates
    map.set(64, 38, TileLayer::Ground, D_CRYSTAL_BLUE);
    map.set(74, 40, TileLayer::Ground, D_CRYSTAL_AMBER);
    map.set(69, 36, TileLayer::Ground, D_GATE_BLUE_UP);
    map.set(69, 42, TileLayer::Ground, D_GATE_AMBER_UP);
    // Multi: twin crystals same-state
    map.set(84, 38, TileLayer::Ground, D_CRYSTAL_BLUE);
    map.set(94, 40, TileLayer::Ground, D_CRYSTAL_BLUE);
    map.set(89, 36, TileLayer::Ground, D_GATE_BLUE_UP);
    map.set(90, 36, TileLayer::Ground, D_GATE_BLUE_UP);
    map.spawns.push(SpawnDef {
        tx: 70,
        ty: 58,
        kind: SpawnKind::Bat,
        group: flags::GRP_DNG_CURRENTS,
    });
}

fn dress_flame(map: &mut MapDef) {
    map.set(89, 70, TileLayer::Ground, D_BRAZIER_ETERNAL);
    map.set(84, 68, TileLayer::Ground, D_TORCH_UNLIT);
    map.set(94, 72, TileLayer::Ground, D_TORCH_UNLIT);
    map.set(89, 66, TileLayer::Ground, D_GATE_BLUE_UP);
    map.set(90, 66, TileLayer::Ground, D_GATE_BLUE_UP);
    map.spawns.push(SpawnDef {
        tx: 89,
        ty: 68,
        kind: SpawnKind::Sign {
            text: TextId::FlameSign,
        },
        group: 0,
    });
    map.spawns.push(SpawnDef {
        tx: 89,
        ty: 71,
        kind: SpawnKind::Chest {
            flag: flags::DCHEST_KEY2,
            loot: Loot::SmallKey,
        },
        group: 0,
    });
}

fn dress_seals(map: &mut MapDef) {
    // West: corner-bend return
    map.set(24, 20, TileLayer::Ground, D_RUNE_1);
    map.set(28, 20, TileLayer::Ground, D_RUNE_2);
    map.set(32, 26, TileLayer::Ground, D_RUNE_3);
    map.set(24, 22, TileLayer::Ground, D_CRYSTAL_BLUE);
    map.set(28, 22, TileLayer::Ground, D_CRYSTAL_BLUE);
    map.set(32, 24, TileLayer::Ground, D_CRYSTAL_BLUE);
    map.set(26, 28, TileLayer::Ground, D_FLOOR_RUNE);
    map.spawns.push(SpawnDef {
        tx: 26,
        ty: 18,
        kind: SpawnKind::Sign {
            text: TextId::SealSignW,
        },
        group: 0,
    });
    // East: cross-water line
    for x in 64..76 {
        map.set(x, 23, TileLayer::Ground, D_WATER);
    }
    map.set(64, 18, TileLayer::Ground, D_RUNE_1);
    map.set(69, 18, TileLayer::Ground, D_RUNE_2);
    map.set(74, 26, TileLayer::Ground, D_RUNE_3);
    map.set(64, 20, TileLayer::Ground, D_CRYSTAL_AMBER);
    map.set(69, 20, TileLayer::Ground, D_CRYSTAL_AMBER);
    map.set(74, 24, TileLayer::Ground, D_CRYSTAL_AMBER);
    map.set(69, 28, TileLayer::Ground, D_FLOOR_RUNE);
    map.spawns.push(SpawnDef {
        tx: 72,
        ty: 18,
        kind: SpawnKind::Sign {
            text: TextId::SealSignE,
        },
        group: 0,
    });
}

fn dress_antechamber(map: &mut MapDef) {
    map.set(49, 40, TileLayer::Ground, D_LIFT);
    map.set(45, 36, TileLayer::Detail, D_BRAZIER_ETERNAL);
    map.set(53, 36, TileLayer::Detail, D_BRAZIER_ETERNAL);
    map.spawns.push(SpawnDef {
        tx: 49,
        ty: 42,
        kind: SpawnKind::Chest {
            flag: flags::DCHEST_BOSSKEY,
            loot: Loot::BossKey,
        },
        group: 0,
    });
    map.spawns.push(SpawnDef {
        tx: 52,
        ty: 50,
        kind: SpawnKind::Chest {
            flag: flags::DCHEST_RUPEE_B,
            loot: Loot::Rupees(30),
        },
        group: 0,
    });
}

fn dress_sanctum_arena(map: &mut MapDef) {
    map.set(49, 8, TileLayer::Ground, D_FLOOR_RUNE);
    // Ironshell duo — group locked until Sanctum entry unlocks.
    map.spawns.push(SpawnDef {
        tx: 46,
        ty: 10,
        kind: SpawnKind::Ironshell,
        group: flags::GRP_DNG_SANCTUM,
    });
    map.spawns.push(SpawnDef {
        tx: 52,
        ty: 10,
        kind: SpawnKind::Ironshell,
        group: flags::GRP_DNG_SANCTUM,
    });
    // Arena floor + crystal perches (E/W). Boss spawns at runtime.
    map.set(69, 7, TileLayer::Ground, D_FLOOR_RUNE);
    map.set(62, 7, TileLayer::Detail, D_CRYSTAL_BLUE);
    map.set(76, 7, TileLayer::Detail, D_CRYSTAL_AMBER);
    for tx in 61..79 {
        map.set(tx, 1, TileLayer::Ground, D_WALL_TOP);
        map.set(tx, 13, TileLayer::Ground, D_WALL_TOP);
    }
}
