//! Save-flag id registry shared by map placement (content) and runtime (game).

pub const QUEST_STARTED: u16 = 1;

pub const CHEST_VILLAGE_TEST: u16 = 10;
pub const CHEST_SHOP_HEDGE: u16 = 11;
pub const CHEST_CAMP_TOWER: u16 = 12;
pub const CHEST_SHRINE_BRAZIERS: u16 = 13;
pub const CHEST_POWER_GEM: u16 = 14;
pub const CHEST_CLIFFS_HEART: u16 = 15;
pub const CHEST_RUINS_CELLAR: u16 = 16;
pub const CHEST_RIVER_ISLAND: u16 = 17;
pub const CHEST_GROVE_BOMB: u16 = 18;
pub const CHEST_GROVE_HEART: u16 = 19;

pub const SECRET_GROVE_BOMB: u16 = 30;
pub const SECRET_CLIFFS_CAVE: u16 = 31;
pub const SECRET_RUINS_CELLAR: u16 = 32;
pub const SECRET_RIVER_ISLAND: u16 = 33;
pub const SECRET_SHOP_HEDGE: u16 = 34;
pub const SECRET_GROVE_TREE: u16 = 35;
pub const SECRET_CAMP_TOWER: u16 = 36;
pub const SECRET_SUMMIT_VISTA: u16 = 37;
pub const SECRET_MEADOW_FLOWERS: u16 = 38;
pub const SECRET_SHRINE_BRAZIERS: u16 = 39;

pub const DOOR_SHRINE_OPEN: u16 = 50;

pub const HEART_PIECE_1: u16 = 60;
pub const HEART_PIECE_2: u16 = 61;
pub const HEART_PIECE_3: u16 = 62;
pub const HEART_PIECE_4: u16 = 63;
pub const HEART_REWARD_APPLIED: u16 = 64;

pub const GEM_COURAGE: u16 = 70;
pub const GEM_POWER: u16 = 71;
pub const GEM_WISDOM: u16 = 72;

pub const GROUP_CAMP_GUARD: u16 = 80;

pub const CHEST_RUINS_BONUS: u16 = 20;

/// Phase 2C puzzle / economy flags.
pub const PUZZLE_CHIME_GATE_1: u16 = 90;
pub const PUZZLE_CHIME_GATE_2: u16 = 91;
pub const PUZZLE_CHIMES_DONE: u16 = 92;
pub const PUZZLE_PLATES_DONE: u16 = 93;
pub const WALL_GROVE_OPEN: u16 = 94;
pub const BRIDGE_LOWERED: u16 = 95;
pub const SHOP_BOMB_BAG: u16 = 96;
pub const SHOP_BOMBS_UNLOCKED: u16 = 97;
pub const TUNIC_UNLOCKED: u16 = 98;
/// Ruins far-switch gate (Phase 3 boomerang preview).
pub const PUZZLE_RUINS_FAR: u16 = 99;

/// Encounter group ids (spawner).
pub const GRP_MEADOW: u16 = 10;
pub const GRP_GROVE: u16 = 30;
pub const GRP_CAMP: u16 = 40;
pub const GRP_CAMP_GUARD: u16 = 41;
/// Camp war-chest wave 2 / 3 (chained from 41).
pub const GRP_CAMP_W2: u16 = 42;
pub const GRP_CAMP_W3: u16 = 43;
pub const GRP_RUINS: u16 = 50;
pub const GRP_CLIFFS: u16 = 60;
pub const GRP_SHRINE: u16 = 70;

/// Spawner wave unlock chain: clear `from` → unlock + force-spawn `to`.
pub const CAMP_WAVE_CHAIN: [(u16, u16); 2] = [(41, 42), (42, 43)];
