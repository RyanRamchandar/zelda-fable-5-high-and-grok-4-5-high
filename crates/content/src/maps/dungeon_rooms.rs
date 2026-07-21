//! Room topology for the Triforce Shrine dungeon (data-driven exits + minimap).

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DoorKind {
    Open,
    SmallKey,
    BossKey,
    SealWest,
    SealEast,
    Shutter,
    /// Key #2 inner door (currents → east seal).
    InnerKey,
}

#[derive(Clone, Copy, Debug)]
pub struct ExitDef {
    pub to_room: u8,
    pub door: DoorKind,
    pub tx: u32,
    pub ty: u32,
}

#[derive(Clone, Copy, Debug)]
pub struct RoomDef {
    pub id: u8,
    /// Inclusive tile rect (x0, y0, x1, y1).
    pub rect: (u32, u32, u32, u32),
    /// N, E, S, W
    pub exits: [Option<ExitDef>; 4],
    pub name: &'static str,
}

pub const ROOM_VESTIBULE: u8 = 0;
pub const ROOM_TRIALS_1: u8 = 1;
pub const ROOM_TRIALS_2: u8 = 2;
pub const ROOM_TRIALS_3: u8 = 3;
pub const ROOM_BOOMERANG: u8 = 4;
pub const ROOM_CURRENTS_HUB: u8 = 5;
pub const ROOM_CURRENTS_TEACH: u8 = 6;
pub const ROOM_CURRENTS_RANGE: u8 = 7;
pub const ROOM_CURRENTS_LINES: u8 = 8;
pub const ROOM_MULTI: u8 = 9;
pub const ROOM_FLAME: u8 = 10;
pub const ROOM_SEAL_W: u8 = 11;
pub const ROOM_SEAL_E: u8 = 12;
pub const ROOM_ANTECHAMBER: u8 = 13;
pub const ROOM_SANCTUM: u8 = 14;
pub const ROOM_ARENA: u8 = 15;

const fn ex(to: u8, door: DoorKind, tx: u32, ty: u32) -> Option<ExitDef> {
    Some(ExitDef {
        to_room: to,
        door,
        tx,
        ty,
    })
}

/// Non-overlapping rooms on a 100×80 map (20×16 cells + tall antechamber).
pub fn rooms() -> &'static [RoomDef] {
    &ROOMS
}

