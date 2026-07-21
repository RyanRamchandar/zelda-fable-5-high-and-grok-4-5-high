//! Ashen Raider Camp shell (164..232, 36..104).

use crate::maps::catalog::*;
use crate::maps::paint::{blob, path};
use crate::maps::{
    EntryPoint, MapDef, RegionDef, SpawnDef, SpawnKind, TileLayer, TriggerDef, TriggerKind,
};

pub fn paint(map: &mut MapDef) {
    // Scorched dirt base.
    for y in 36..104 {
        for x in 164..232 {
            let tile = if (x * 3 + y * 7) % 11 < 4 {
                T_DIRT_ASH
            } else {
                T_DIRT
            };
            map.set(x, y, TileLayer::Ground, tile);
        }
    }

    // Palisade ring with 2 openings (south + west).
    for x in 170..=226 {
        map.set(x, 42, TileLayer::Ground, T_FENCE);
        map.set(x, 98, TileLayer::Ground, T_FENCE);
    }
    for y in 42..=98 {
        map.set(170, y, TileLayer::Ground, T_FENCE);
        map.set(226, y, TileLayer::Ground, T_FENCE);
    }
    // Openings
    for x in 194..=200 {
        map.set(x, 98, TileLayer::Ground, T_DIRT);
    }
    for y in 66..=72 {
        map.set(170, y, TileLayer::Ground, T_DIRT);
    }

    // Bonfire clearings.
    blob(map, 190, 70, 4, T_DIRT_ASH, 0xCA11);
    blob(map, 210, 60, 3, T_DIRT_ASH, 0xCA12);
    map.set(190, 70, TileLayer::Ground, T_SHRINE_STONE); // bonfire marker

    // Watchtower 2×2.
    map.fill_rect(215, 50, 216, 51, T_HOUSE_WALL, true);

    // Inner war-chest clearing reserved (2B).
    map.fill_rect_layer(198, 55, 208, 65, TileLayer::Ground, T_DIRT);

    path(map, &[(197, 98), (197, 80), (197, 60)], 2, T_PATH);
    path(map, &[(170, 69), (185, 69), (197, 69)], 2, T_PATH);

    map.regions.push(RegionDef {
        name: "Ashen Raider Camp",
        rect: (164, 36, 232, 104),
    });
    let ri = (map.regions.len() - 1) as u8;
    map.triggers.push(TriggerDef {
        tx: 190,
        ty: 90,
        w: 16,
        h: 8,
        kind: TriggerKind::Banner { region: ri },
    });
    map.triggers.push(TriggerDef {
        tx: 194,
        ty: 96,
        w: 6,
        h: 3,
        kind: TriggerKind::Checkpoint { id: 3 },
    });
    map.entries.push(EntryPoint {
        id: 3,
        tx: 197,
        ty: 94,
    });

    for (tx, ty, kind) in [
        (180u32, 75, SpawnKind::Octorok),
        (200, 80, SpawnKind::Slime),
        (210, 70, SpawnKind::Bat),
        (185, 55, SpawnKind::Octorok),
        (220, 85, SpawnKind::Slime),
        (195, 50, SpawnKind::Bat),
        (205, 90, SpawnKind::Octorok),
        (175, 60, SpawnKind::Slime),
    ] {
        map.spawns.push(SpawnDef {
            tx,
            ty,
            kind,
            group: 40,
        });
    }
}
