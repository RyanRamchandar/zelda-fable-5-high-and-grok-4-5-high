//! Triforce Shrine shell (96..144, 4..32).

use crate::maps::catalog::*;
use crate::maps::paint::{path, stamp};
use crate::maps::{
    EntryPoint, MapDef, RegionDef, SpawnDef, SpawnKind, TileLayer, TriggerDef, TriggerKind,
};

pub fn paint(map: &mut MapDef) {
    // Stone approach + vista terrace.
    for y in 4..32 {
        for x in 96..144 {
            map.set(x, y, TileLayer::Ground, T_SAND);
        }
    }
    map.fill_rect_layer(104, 8, 136, 28, TileLayer::Ground, T_PATH);
    path(map, &[(120, 30), (120, 20), (120, 12)], 3, T_PATH);

    // Shrine facade.
    let facade = [
        "SSSSSSS",
        "S.....S",
        "S..D..S",
        "S.....S",
        "SSSSSSS",
    ];
    let legend = [
        ('S', TileLayer::Ground, T_SHRINE_STONE),
        ('D', TileLayer::Ground, T_DOOR_SEALED),
    ];
    stamp(map, 117, 8, &facade, &legend);

    // Approach pillars.
    map.set(110, 18, TileLayer::Ground, T_COLUMN);
    map.set(130, 18, TileLayer::Ground, T_COLUMN);
    map.set(110, 18, TileLayer::Overhang, T_ARCH_TOP);
    map.set(130, 18, TileLayer::Overhang, T_ARCH_TOP);

    map.regions.push(RegionDef {
        name: "Triforce Shrine",
        rect: (96, 4, 144, 32),
    });
    let ri = (map.regions.len() - 1) as u8;
    map.triggers.push(TriggerDef {
        tx: 110,
        ty: 26,
        w: 20,
        h: 4,
        kind: TriggerKind::Banner { region: ri },
    });
    map.triggers.push(TriggerDef {
        tx: 116,
        ty: 20,
        w: 8,
        h: 4,
        kind: TriggerKind::Checkpoint { id: 6 },
    });
    map.entries.push(EntryPoint {
        id: 6,
        tx: 120,
        ty: 22,
    });

    for (tx, ty, kind) in [
        (108u32, 24, SpawnKind::Bat),
        (132, 24, SpawnKind::Bat),
        (115, 16, SpawnKind::Octorok),
        (125, 16, SpawnKind::Slime),
        (105, 12, SpawnKind::Bat),
        (135, 14, SpawnKind::Octorok),
    ] {
        map.spawns.push(SpawnDef {
            tx,
            ty,
            kind,
            group: 70,
        });
    }
}
