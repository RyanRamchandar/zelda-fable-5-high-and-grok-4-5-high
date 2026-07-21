//! Echoing Ruins shell (156..232, 110..184).

use crate::maps::catalog::*;
use crate::maps::paint::{path, stamp};
use crate::maps::{
    EntryPoint, MapDef, RegionDef, SpawnDef, SpawnKind, TileLayer, TriggerDef, TriggerKind,
};

pub fn paint(map: &mut MapDef) {
    // Sand / cracked stone ground.
    for y in 110..184 {
        for x in 156..232 {
            let tile = if (x + y) % 3 == 0 { T_SAND } else { T_DIRT };
            map.set(x, y, TileLayer::Ground, tile);
        }
    }

    // Broken column grid.
    for gy in 0..5 {
        for gx in 0..6 {
            let x = 164 + gx * 10;
            let y = 120 + gy * 10;
            if (gx + gy) % 2 == 0 {
                map.set(x, y, TileLayer::Ground, T_COLUMN);
            }
        }
    }

    // Collapsed arch prefabs (overhang tops).
    let arch = ["C.C", ".A.", "..."];
    let legend = [
        ('C', TileLayer::Ground, T_COLUMN),
        ('A', TileLayer::Overhang, T_ARCH_TOP),
    ];
    stamp(map, 180, 130, &arch, &legend);
    stamp(map, 200, 150, &arch, &legend);
    stamp(map, 170, 160, &arch, &legend);

    // Plaza reserved for plate court (2B).
    map.fill_rect_layer(186, 140, 210, 160, TileLayer::Ground, T_SAND);

    path(map, &[(178, 148), (190, 150), (200, 145), (210, 140)], 2, T_PATH);
    path(map, &[(190, 180), (190, 160), (190, 150)], 2, T_PATH);

    map.regions.push(RegionDef {
        name: "Echoing Ruins",
        rect: (156, 110, 232, 184),
    });
    let ri = (map.regions.len() - 1) as u8;
    map.triggers.push(TriggerDef {
        tx: 180,
        ty: 165,
        w: 24,
        h: 8,
        kind: TriggerKind::Banner { region: ri },
    });
    map.triggers.push(TriggerDef {
        tx: 192,
        ty: 148,
        w: 8,
        h: 8,
        kind: TriggerKind::Checkpoint { id: 4 },
    });
    map.entries.push(EntryPoint {
        id: 4,
        tx: 196,
        ty: 150,
    });

    for (tx, ty, kind) in [
        (170u32, 125, SpawnKind::Octorok),
        (200, 135, SpawnKind::Bat),
        (185, 155, SpawnKind::Slime),
        (210, 165, SpawnKind::Octorok),
        (175, 170, SpawnKind::Bat),
        (220, 140, SpawnKind::Slime),
        (195, 120, SpawnKind::Octorok),
        (165, 150, SpawnKind::Bat),
    ] {
        map.spawns.push(SpawnDef {
            tx,
            ty,
            kind,
            group: 50,
        });
    }
}