static ROOMS: [RoomDef; 16] = [
        RoomDef {
            id: ROOM_VESTIBULE,
            rect: (40, 64, 59, 79),
            exits: [
                ex(ROOM_ANTECHAMBER, DoorKind::Shutter, 49, 64),
                ex(ROOM_CURRENTS_HUB, DoorKind::SmallKey, 59, 71),
                None,
                ex(ROOM_TRIALS_1, DoorKind::Open, 40, 71),
            ],
            name: "Vestibule",
        },
        RoomDef {
            id: ROOM_TRIALS_1,
            rect: (20, 64, 39, 79),
            exits: [
                ex(ROOM_TRIALS_2, DoorKind::Shutter, 29, 64),
                ex(ROOM_VESTIBULE, DoorKind::Open, 39, 71),
                None,
                ex(ROOM_BOOMERANG, DoorKind::Open, 20, 71),
            ],
            name: "Hall of Trials I",
        },
        RoomDef {
            id: ROOM_TRIALS_2,
            rect: (20, 48, 39, 63),
            exits: [
                None,
                None,
                ex(ROOM_TRIALS_1, DoorKind::Shutter, 29, 63),
                ex(ROOM_TRIALS_3, DoorKind::Shutter, 20, 55),
            ],
            name: "Hall of Trials II",
        },
        RoomDef {
            id: ROOM_TRIALS_3,
            rect: (0, 48, 19, 63),
            exits: [
                None,
                ex(ROOM_TRIALS_2, DoorKind::Shutter, 19, 55),
                None,
                None,
            ],
            name: "Hall of Trials III",
        },
        RoomDef {
            id: ROOM_BOOMERANG,
            rect: (0, 64, 19, 79),
            exits: [
                None,
                ex(ROOM_TRIALS_1, DoorKind::Open, 19, 71),
                None,
                None,
            ],
            name: "Gale Chamber",
        },
        RoomDef {
            id: ROOM_CURRENTS_HUB,
            rect: (60, 64, 79, 79),
            exits: [
                ex(ROOM_CURRENTS_TEACH, DoorKind::Open, 69, 64),
                ex(ROOM_FLAME, DoorKind::Open, 79, 71),
                None,
                ex(ROOM_VESTIBULE, DoorKind::SmallKey, 60, 71),
            ],
            name: "Hall of Currents",
        },
        RoomDef {
            id: ROOM_CURRENTS_TEACH,
            rect: (60, 48, 79, 63),
            exits: [
                ex(ROOM_CURRENTS_LINES, DoorKind::Open, 69, 48),
                ex(ROOM_CURRENTS_RANGE, DoorKind::Open, 79, 55),
                ex(ROOM_CURRENTS_HUB, DoorKind::Open, 69, 63),
                None,
            ],
            name: "Current Lesson",
        },
        RoomDef {
            id: ROOM_CURRENTS_RANGE,
            rect: (80, 48, 99, 63),
            exits: [
                ex(ROOM_MULTI, DoorKind::Open, 89, 48),
                None,
                None,
                ex(ROOM_CURRENTS_TEACH, DoorKind::Open, 80, 55),
            ],
            name: "Wind Channel",
        },
        RoomDef {
            id: ROOM_CURRENTS_LINES,
            rect: (60, 32, 79, 47),
            exits: [
                ex(ROOM_SEAL_E, DoorKind::InnerKey, 69, 32),
                ex(ROOM_MULTI, DoorKind::Open, 79, 39),
                ex(ROOM_CURRENTS_TEACH, DoorKind::Open, 69, 47),
                None,
            ],
            name: "Throw Lines",
        },
        RoomDef {
            id: ROOM_MULTI,
            rect: (80, 32, 99, 47),
            exits: [
                None,
                None,
                ex(ROOM_CURRENTS_RANGE, DoorKind::Open, 89, 47),
                ex(ROOM_CURRENTS_LINES, DoorKind::Open, 80, 39),
            ],
            name: "Twin Crystals",
        },
        RoomDef {
            id: ROOM_FLAME,
            rect: (80, 64, 99, 79),
            exits: [
                None,
                None,
                None,
                ex(ROOM_CURRENTS_HUB, DoorKind::Open, 80, 71),
            ],
            name: "Eternal Flame",
        },
        RoomDef {
            id: ROOM_SEAL_W,
            rect: (20, 16, 39, 31),
            exits: [
                None,
                ex(ROOM_ANTECHAMBER, DoorKind::SealWest, 39, 23),
                None,
                None,
            ],
            name: "West Seal",
        },
        RoomDef {
            id: ROOM_SEAL_E,
            rect: (60, 16, 79, 31),
            exits: [
                None,
                None,
                ex(ROOM_CURRENTS_LINES, DoorKind::InnerKey, 69, 31),
                ex(ROOM_ANTECHAMBER, DoorKind::SealEast, 60, 23),
            ],
            name: "East Seal",
        },
        // Tall antechamber: north of vestibule through seal latitude.
        RoomDef {
            id: ROOM_ANTECHAMBER,
            rect: (40, 16, 59, 63),
            exits: [
                ex(ROOM_SANCTUM, DoorKind::Shutter, 49, 16),
                ex(ROOM_SEAL_E, DoorKind::SealEast, 59, 23),
                ex(ROOM_VESTIBULE, DoorKind::Shutter, 49, 63),
                ex(ROOM_SEAL_W, DoorKind::SealWest, 40, 23),
            ],
            name: "Sanctum Antechamber",
        },
        RoomDef {
            id: ROOM_SANCTUM,
            rect: (40, 0, 59, 15),
            exits: [
                None,
                ex(ROOM_ARENA, DoorKind::BossKey, 59, 7),
                ex(ROOM_ANTECHAMBER, DoorKind::Shutter, 49, 15),
                None,
            ],
            name: "Sanctum Core",
        },
        RoomDef {
            id: ROOM_ARENA,
            rect: (60, 0, 79, 15),
            exits: [
                None,
                None,
                None,
                ex(ROOM_SANCTUM, DoorKind::BossKey, 60, 7),
            ],
            name: "Guardian Arena",
        },
];

pub fn room_by_id(id: u8) -> Option<&'static RoomDef> {
    rooms().iter().find(|r| r.id == id)
}

pub fn room_at_tile(tx: u32, ty: u32) -> Option<&'static RoomDef> {
    rooms().iter().find(|r| {
        let (x0, y0, x1, y1) = r.rect;
        tx >= x0 && tx <= x1 && ty >= y0 && ty <= y1
    })
}
