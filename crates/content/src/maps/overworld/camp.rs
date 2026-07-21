//! Ashen Raider Camp — tents, bonfires, Power Gem chest + guard group.

use crate::flags;
use crate::maps::catalog::*;
use crate::maps::paint::{blob, path};
use crate::maps::{
    EntryPoint, Loot, MapDef, RegionDef, SpawnDef, SpawnKind, TileLayer, TriggerDef, TriggerKind,
};
use crate::text::TextId;

pub fn paint(map: &mut MapDef) {
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

    for x in 170..=226 {
        map.set(x, 42, TileLayer::Ground, T_FENCE);
        map.set(x, 98, TileLayer::Ground, T_FENCE);
    }
    for y in 42..=98 {
        map.set(170, y, TileLayer::Ground, T_FENCE);
        map.set(226, y, TileLayer::Ground, T_FENCE);
    }
    for x in 194..=200 {
        map.set(x, 98, TileLayer::Ground, T_DIRT);
    }
    for y in 66..=72 {
        map.set(170, y, TileLayer::Ground, T_DIRT);
    }

    blob(map, 190, 70, 4, T_DIRT_ASH, 0xCA11);
    blob(map, 210, 60, 3, T_DIRT_ASH, 0xCA12);
    map.set(190, 70, TileLayer::Ground, T_BONFIRE);
    map.set(210, 60, TileLayer::Ground, T_BONFIRE);
    map.set(200, 80, TileLayer::Ground, T_BONFIRE);

    // Tents + barricade cover previews (runtime paints T_BARRICADE at load).
    map.set(180, 55, TileLayer::Ground, T_TENT);
    map.set(220, 55, TileLayer::Ground, T_TENT);
    map.set(185, 85, TileLayer::Ground, T_TENT);
    map.set(215, 85, TileLayer::Detail, T_RUBBLE);
    map.set(205, 72, TileLayer::Detail, T_RUBBLE);

    // Watchtower + ladder tile (back).
    map.fill_rect(215, 50, 216, 51, T_HOUSE_WALL, true);
    map.set(214, 51, TileLayer::Ground, T_PATH); // ladder approach
    map.set(215, 49, TileLayer::Detail, T_LANTERN);
    map.spawns.push(SpawnDef {
        tx: 216,
        ty: 49,
        kind: SpawnKind::Chest {
            flag: flags::CHEST_CAMP_TOWER,
            loot: Loot::Rupees(50),
        },
        group: 0,
    });
    map.spawns.push(SpawnDef {
        tx: 215,
        ty: 48,
        kind: SpawnKind::Sign {
            text: TextId::WatchtowerView,
        },
        group: 0,
    });

    map.fill_rect_layer(198, 55, 208, 65, TileLayer::Ground, T_DIRT);
    map.set(203, 60, TileLayer::Ground, T_PEDESTAL);
    map.spawns.push(SpawnDef {
        tx: 203,
        ty: 60,
        kind: SpawnKind::Chest {
            flag: flags::CHEST_POWER_GEM,
            loot: Loot::Gem(1),
        },
        group: 0,
    });

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

    // Raider camp around bonfires + a few bat harassers.
    for (tx, ty, kind) in [
        (188u32, 72, SpawnKind::RaiderSpear),
        (192, 68, SpawnKind::RaiderTorch),
        (186, 66, SpawnKind::Bat),
        (212, 58, SpawnKind::RaiderSpear),
        (214, 64, SpawnKind::RaiderTorch),
        (198, 82, SpawnKind::RaiderSpear),
        (202, 78, SpawnKind::Bat),
        (196, 76, SpawnKind::RaiderTorch),
        (178, 60, SpawnKind::Bat),
        (220, 70, SpawnKind::RaiderSpear),
        (175, 80, SpawnKind::RaiderTorch),
    ] {
        map.spawns.push(SpawnDef {
            tx,
            ty,
            kind,
            group: flags::GRP_CAMP,
        });
    }
    // War-chest wave 1 (group 41): 3 spears + 2 bats + 1 torch.
    for (tx, ty, kind) in [
        (200u32, 58, SpawnKind::RaiderSpear),
        (206, 58, SpawnKind::RaiderSpear),
        (198, 61, SpawnKind::RaiderSpear),
        (200, 64, SpawnKind::Bat),
        (206, 64, SpawnKind::Bat),
        (208, 61, SpawnKind::RaiderTorch),
    ] {
        map.spawns.push(SpawnDef {
            tx,
            ty,
            kind,
            group: flags::GRP_CAMP_GUARD,
        });
    }
    // Wave 2 (42): 2 spears + 2 torches — locked until 41 clears.
    for (tx, ty, kind) in [
        (199u32, 56, SpawnKind::RaiderSpear),
        (207, 56, SpawnKind::RaiderSpear),
        (199, 66, SpawnKind::RaiderTorch),
        (207, 66, SpawnKind::RaiderTorch),
    ] {
        map.spawns.push(SpawnDef {
            tx,
            ty,
            kind,
            group: flags::GRP_CAMP_W2,
        });
    }
    // Wave 3 (43): 3 spears + 1 torch + 1 skeleton veteran.
    for (tx, ty, kind) in [
        (198u32, 57, SpawnKind::RaiderSpear),
        (203, 55, SpawnKind::RaiderSpear),
        (208, 57, SpawnKind::RaiderSpear),
        (203, 65, SpawnKind::RaiderTorch),
        (205, 62, SpawnKind::Skeleton),
    ] {
        map.spawns.push(SpawnDef {
            tx,
            ty,
            kind,
            group: flags::GRP_CAMP_W3,
        });
    }
}
