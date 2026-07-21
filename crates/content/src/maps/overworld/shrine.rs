//! Triforce Shrine — pedestals, braziers, soft seal gate.

use crate::flags;
use crate::maps::catalog::*;
use crate::maps::paint::{path, stamp};
use crate::maps::{
    EntryPoint, MapDef, RegionDef, SpawnDef, SpawnKind, TileLayer, TriggerDef, TriggerKind,
};
use crate::text::TextId;

pub fn paint(map: &mut MapDef) {
    for y in 4..32 {
        for x in 96..144 {
            map.set(x, y, TileLayer::Ground, T_SAND);
        }
    }
    map.fill_rect_layer(104, 8, 136, 28, TileLayer::Ground, T_PATH);
    path(map, &[(120, 30), (120, 20), (120, 12)], 3, T_PATH);

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

    // Pedestal trio + braziers.
    map.set(114, 14, TileLayer::Ground, T_PEDESTAL);
    map.set(120, 14, TileLayer::Ground, T_PEDESTAL);
    map.set(126, 14, TileLayer::Ground, T_PEDESTAL);
    map.set(112, 14, TileLayer::Ground, T_BRAZIER);
    map.set(128, 14, TileLayer::Ground, T_BRAZIER);

    map.set(110, 18, TileLayer::Ground, T_COLUMN);
    map.set(130, 18, TileLayer::Ground, T_COLUMN);
    map.set(110, 18, TileLayer::Overhang, T_ARCH_TOP);
    map.set(130, 18, TileLayer::Overhang, T_ARCH_TOP);

    map.spawns.push(SpawnDef {
        tx: 120,
        ty: 16,
        kind: SpawnKind::Sign {
            text: TextId::ShrineLore,
        },
        group: 0,
    });
    map.spawns.push(SpawnDef {
        tx: 122,
        ty: 18,
        kind: SpawnKind::Sign {
            text: TextId::TwinFlames,
        },
        group: 0,
    });

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

    // Soft-gate sentinels framing the road (groups away from CP).
    for (tx, ty, kind) in [
        (108u32, 28, SpawnKind::Octorok),
        (106, 26, SpawnKind::Slime),
        (110, 30, SpawnKind::Bat),
        (132, 28, SpawnKind::Octorok),
        (134, 26, SpawnKind::Slime),
        (130, 30, SpawnKind::Bat),
    ] {
        map.spawns.push(SpawnDef {
            tx,
            ty,
            kind,
            group: flags::GRP_SHRINE,
        });
    }
}
