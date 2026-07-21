//! Razor Cliffs shell (40..160, 16..92) — elevation bands + ledge hops.

use crate::flags;
use crate::maps::catalog::*;
use crate::maps::paint::path;
use crate::maps::{
    EntryPoint, MapDef, MapId, RegionDef, SpawnDef, SpawnKind, TileLayer, TriggerDef, TriggerKind,
};

pub fn paint(map: &mut MapDef) {
    // Base cliff face fill for bands.
    for y in 16..92 {
        for x in 40..160 {
            let band = if y < 40 {
                2
            } else if y < 64 {
                1
            } else {
                0
            };
            let tile = match band {
                2 => T_CLIFF_TOP,
                1 => T_CLIFF_FACE,
                _ => T_DIRT,
            };
            map.set(x, y, TileLayer::Ground, tile);
        }
    }

    // River source pool.
    map.fill_rect_layer(92, 24, 100, 32, TileLayer::Ground, T_WATER_DEEP);
    map.set(96, 28, TileLayer::Ground, T_WATER_SHIMMER);
    for x in 91..=101 {
        map.set(x, 23, TileLayer::Ground, T_SHORE_N);
        map.set(x, 33, TileLayer::Ground, T_SHORE_S);
    }

    // Switchback paths with stairs.
    path(
        map,
        &[
            (80, 88),
            (90, 80),
            (100, 72),
            (90, 64),
            (100, 56),
            (110, 48),
            (100, 40),
            (110, 32),
            (120, 28),
        ],
        2,
        T_PATH,
    );
    // Stairs markers on steep segments.
    for (x, y) in [(90u32, 80), (100, 72), (90, 64), (100, 56), (110, 48), (100, 40)] {
        map.set(x, y, TileLayer::Ground, T_CLIFF_STAIRS);
        map.set(x + 1, y, TileLayer::Ground, T_CLIFF_STAIRS);
    }

    // ≥3 LEDGE_S hop points shortcutting down.
    for (x, y) in [(70u32, 50), (85, 58), (115, 45), (130, 60)] {
        map.set(x, y, TileLayer::Ground, T_LEDGE_S);
        map.set(x + 1, y, TileLayer::Ground, T_LEDGE_S);
        // Landing pads south.
        map.set(x, y + 1, TileLayer::Ground, T_PATH);
        map.set(x + 1, y + 1, TileLayer::Ground, T_PATH);
        map.set(x, y + 2, TileLayer::Ground, T_PATH);
        map.set(x + 1, y + 2, TileLayer::Ground, T_PATH);
    }

    // Summit pocket + cave mouth (hidden notch) + birds telegraph.
    map.fill_rect_layer(112, 20, 130, 30, TileLayer::Ground, T_CLIFF_TOP);
    map.fill_rect_layer(116, 22, 126, 28, TileLayer::Ground, T_PATH);
    map.set(120, 22, TileLayer::Ground, T_CAVE_MOUTH);
    map.set_flags(120, 22, 0);
    map.set(120, 20, TileLayer::Overhang, T_BIRDS);
    map.set(100, 30, TileLayer::Ground, T_SHRINE_STONE); // vista interact
    map.triggers.push(TriggerDef {
        tx: 120,
        ty: 22,
        w: 1,
        h: 1,
        kind: TriggerKind::Door {
            target: MapId::Cave(1),
            entry: 0,
        },
    });
    map.entries.push(EntryPoint {
        id: 31,
        tx: 120,
        ty: 24,
    });

    // Soften cliff edges along path.
    for x in 40..160 {
        map.set(x, 16, TileLayer::Ground, T_CLIFF_EDGE_N);
    }

    map.regions.push(RegionDef {
        name: "Razor Cliffs",
        rect: (40, 16, 160, 92),
    });
    let ri = (map.regions.len() - 1) as u8;
    map.triggers.push(TriggerDef {
        tx: 70,
        ty: 84,
        w: 30,
        h: 6,
        kind: TriggerKind::Banner { region: ri },
    });
    map.triggers.push(TriggerDef {
        tx: 78,
        ty: 86,
        w: 6,
        h: 4,
        kind: TriggerKind::Checkpoint { id: 5 },
    });
    map.entries.push(EntryPoint {
        id: 5,
        tx: 80,
        ty: 88,
    });

    // Octorok showcase + one summit wisp (~12).
    for (tx, ty, kind) in [
        (95u32, 55, SpawnKind::Octorok),
        (100, 35, SpawnKind::Octorok),
        (110, 32, SpawnKind::Octorok),
        (85, 40, SpawnKind::Octorok),
        (130, 50, SpawnKind::Octorok),
        (75, 70, SpawnKind::Slime),
        (60, 80, SpawnKind::Slime),
        (110, 65, SpawnKind::Slime),
        (140, 75, SpawnKind::Slime),
        (90, 80, SpawnKind::Bat),
        (120, 48, SpawnKind::Bat),
        (100, 30, SpawnKind::Wisp),
    ] {
        map.spawns.push(SpawnDef {
            tx,
            ty,
            kind,
            group: flags::GRP_CLIFFS,
        });
    }
}
