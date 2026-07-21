//! Gray-box combat arena for Phase 1 feel testing (MapDef v2).

use super::catalog;
use super::{EntryPoint, MapDef, SpawnDef, SpawnKind, TileLayer};

pub use catalog::{FLOOR, FOUNTAIN, WALL};

/// 60×34 tiles (960×544 px) — ~2×2 screens with room for camera travel.
pub fn arena() -> MapDef {
    use catalog::*;
    let mut map = MapDef::new(60, 34, T_FLOOR_A);
    map.rect_border(T_WALL, true);

    // Checkerboard ground for readability.
    for y in 1..map.height - 1 {
        for x in 1..map.width - 1 {
            let tile = if (x + y) % 2 == 0 {
                T_FLOOR_A
            } else {
                T_FLOOR_B
            };
            map.set(x, y, TileLayer::Ground, tile);
        }
    }

    // Interior 2×2 pillars.
    for (px, py) in [(18u32, 10), (40, 10), (18, 22), (40, 22), (28, 14)] {
        map.fill_rect(px, py, px + 1, py + 1, T_PILLAR, true);
    }

    // Fountain corner (NW open pocket).
    map.fill_rect(3, 3, 6, 6, T_FLOOR_A, false);
    for (fx, fy) in [(4u32, 4), (5, 4), (4, 5), (5, 5)] {
        map.set(fx, fy, TileLayer::Ground, T_FOUNTAIN);
    }

    // Terrain strip (M2 DoD) along south edge inside border.
    let strip_y = 31u32;
    let samples = [
        T_GRASS_A,
        T_GRASS_B,
        T_PATH,
        T_DIRT,
        T_SAND,
        T_WATER_DEEP,
        T_WATER_SHIMMER,
        T_CLIFF_FACE,
        T_CLIFF_STAIRS,
        T_LEDGE_S,
        T_TREE_TRUNK,
        T_BRIDGE_H,
        T_FENCE,
        T_SHRINE_STONE,
    ];
    for (i, &tile) in samples.iter().enumerate() {
        let x = 2 + i as u32 * 2;
        if x + 1 < map.width - 1 {
            map.set(x, strip_y, TileLayer::Ground, tile);
            map.set(x + 1, strip_y, TileLayer::Ground, tile);
        }
    }
    // Ledge hop test: solid ledge facing south with clear landing.
    map.set(48, 28, TileLayer::Ground, T_LEDGE_S);
    map.set(49, 28, TileLayer::Ground, T_LEDGE_S);
    map.set(48, 29, TileLayer::Ground, T_FLOOR_A);
    map.set(49, 29, TileLayer::Ground, T_FLOOR_A);
    map.set(48, 30, TileLayer::Ground, T_FLOOR_A);
    map.set(49, 30, TileLayer::Ground, T_FLOOR_A);

    map.entries.push(EntryPoint {
        id: 0,
        tx: 30,
        ty: 18,
    });
    map.spawns.push(SpawnDef {
        tx: 4,
        ty: 4,
        kind: SpawnKind::FairyFountain,
        group: 0,
    });
    for (tx, ty) in [(25u32, 15), (32, 16), (28, 20)] {
        map.spawns.push(SpawnDef {
            tx,
            ty,
            kind: SpawnKind::Dummy,
            group: 1,
        });
    }

    map
}
