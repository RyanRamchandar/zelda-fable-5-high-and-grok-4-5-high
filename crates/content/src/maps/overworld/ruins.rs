//! Echoing Ruins — columns, Wisdom Gem, octorok lanes.

use crate::flags;
use crate::maps::catalog::*;
use crate::maps::paint::{path, stamp};
use crate::maps::{
    EntryPoint, Loot, MapDef, RegionDef, SpawnDef, SpawnKind, TileLayer, TriggerDef, TriggerKind,
};
use crate::text::TextId;

pub fn paint(map: &mut MapDef) {
    for y in 110..184 {
        for x in 156..232 {
            let tile = if (x + y) % 3 == 0 { T_SAND } else { T_DIRT };
            map.set(x, y, TileLayer::Ground, tile);
        }
    }

    for gy in 0..5 {
        for gx in 0..6 {
            let x = 164 + gx * 10;
            let y = 120 + gy * 10;
            if (gx + gy) % 2 == 0 {
                map.set(x, y, TileLayer::Ground, T_COLUMN);
            }
        }
    }

    let arch = ["C.C", ".A.", "..."];
    let legend = [
        ('C', TileLayer::Ground, T_COLUMN),
        ('A', TileLayer::Overhang, T_ARCH_TOP),
    ];
    stamp(map, 180, 130, &arch, &legend);
    stamp(map, 200, 150, &arch, &legend);
    stamp(map, 170, 160, &arch, &legend);

    // Plate court + Wisdom Gem.
    map.fill_rect_layer(186, 140, 210, 160, TileLayer::Ground, T_SAND);
    for x in (188..208).step_by(4) {
        for y in (142..158).step_by(4) {
            map.set(x, y, TileLayer::Detail, T_RUBBLE); // inert plates
        }
    }
    map.set(198, 150, TileLayer::Ground, T_PEDESTAL);
    map.spawns.push(SpawnDef {
        tx: 198,
        ty: 150,
        kind: SpawnKind::Gem { id: 2 },
        group: 0,
    });
    map.spawns.push(SpawnDef {
        tx: 192,
        ty: 146,
        kind: SpawnKind::Sign {
            text: TextId::RuinsTablet,
        },
        group: 0,
    });

    // Collapsed cellar secret — walk behind rubble.
    map.set(168, 168, TileLayer::Detail, T_RUBBLE);
    map.set(169, 168, TileLayer::Ground, T_PATH);
    map.set_flags(168, 168, 0);
    map.spawns.push(SpawnDef {
        tx: 167,
        ty: 169,
        kind: SpawnKind::Chest {
            flag: flags::CHEST_RUINS_CELLAR,
            loot: Loot::HeartPiece,
        },
        group: 0,
    });

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

    // Octorok lanes + bats (~12).
    for (tx, ty, kind) in [
        (170u32, 125, SpawnKind::Octorok),
        (180, 125, SpawnKind::Octorok),
        (200, 135, SpawnKind::Bat),
        (210, 145, SpawnKind::Octorok),
        (185, 155, SpawnKind::Bat),
        (205, 155, SpawnKind::Octorok),
        (175, 170, SpawnKind::Bat),
        (220, 140, SpawnKind::Octorok),
        (195, 120, SpawnKind::Bat),
        (165, 150, SpawnKind::Octorok),
        (215, 160, SpawnKind::Bat),
        (200, 170, SpawnKind::Octorok),
    ] {
        map.spawns.push(SpawnDef {
            tx,
            ty,
            kind,
            group: flags::GRP_RUINS,
        });
    }
}
