//! River, roads, bridges, south meadow.

use crate::flags;
use crate::maps::catalog::*;
use crate::maps::paint::{self, path, river, scatter, scatter_detail};
use crate::maps::{
    EntryPoint, Loot, MapDef, RegionDef, SpawnDef, SpawnKind, TileLayer, TriggerDef, TriggerKind,
};
use crate::text::TextId;

pub fn paint(map: &mut MapDef) {
    // Base meadow grass variance.
    scatter(map, 84, 196, 232, 236, T_GRASS_B, 0.35, 0x51ADE);
    scatter_detail(map, 84, 196, 232, 236, T_GRASS_FLOWER, 0.04, 0xF10);
    scatter_detail(map, 84, 196, 232, 236, T_GRASS_PEBBLE, 0.03, 0xF11);

    // Tree clusters in south meadow.
    for (cx, cy, r, seed) in [
        (100u32, 220, 4u32, 1u32),
        (140, 228, 5, 2),
        (180, 218, 4, 3),
        (210, 225, 3, 4),
        (95, 205, 3, 5),
    ] {
        paint::blob(map, cx, cy, r, T_GRASS_B, seed);
        stamp_tree_cluster(map, cx, cy);
    }

    // River: source ~ (96,28) south then east to ~(238,150).
    let river_pts = [
        (96, 28),
        (96, 50),
        (94, 80),
        (90, 110),
        (100, 130),
        (120, 142),
        (150, 145),
        (178, 148),
        (200, 150),
        (220, 150),
        (238, 150),
    ];
    river(map, &river_pts, 5, T_WATER_DEEP, Some(T_SHORE_N));
    // Shimmer accents along river.
    for &(x, y) in &[(96u32, 40), (94, 90), (130, 143), (200, 150)] {
        map.set(x, y, TileLayer::Ground, T_WATER_SHIMMER);
        map.set(x + 1, y, TileLayer::Ground, T_WATER_SHIMMER);
    }

    // Intact bridges: village north ~(120,142), ruins approach ~(178,148).
    stamp_bridge_h(map, 118, 141);
    stamp_bridge_h(map, 176, 147);

    // Broken bridge grove→cliffs ~(66,94) + river island via ledge hops.
    stamp_bridge_broken(map, 64, 93);
    map.set(67, 91, TileLayer::Ground, T_CRANK);
    map.spawns.push(SpawnDef {
        tx: 68,
        ty: 95,
        kind: SpawnKind::Sign {
            text: TextId::CrankSign,
        },
        group: 0,
    });
    map.set(62, 92, TileLayer::Ground, T_LEDGE_S);
    map.set(63, 92, TileLayer::Ground, T_LEDGE_S);
    map.fill_rect_layer(66, 96, 70, 100, TileLayer::Ground, T_GRASS_A);
    map.set(68, 98, TileLayer::Ground, T_PATH);
    map.spawns.push(SpawnDef {
        tx: 68,
        ty: 98,
        kind: SpawnKind::Chest {
            flag: flags::CHEST_RIVER_ISLAND,
            loot: Loot::Rupees(25),
        },
        group: 0,
    });

    // Meadow flower ring (secret #9).
    for (dx, dy) in [
        (0i32, -2),
        (2, -1),
        (2, 1),
        (0, 2),
        (-2, 1),
        (-2, -1),
    ] {
        let x = (150i32 + dx) as u32;
        let y = (220i32 + dy) as u32;
        map.set(x, y, TileLayer::Ground, T_GRASS_FLOWER);
    }
    map.set(150, 220, TileLayer::Ground, T_GRASS_A);
    map.triggers.push(TriggerDef {
        tx: 149,
        ty: 219,
        w: 3,
        h: 3,
        kind: TriggerKind::Secret {
            flag: flags::SECRET_MEADOW_FLOWERS,
        },
    });

    // Dirt roads: village ↔ regions (winding).
    path(
        map,
        &[(120, 180), (120, 160), (120, 148), (120, 142)],
        2,
        T_PATH,
    );
    path(
        map,
        &[(100, 180), (80, 170), (60, 150), (50, 130), (40, 120)],
        2,
        T_PATH,
    );
    path(
        map,
        &[(140, 180), (160, 170), (180, 160), (190, 140), (200, 120)],
        2,
        T_PATH,
    );
    path(
        map,
        &[(120, 148), (140, 140), (160, 130), (180, 120), (190, 100)],
        2,
        T_PATH,
    );
    path(
        map,
        &[(120, 148), (110, 120), (100, 90), (100, 60), (110, 40)],
        2,
        T_PATH,
    );
    // South gate road.
    path(map, &[(118, 206), (118, 190), (120, 180)], 3, T_PATH);

    map.regions.push(RegionDef {
        name: "South Meadow",
        rect: (84, 196, 232, 236),
    });
    let meadow_ri = (map.regions.len() - 1) as u8;
    map.triggers.push(TriggerDef {
        tx: 100,
        ty: 200,
        w: 40,
        h: 8,
        kind: TriggerKind::Banner {
            region: meadow_ri,
        },
    });

    // New Game entry — village south gate.
    map.entries.push(EntryPoint {
        id: 0,
        tx: 118,
        ty: 206,
    });

    // Sparse meadow / road hostiles (off spine).
    for (tx, ty, kind) in [
        (130u32, 220, SpawnKind::Slime),
        (160, 215, SpawnKind::Slime),
        (200, 225, SpawnKind::Bat),
        (95, 218, SpawnKind::Slime),
    ] {
        map.spawns.push(SpawnDef {
            tx,
            ty,
            kind,
            group: flags::GRP_MEADOW,
        });
    }
}

fn stamp_bridge_h(map: &mut MapDef, ox: u32, oy: u32) {
    let rows = ["BBB", "BBB"];
    let legend = [('B', TileLayer::Ground, T_BRIDGE_H)];
    paint::stamp(map, ox, oy, &rows, &legend);
    // Clear water collision under bridge tiles already walkable via catalog.
}

fn stamp_bridge_broken(map: &mut MapDef, ox: u32, oy: u32) {
    // Ends solid, gap in middle (water remains).
    map.set(ox, oy, TileLayer::Ground, T_BRIDGE_BROKEN);
    map.set(ox + 1, oy, TileLayer::Ground, T_BRIDGE_BROKEN);
    map.set(ox + 4, oy, TileLayer::Ground, T_BRIDGE_BROKEN);
    map.set(ox + 5, oy, TileLayer::Ground, T_BRIDGE_BROKEN);
    map.set(ox, oy + 1, TileLayer::Ground, T_BRIDGE_BROKEN);
    map.set(ox + 5, oy + 1, TileLayer::Ground, T_BRIDGE_BROKEN);
}

fn stamp_tree_cluster(map: &mut MapDef, cx: u32, cy: u32) {
    for (dx, dy) in [(0i32, 0), (2, 1), (-2, 0), (1, -2)] {
        let x = (cx as i32 + dx).max(2) as u32;
        let y = (cy as i32 + dy).max(2) as u32;
        map.set(x, y, TileLayer::Ground, T_TREE_TRUNK);
        map.set(x, y.saturating_sub(1), TileLayer::Overhang, T_CANOPY_SW);
        map.set(x + 1, y.saturating_sub(1), TileLayer::Overhang, T_CANOPY_SE);
        map.set(x, y.saturating_sub(2), TileLayer::Overhang, T_CANOPY_NW);
        map.set(x + 1, y.saturating_sub(2), TileLayer::Overhang, T_CANOPY_NE);
    }
}
