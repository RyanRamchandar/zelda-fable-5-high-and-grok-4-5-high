//! Whispering Grove shell (8..84, 96..200).

use crate::maps::catalog::*;
use crate::maps::paint::path;
use crate::maps::{
    EntryPoint, MapDef, MapId, RegionDef, SpawnDef, SpawnKind, TileLayer, TriggerDef, TriggerKind,
};

pub fn paint(map: &mut MapDef) {
    // Dense tree-wall fill, then carve maze paths.
    for y in 96..200 {
        for x in 8..84 {
            if (x + y * 3) % 5 < 3 {
                place_tree(map, x, y);
            } else {
                map.set(x, y, TileLayer::Ground, T_GRASS_B);
            }
        }
    }

    // Carve loops / dead-ends.
    let corridors = [
        &[(40u32, 180), (40, 160), (40, 140), (40, 120), (50, 110), (60, 105), (70, 100)][..],
        &[(20, 170), (30, 170), (40, 170), (50, 170), (60, 170), (70, 165)][..],
        &[(25, 150), (35, 150), (45, 145), (55, 140), (55, 130), (45, 125)][..],
        &[(15, 130), (25, 125), (30, 115), (35, 110), (40, 105)][..],
        &[(60, 150), (65, 140), (70, 130), (72, 120), (74, 110), (76, 100)][..],
        &[(50, 190), (50, 180), (55, 175), (60, 180), (55, 185)][..], // loop
        &[(30, 190), (30, 185), (25, 185)][..],                       // dead-end
        &[(70, 190), (75, 185), (78, 180)][..],                       // dead-end
    ];
    for pts in corridors {
        path(map, pts, 2, T_PATH);
        // Clear trunks on path tiles.
        for w in pts.windows(2) {
            clear_path_strip(map, w[0], w[1]);
        }
    }

    // NE clearing reserved for Courage Gem (2B).
    map.fill_rect_layer(68, 108, 80, 122, TileLayer::Ground, T_GRASS_A);
    for y in 108..122 {
        for x in 68..80 {
            map.set(x, y, TileLayer::Overhang, T_VOID);
            map.set(x, y, TileLayer::Detail, T_VOID);
        }
    }

    // Cave mouth → fairy grotto near (76,100).
    map.set(76, 100, TileLayer::Ground, T_CAVE_MOUTH);
    map.set_flags(76, 100, 0);
    map.triggers.push(TriggerDef {
        tx: 76,
        ty: 100,
        w: 1,
        h: 1,
        kind: TriggerKind::Door {
            target: MapId::Cave(0),
            entry: 0,
        },
    });
    map.entries.push(EntryPoint {
        id: 30,
        tx: 76,
        ty: 102,
    });

    map.regions.push(RegionDef {
        name: "Whispering Grove",
        rect: (8, 96, 84, 200),
    });
    let ri = (map.regions.len() - 1) as u8;
    map.triggers.push(TriggerDef {
        tx: 30,
        ty: 175,
        w: 20,
        h: 8,
        kind: TriggerKind::Banner { region: ri },
    });
    map.triggers.push(TriggerDef {
        tx: 70,
        ty: 112,
        w: 6,
        h: 6,
        kind: TriggerKind::Checkpoint { id: 2 },
    });
    map.entries.push(EntryPoint {
        id: 2,
        tx: 72,
        ty: 114,
    });

    for (tx, ty, kind) in [
        (35u32, 160, SpawnKind::Slime),
        (45, 140, SpawnKind::Bat),
        (55, 155, SpawnKind::Bat),
        (25, 145, SpawnKind::Slime),
        (65, 125, SpawnKind::Octorok),
        (40, 115, SpawnKind::Bat),
        (70, 170, SpawnKind::Slime),
        (20, 165, SpawnKind::Bat),
    ] {
        map.spawns.push(SpawnDef {
            tx,
            ty,
            kind,
            group: 30,
        });
    }
}

fn place_tree(map: &mut MapDef, x: u32, y: u32) {
    map.set(x, y, TileLayer::Ground, T_TREE_TRUNK);
    if y >= 2 && x + 1 < map.width {
        map.set(x, y - 1, TileLayer::Overhang, T_CANOPY_SW);
        map.set(x + 1, y - 1, TileLayer::Overhang, T_CANOPY_SE);
        map.set(x, y - 2, TileLayer::Overhang, T_CANOPY_NW);
        map.set(x + 1, y - 2, TileLayer::Overhang, T_CANOPY_NE);
    }
}

fn clear_path_strip(map: &mut MapDef, a: (u32, u32), b: (u32, u32)) {
    let (x0, y0) = (a.0 as i32, a.1 as i32);
    let (x1, y1) = (b.0 as i32, b.1 as i32);
    let steps = (x1 - x0).abs().max((y1 - y0).abs()).max(1);
    for s in 0..=steps {
        let t = s as f32 / steps as f32;
        let cx = (x0 as f32 + (x1 - x0) as f32 * t) as i32;
        let cy = (y0 as f32 + (y1 - y0) as f32 * t) as i32;
        for dy in -1..=1 {
            for dx in -1..=1 {
                let x = cx + dx;
                let y = cy + dy;
                if map.in_bounds(x, y) {
                    let ux = x as u32;
                    let uy = y as u32;
                    map.set(ux, uy, TileLayer::Ground, T_PATH);
                    map.set(ux, uy, TileLayer::Overhang, T_VOID);
                }
            }
        }
    }
}
