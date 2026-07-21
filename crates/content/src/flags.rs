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

/// Phase 3A dungeon / boomerang (100–139). 140–149 reserved for 3B.
pub const ITEM_BOOMERANG: u16 = 100;
pub const DKEY_SMALL_1: u16 = 101;
pub const DKEY_SMALL_2: u16 = 102;
pub const DKEY_BOSS: u16 = 103;
pub const DDOOR_WING: u16 = 104; // unused — west wing open by design
pub const DDOOR_INNER: u16 = 105;
pub const DDOOR_BOSS_USED: u16 = 106;
pub const SEAL_WEST: u16 = 107;
pub const SEAL_EAST: u16 = 108;
pub const DCHEST_BOOMERANG: u16 = 110;
pub const DCHEST_KEY1: u16 = 111;
pub const DCHEST_KEY2: u16 = 112;
pub const DCHEST_BOSSKEY: u16 = 113;
pub const DCHEST_RUPEE_A: u16 = 114;
pub const DCHEST_RUPEE_B: u16 = 115;
pub const DCHEST_RUPEE_C: u16 = 116;
/// Room discovery flags: DROOM_0 + room_id
pub const DROOM_0: u16 = 120;
pub const DROOM_MAX: u16 = 139;

pub const GRP_DNG_TRIALS_1: u16 = 90;
pub const GRP_DNG_TRIALS_2: u16 = 91;
pub const GRP_DNG_TRIALS_3: u16 = 92;
pub const GRP_DNG_CURRENTS: u16 = 93;
/// Sanctum miniboss — unlocked on Sanctum Core entry (Phase 3B).
pub const GRP_DNG_SANCTUM: u16 = 94;

/// Phase 3B boss / victory (140–149).
pub const WARDEN_INTRO_SEEN: u16 = 140;
pub const WARDEN_DEFEATED: u16 = 141;
pub const WARDEN_HEART: u16 = 142;
pub const SHARD_OF_COURAGE: u16 = 143;
pub const TUNIC_BOUGHT: u16 = 144;
pub const SANCTUM_CLEARED: u16 = 145;

pub fn droom_flag(room_id: u8) -> u16 {
    DROOM_0 + room_id as u16
}
