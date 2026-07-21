//! Pure puzzle-site data (coordinates + flags). Runtime lives in `game::puzzle`.

use crate::flags;
use crate::maps::catalog::{
    T_BRIDGE_LOWERED, T_CAVE_MOUTH, T_GATE, T_PATH, T_SAND,
};
use crate::maps::MapId;

#[derive(Clone, Copy, Debug)]
pub struct ChimeGateDef {
    pub chime: (u32, u32),
    pub gate: &'static [(u32, u32)],
    pub open_tile: u16,
    pub flag: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct ChimeFinaleDef {
    pub chimes: [(u32, u32); 3],
    pub window_ticks: u32,
    pub flag: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct PlateCourtDef {
    pub plates: [(u32, u32); 2],
    pub blocks: [(u32, u32); 2],
    pub gate: &'static [(u32, u32)],
    pub open_tile: u16,
    pub floor_tile: u16,
    pub flag: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct BarricadeDef {
    pub tiles: &'static [(u32, u32)],
    pub floor_tile: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct CrankDef {
    pub crank: (u32, u32),
    /// `(tx, ty, new_ground_tile)`
    pub swaps: &'static [(u32, u32, u16)],
    pub flag: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct BombWallDef {
    pub wall: (u32, u32),
    pub open_tile: u16,
    pub door: (MapId, u8),
    pub flag: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct FarSwitchDef {
    pub crank: (u32, u32),
    pub gate: &'static [(u32, u32)],
    pub open_tile: u16,
    pub flag: u16,
}

#[derive(Clone, Copy, Debug)]
pub struct OverworldPuzzles {
    pub chime_gates: &'static [ChimeGateDef],
    pub chime_finale: ChimeFinaleDef,
    pub plate_court: PlateCourtDef,
    pub barricades: &'static [BarricadeDef],
    pub bridge_crank: CrankDef,
    pub bomb_wall: BombWallDef,
    pub ruins_far_switch: FarSwitchDef,
}

static CHIME_GATE_1_TILES: [(u32, u32); 2] = [(49, 110), (50, 110)];
static CHIME_GATE_2_TILES: [(u32, u32); 2] = [(74, 111), (75, 111)];

/// West approach only — fence ring is painted solid in ruins map.
static PLATE_GATE: [(u32, u32); 3] = [(194, 149), (194, 150), (194, 151)];

static CAMP_BARRICADE_A: [(u32, u32); 3] = [(198, 58), (199, 58), (200, 58)];
static CAMP_BARRICADE_B: [(u32, u32); 3] = [(208, 62), (208, 63), (208, 64)];

static BRIDGE_SWAPS: [(u32, u32, u16); 4] = [
    (66, 93, T_BRIDGE_LOWERED),
    (67, 93, T_BRIDGE_LOWERED),
    (66, 94, T_BRIDGE_LOWERED),
    (67, 94, T_BRIDGE_LOWERED),
];

static FAR_SWITCH_GATE: [(u32, u32); 1] = [(210, 142)];

pub static OVERWORLD: OverworldPuzzles = OverworldPuzzles {
    chime_gates: &[
        ChimeGateDef {
            chime: (48, 109),
            gate: &CHIME_GATE_1_TILES,
            open_tile: T_PATH,
            flag: flags::PUZZLE_CHIME_GATE_1,
        },
        ChimeGateDef {
            chime: (77, 110),
            gate: &CHIME_GATE_2_TILES,
            open_tile: T_PATH,
            flag: flags::PUZZLE_CHIME_GATE_2,
        },
    ],
    chime_finale: ChimeFinaleDef {
        chimes: [(70, 112), (78, 112), (74, 118)],
        window_ticks: 270, // Phase 5: 4.0s → 4.5s finale window
        flag: flags::PUZZLE_CHIMES_DONE,
    },
    plate_court: PlateCourtDef {
        plates: [(190, 145), (206, 155)],
        blocks: [(192, 151), (202, 147)],
        gate: &PLATE_GATE,
        open_tile: T_SAND,
        floor_tile: T_SAND,
        flag: flags::PUZZLE_PLATES_DONE,
    },
    barricades: &[
        BarricadeDef {
            tiles: &CAMP_BARRICADE_A,
            floor_tile: T_DIRT_ASH_ALIAS,
        },
        BarricadeDef {
            tiles: &CAMP_BARRICADE_B,
            floor_tile: T_DIRT_ASH_ALIAS,
        },
    ],
    bridge_crank: CrankDef {
        crank: (67, 91),
        swaps: &BRIDGE_SWAPS,
        flag: flags::BRIDGE_LOWERED,
    },
    bomb_wall: BombWallDef {
        wall: (30, 185),
        open_tile: T_CAVE_MOUTH,
        door: (MapId::Cave(2), 0),
        flag: flags::WALL_GROVE_OPEN,
    },
    ruins_far_switch: FarSwitchDef {
        crank: (212, 140),
        gate: &FAR_SWITCH_GATE,
        open_tile: T_SAND,
        flag: flags::PUZZLE_RUINS_FAR,
    },
};

/// Avoid importing dirt-ash in the gate const table above (catalog alias).
const T_DIRT_ASH_ALIAS: u16 = crate::maps::catalog::T_DIRT_ASH;

pub fn for_map(map: MapId) -> Option<&'static OverworldPuzzles> {
    match map {
        MapId::Overworld => Some(&OVERWORLD),
        _ => None,
    }
}

/// Convenience: gate tiles that start closed (painted at load when unsolved).
#[allow(dead_code)]
pub fn closed_gate_tile() -> u16 {
    T_GATE
}
