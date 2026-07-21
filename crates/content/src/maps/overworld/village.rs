//! Mosslight Village shell (92..152, 148..212).

use crate::maps::catalog::*;
use crate::maps::paint::{self, path};
use crate::maps::{
    EntryPoint, MapDef, MapId, RegionDef, SpawnDef, SpawnKind, TileLayer, TriggerDef, TriggerKind,
};

pub fn paint(map: &mut MapDef) {
    // Cleared plaza.
    map.fill_rect_layer(100, 160, 144, 200, TileLayer::Ground, T_PATH);
    map.fill_rect_layer(108, 168, 136, 192, TileLayer::Ground, T_DIRT);

    // Path grid.
    path(map, &[(120, 200), (120, 160)], 2, T_PATH);
    path(map, &[(100, 180), (144, 180)], 2, T_PATH);
    path(map, &[(110, 170), (110, 195)], 1, T_PATH);
    path(map, &[(130, 170), (130, 195)], 1, T_PATH);

    // Fence / hedge border with 3 gates.
    for x in 96..=148 {
        if x != 118 && x != 119 && x != 120 {
            // south gate opening
            map.set(x, 208, TileLayer::Ground, T_FENCE);
        }
        map.set(x, 152, TileLayer::Ground, T_FENCE);
    }
    for y in 152..=208 {
        // west gate → grove
        if y != 178 && y != 179 && y != 180 {
            map.set(96, y, TileLayer::Ground, T_FENCE);
        }
        // east solid
        map.set(148, y, TileLayer::Ground, T_FENCE);
    }
    // North gate → bridge (opening)
    for x in 116..=124 {
        map.set(x, 152, TileLayer::Ground, T_PATH);
    }
    // West gate opening
    for y in 177..=181 {
        map.set(96, y, TileLayer::Ground, T_PATH);
    }
    // South gate
    for x in 116..=122 {
        map.set(x, 208, TileLayer::Ground, T_PATH);
    }

    // 6 house footprints + shop + fountain.
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

    // Village door return landings (south of each door).
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

    for (tx, ty, kind) in [
        (108u32, 170, SpawnKind::Slime),
        (140, 190, SpawnKind::Slime),
        (112, 195, SpawnKind::Bat),
        (132, 172, SpawnKind::Octorok),
        (100, 185, SpawnKind::Slime),
        (145, 175, SpawnKind::Bat),
    ] {
        map.spawns.push(SpawnDef {
            tx,
            ty,
            kind,
            group: 20,
        });
    }
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
    map.spawns.push(SpawnDef {
        tx: ox + 1,
        ty: oy + 1,
        kind: SpawnKind::FairyFountain,
        group: 0,
    });
}
