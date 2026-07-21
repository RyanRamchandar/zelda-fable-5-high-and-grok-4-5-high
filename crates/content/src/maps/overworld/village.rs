//! Mosslight Village — decorated + NPCs / signs / chest (Phase 2B).

use crate::flags;
use crate::maps::catalog::*;
use crate::maps::paint::{self, path};
use crate::maps::{
    EntryPoint, Loot, MapDef, MapId, RegionDef, SpawnDef, SpawnKind, TileLayer, TriggerDef,
    TriggerKind,
};
use crate::text::{NpcId, TextId};

pub fn paint(map: &mut MapDef) {
    map.fill_rect_layer(100, 160, 144, 200, TileLayer::Ground, T_PATH);
    map.fill_rect_layer(108, 168, 136, 192, TileLayer::Ground, T_DIRT);

    path(map, &[(120, 200), (120, 160)], 2, T_PATH);
    path(map, &[(100, 180), (144, 180)], 2, T_PATH);
    path(map, &[(110, 170), (110, 195)], 1, T_PATH);
    path(map, &[(130, 170), (130, 195)], 1, T_PATH);

    for x in 96..=148 {
        if x != 118 && x != 119 && x != 120 {
            map.set(x, 208, TileLayer::Ground, T_FENCE);
        }
        map.set(x, 152, TileLayer::Ground, T_FENCE);
    }
    for y in 152..=208 {
        if y != 178 && y != 179 && y != 180 {
            map.set(96, y, TileLayer::Ground, T_FENCE);
        }
        map.set(148, y, TileLayer::Ground, T_FENCE);
    }
    for x in 116..=124 {
        map.set(x, 152, TileLayer::Ground, T_PATH);
    }
    for y in 177..=181 {
        map.set(96, y, TileLayer::Ground, T_PATH);
    }
    for x in 116..=122 {
        map.set(x, 208, TileLayer::Ground, T_PATH);
    }

    // Hedge gap behind shop (secret #5).
    map.set(122, 190, TileLayer::Ground, T_PATH);
    map.set(123, 190, TileLayer::Ground, T_PATH);
    map.set(123, 191, TileLayer::Ground, T_DIRT);

    let houses = [
        (104u32, 164u32, 0u8),
        (114, 164, 1),
        (124, 164, 2),
        (134, 164, 3),
        (104, 186, 4),
        (134, 186, 5),
    ];
    for (hx, hy, n) in houses {
        stamp_house(map, hx, hy, n);
    }
    stamp_shop(map, 118, 186);
    stamp_fountain(map, 118, 174);

    // Lantern-lined paths + flower beds + stall.
    for &(x, y) in &[
        (112u32, 200),
        (128, 200),
        (112, 180),
        (128, 180),
        (120, 160),
        (100, 180),
        (140, 180),
    ] {
        map.set(x, y, TileLayer::Detail, T_LANTERN);
    }
    map.set(110, 172, TileLayer::Detail, T_FLOWER_BED);
    map.set(130, 172, TileLayer::Detail, T_FLOWER_BED);
    map.set(108, 190, TileLayer::Detail, T_FLOWER_BED);
    map.set(136, 190, TileLayer::Detail, T_FLOWER_BED);
    map.set(126, 178, TileLayer::Ground, T_STALL);

    map.regions.push(RegionDef {
        name: "Mosslight Village",
        rect: (92, 148, 152, 212),
    });
    let ri = (map.regions.len() - 1) as u8;
    map.triggers.push(TriggerDef {
        tx: 110,
        ty: 200,
        w: 20,
        h: 6,
        kind: TriggerKind::Banner { region: ri },
    });
    map.triggers.push(TriggerDef {
        tx: 118,
        ty: 174,
        w: 4,
        h: 4,
        kind: TriggerKind::Checkpoint { id: 1 },
    });
    map.entries.push(EntryPoint {
        id: 1,
        tx: 120,
        ty: 178,
    });

    for (i, &(hx, hy, _)) in houses.iter().enumerate() {
        map.entries.push(EntryPoint {
            id: 10 + i as u8,
            tx: hx + 1,
            ty: hy + 4,
        });
    }
    map.entries.push(EntryPoint {
        id: 16,
        tx: 120,
        ty: 191,
    });

    // Signs
    for (tx, ty, text) in [
        (118u32, 204, TextId::VillageWelcome),
        (120, 192, TextId::ShopSign),
        (120, 154, TextId::WaypostShrine),
        (116, 176, TextId::FountainLore),
    ] {
        map.spawns.push(SpawnDef {
            tx,
            ty,
            kind: SpawnKind::Sign { text },
            group: 0,
        });
    }

    // NPCs (zero hostiles in village).
    for (tx, ty, npc) in [
        (122u32, 176, NpcId::Elder),
        (110, 182, NpcId::VillagerA),
        (132, 184, NpcId::VillagerB),
        (106, 170, NpcId::VillagerC),
        (128, 196, NpcId::Kid),
    ] {
        map.spawns.push(SpawnDef {
            tx,
            ty,
            kind: SpawnKind::Npc { npc },
            group: 0,
        });
    }

    // Test chest near plaza + hedge secret chest.
    map.spawns.push(SpawnDef {
        tx: 114,
        ty: 178,
        kind: SpawnKind::Chest {
            flag: flags::CHEST_VILLAGE_TEST,
            loot: Loot::Rupees(5),
        },
        group: 0,
    });
    map.spawns.push(SpawnDef {
        tx: 124,
        ty: 191,
        kind: SpawnKind::Chest {
            flag: flags::CHEST_SHOP_HEDGE,
            loot: Loot::Rupees(20),
        },
        group: 0,
    });
}

fn stamp_house(map: &mut MapDef, ox: u32, oy: u32, n: u8) {
    let rows = ["WWW", "WDW", "WWW", "..."];
    let legend = [
        ('W', TileLayer::Ground, T_HOUSE_WALL),
        ('D', TileLayer::Ground, T_HOUSE_DOOR),
    ];
    paint::stamp(map, ox, oy, &rows, &legend);
    map.triggers.push(TriggerDef {
        tx: ox + 1,
        ty: oy + 1,
        w: 1,
        h: 1,
        kind: TriggerKind::Door {
            target: MapId::House(n),
            entry: 0,
        },
    });
}

fn stamp_shop(map: &mut MapDef, ox: u32, oy: u32) {
    let rows = ["WWWW", "WDDW", "WWWW"];
    let legend = [
        ('W', TileLayer::Ground, T_HOUSE_WALL),
        ('D', TileLayer::Ground, T_HOUSE_DOOR),
    ];
    paint::stamp(map, ox, oy, &rows, &legend);
    map.triggers.push(TriggerDef {
        tx: ox + 1,
        ty: oy + 1,
        w: 2,
        h: 1,
        kind: TriggerKind::Door {
            target: MapId::Shop,
            entry: 0,
        },
    });
}

fn stamp_fountain(map: &mut MapDef, ox: u32, oy: u32) {
    for dy in 0..3u32 {
        for dx in 0..3u32 {
            map.set(ox + dx, oy + dy, TileLayer::Ground, T_DIRT);
        }
    }
    map.set(ox + 1, oy + 1, TileLayer::Ground, T_FOUNTAIN);
    map.set(ox + 1, oy + 1, TileLayer::Detail, T_BASIN);
    map.spawns.push(SpawnDef {
        tx: ox + 1,
        ty: oy + 1,
        kind: SpawnKind::FairyFountain,
        group: 0,
    });
}
