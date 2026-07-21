//! Dungeon puzzle site data (crystals, flame, ordered seals).

use crate::maps::dungeon::{
    self, ROOM_CURRENTS_LINES, ROOM_CURRENTS_RANGE, ROOM_CURRENTS_TEACH, ROOM_FLAME, ROOM_MULTI,
    ROOM_SEAL_E, ROOM_SEAL_W,
};

#[derive(Clone, Copy, Debug)]
pub struct CrystalDef {
    pub tx: u32,
    pub ty: u32,
    pub amber: bool,
    pub room: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct GatePair {
    pub room: u8,
    pub tiles: &'static [(u32, u32)],
    pub amber: bool,
}

#[derive(Clone, Copy, Debug)]
pub struct FlameRoomDef {
    pub room: u8,
    pub brazier: (u32, u32),
    pub torches: [(u32, u32); 2],
    pub gate: &'static [(u32, u32)],
}

#[derive(Clone, Copy, Debug)]
pub struct SealRoomDef {
    pub room: u8,
    pub flag: u16,
    pub crystals: [(u32, u32); 3],
    pub plinth: (u32, u32),
}

#[derive(Clone, Copy, Debug)]
pub struct DungeonPuzzles {
    pub crystals: &'static [CrystalDef],
    pub gates: &'static [GatePair],
    pub flame: FlameRoomDef,
    pub seal_w: SealRoomDef,
    pub seal_e: SealRoomDef,
    pub multi_gate: &'static [(u32, u32)],
}

pub fn def() -> DungeonPuzzles {
    DungeonPuzzles {
        crystals: &[
            CrystalDef {
                tx: 72,
                ty: 54,
                amber: false,
                room: ROOM_CURRENTS_TEACH,
            },
            CrystalDef {
                tx: 85,
                ty: 52,
                amber: true,
                room: ROOM_CURRENTS_RANGE,
            },
            CrystalDef {
                tx: 64,
                ty: 38,
                amber: false,
                room: ROOM_CURRENTS_LINES,
            },
            CrystalDef {
                tx: 74,
                ty: 40,
                amber: true,
                room: ROOM_CURRENTS_LINES,
            },
            CrystalDef {
                tx: 84,
                ty: 38,
                amber: false,
                room: ROOM_MULTI,
            },
            CrystalDef {
                tx: 94,
                ty: 40,
                amber: false,
                room: ROOM_MULTI,
            },
            CrystalDef {
                tx: 24,
                ty: 22,
                amber: false,
                room: ROOM_SEAL_W,
            },
            CrystalDef {
                tx: 28,
                ty: 22,
                amber: false,
                room: ROOM_SEAL_W,
            },
            CrystalDef {
                tx: 32,
                ty: 24,
                amber: false,
                room: ROOM_SEAL_W,
            },
            CrystalDef {
                tx: 64,
                ty: 20,
                amber: true,
                room: ROOM_SEAL_E,
            },
            CrystalDef {
                tx: 69,
                ty: 20,
                amber: true,
                room: ROOM_SEAL_E,
            },
            CrystalDef {
                tx: 74,
                ty: 24,
                amber: true,
                room: ROOM_SEAL_E,
            },
        ],
        gates: &[
            GatePair {
                room: ROOM_CURRENTS_TEACH,
                tiles: &[(66, 55), (66, 56)],
                amber: false,
            },
            GatePair {
                room: ROOM_CURRENTS_RANGE,
                tiles: &[(92, 58), (93, 58)],
                amber: true,
            },
            GatePair {
                room: ROOM_CURRENTS_LINES,
                tiles: &[(69, 36)],
                amber: false,
            },
            GatePair {
                room: ROOM_CURRENTS_LINES,
                tiles: &[(69, 42)],
                amber: true,
            },
        ],
        flame: FlameRoomDef {
            room: ROOM_FLAME,
            brazier: (89, 70),
            torches: [(84, 68), (94, 72)],
            gate: &[(89, 66), (90, 66)],
        },
        seal_w: SealRoomDef {
            room: ROOM_SEAL_W,
            flag: crate::flags::SEAL_WEST,
            crystals: [(24, 22), (28, 22), (32, 24)],
            plinth: (26, 28),
        },
        seal_e: SealRoomDef {
            room: ROOM_SEAL_E,
            flag: crate::flags::SEAL_EAST,
            crystals: [(64, 20), (69, 20), (74, 24)],
            plinth: (69, 28),
        },
        multi_gate: &[(89, 36), (90, 36)],
    }
}

pub fn for_room(room: u8) -> bool {
    matches!(
        room,
        dungeon::ROOM_CURRENTS_TEACH
            | dungeon::ROOM_CURRENTS_RANGE
            | dungeon::ROOM_CURRENTS_LINES
            | dungeon::ROOM_MULTI
            | dungeon::ROOM_FLAME
            | dungeon::ROOM_SEAL_W
            | dungeon::ROOM_SEAL_E
    )
}
