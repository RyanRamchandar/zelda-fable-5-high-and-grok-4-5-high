//! Tile id catalog — data-driven contract for rendering + default collision.
//!
//! Id ranges (gaps reserved for Phase 2B):
//! - 0: void / empty (detail & overhang)
//! - 1–39: terrain (grass, path, dirt, sand, flowers)
//! - 40–79: water (deep, shallow, shores, shimmer)
//! - 80–119: cliffs (face, top, edges, stairs, ledges)
//! - 120–159: forest (trunk, canopy)
//! - 160–199: structures (bridge, fence, shrine)
//! - 200–239: village / interior / arena shells
//! - 240+: reserved 2B

use super::flags;

#[derive(Clone, Copy, Debug)]
pub struct TileInfo {
    pub sprite: &'static str,
    pub frames: u16,
    pub anim_rate: u16,
    pub flags: u8,
}

pub const T_VOID: u16 = 0;

// —— 1–39 terrain ——
pub const T_GRASS_A: u16 = 1;
pub const T_GRASS_B: u16 = 2;
pub const T_GRASS_FLOWER: u16 = 3;
pub const T_GRASS_PEBBLE: u16 = 4;
pub const T_PATH: u16 = 5;
pub const T_PATH_N: u16 = 6;
pub const T_PATH_S: u16 = 7;
pub const T_PATH_E: u16 = 8;
pub const T_PATH_W: u16 = 9;
pub const T_DIRT: u16 = 10;
pub const T_DIRT_ASH: u16 = 11;
pub const T_SAND: u16 = 12;

// —— 40–79 water ——
pub const T_WATER_DEEP: u16 = 40;
pub const T_WATER_SHALLOW: u16 = 41;
pub const T_WATER_SHIMMER: u16 = 42;
pub const T_SHORE_N: u16 = 43;
pub const T_SHORE_S: u16 = 44;
pub const T_SHORE_E: u16 = 45;
pub const T_SHORE_W: u16 = 46;
pub const T_SHORE_NE: u16 = 47;
pub const T_SHORE_NW: u16 = 48;
pub const T_SHORE_SE: u16 = 49;
pub const T_SHORE_SW: u16 = 50;

// —— 80–119 cliffs ——
pub const T_CLIFF_TOP: u16 = 80;
pub const T_CLIFF_FACE: u16 = 81;
pub const T_CLIFF_EDGE_N: u16 = 82;
pub const T_CLIFF_EDGE_S: u16 = 83;
pub const T_CLIFF_EDGE_E: u16 = 84;
pub const T_CLIFF_EDGE_W: u16 = 85;
pub const T_CLIFF_STAIRS: u16 = 86;
pub const T_LEDGE_S: u16 = 87;
pub const T_LEDGE_N: u16 = 88;
pub const T_LEDGE_E: u16 = 89;
pub const T_LEDGE_W: u16 = 90;

// —— 120–159 forest ——
pub const T_TREE_TRUNK: u16 = 120;
pub const T_CANOPY_NW: u16 = 121;
pub const T_CANOPY_NE: u16 = 122;
pub const T_CANOPY_SW: u16 = 123;
pub const T_CANOPY_SE: u16 = 124;

// —— 160–199 structures ——
pub const T_BRIDGE_H: u16 = 160;
pub const T_BRIDGE_V: u16 = 161;
pub const T_BRIDGE_BROKEN: u16 = 162;
pub const T_FENCE: u16 = 163;
pub const T_SHRINE_STONE: u16 = 164;
pub const T_DOOR_SEALED: u16 = 165;
pub const T_CAVE_MOUTH: u16 = 166;
pub const T_HOUSE_WALL: u16 = 167;
pub const T_HOUSE_DOOR: u16 = 168;
pub const T_COLUMN: u16 = 169;
pub const T_ARCH_TOP: u16 = 170;

// —— 200–239 village / arena / interior (Phase 1 ids relocated; 0 = void) ——
pub const T_FLOOR_A: u16 = 200;
pub const T_FLOOR_B: u16 = 201;
pub const T_WALL: u16 = 202;
pub const T_WALL_TOP: u16 = 203;
pub const T_FOUNTAIN: u16 = 204;
pub const T_PILLAR: u16 = 205;
pub const T_INT_FLOOR: u16 = 206;
pub const T_INT_WALL: u16 = 207;

// —— 240+ Phase 2B props (detail/overhang; mostly non-solid) ——
pub const T_LANTERN: u16 = 240;
pub const T_FLOWER_BED: u16 = 241;
pub const T_STALL: u16 = 242;
pub const T_BASIN: u16 = 243;
pub const T_TENT: u16 = 244;
pub const T_BONFIRE: u16 = 245;
pub const T_CRACKED_WALL: u16 = 246;
pub const T_CHIME: u16 = 247;
pub const T_BIRDS: u16 = 248;
pub const T_RUBBLE: u16 = 249;
pub const T_BRAZIER: u16 = 250;
pub const T_PALE_TREE: u16 = 251;
pub const T_PEDESTAL: u16 = 252;
pub const T_RUG: u16 = 253;
pub const T_TABLE: u16 = 254;
pub const T_BED: u16 = 255;
pub const T_SHELF: u16 = 256;
pub const T_COUNTER: u16 = 257;
pub const T_POT: u16 = 258;
pub const T_DOOR_OPEN: u16 = 259;

// —— 260+ Phase 2C puzzle interactives ——
pub const T_GATE: u16 = 260;
pub const T_BLOCK: u16 = 261;
pub const T_PLATE_UP: u16 = 262;
pub const T_PLATE_DOWN: u16 = 263;
pub const T_BARRICADE: u16 = 264;
pub const T_CRANK: u16 = 265;
pub const T_BRIDGE_LOWERED: u16 = 266;

// —— 280–309 Phase 3A dungeon ——
pub const D_FLOOR_A: u16 = 280;
pub const D_FLOOR_B: u16 = 281;
pub const D_FLOOR_RUNE: u16 = 282;
pub const D_WALL: u16 = 283;
pub const D_WALL_TOP: u16 = 284;
pub const D_PIT: u16 = 285;
pub const D_WATER: u16 = 286;
pub const D_WATER_EDGE: u16 = 287;
pub const D_STAIRS: u16 = 288;
pub const D_DOOR_OPEN: u16 = 289;
pub const D_DOOR_LOCKED: u16 = 290;
pub const D_DOOR_BOSS: u16 = 291;
pub const D_SHUTTER: u16 = 292;
pub const D_LIFT: u16 = 293;
pub const D_CRYSTAL_BLUE: u16 = 294;
pub const D_CRYSTAL_AMBER: u16 = 295;
pub const D_GATE_BLUE_UP: u16 = 296;
pub const D_GATE_BLUE_DOWN: u16 = 297;
pub const D_GATE_AMBER_UP: u16 = 298;
pub const D_GATE_AMBER_DOWN: u16 = 299;
pub const D_TORCH_LIT: u16 = 300;
pub const D_TORCH_UNLIT: u16 = 301;
pub const D_BRAZIER_ETERNAL: u16 = 302;
pub const D_RUNE_1: u16 = 303;
pub const D_RUNE_2: u16 = 304;
pub const D_RUNE_3: u16 = 305;
pub const D_SEAL_DOOR: u16 = 306;
pub const D_SEAL_BROKEN: u16 = 307;
// 308–309 spare

/// Compatibility aliases (Phase 1 consumers / arena builder).
pub const FLOOR: u16 = T_FLOOR_A;
pub const WALL: u16 = T_WALL;
pub const FOUNTAIN: u16 = T_FOUNTAIN;

const VOID: TileInfo = TileInfo {
    sprite: "",
    frames: 1,
    anim_rate: 0,
    flags: 0,
};

const fn solid(sprite: &'static str) -> TileInfo {
    TileInfo {
        sprite,
        frames: 1,
        anim_rate: 0,
        flags: flags::SOLID,
    }
}

const fn walk(sprite: &'static str) -> TileInfo {
    TileInfo {
        sprite,
        frames: 1,
        anim_rate: 0,
        flags: 0,
    }
}

const fn water(sprite: &'static str) -> TileInfo {
    TileInfo {
        sprite,
        frames: 1,
        anim_rate: 0,
        flags: flags::SOLID | flags::WATER,
    }
}

const fn anim_water(sprite: &'static str, frames: u16, rate: u16) -> TileInfo {
    TileInfo {
        sprite,
        frames,
        anim_rate: rate,
        flags: flags::SOLID | flags::WATER,
    }
}

/// Lookup tile metadata. Unknown ids return void (never drawn).
pub fn tile_info(id: u16) -> &'static TileInfo {
    match id {
        T_VOID => &VOID,
        T_GRASS_A => &TI_GRASS_A,
        T_GRASS_B => &TI_GRASS_B,
        T_GRASS_FLOWER => &TI_GRASS_FLOWER,
        T_GRASS_PEBBLE => &TI_GRASS_PEBBLE,
        T_PATH => &TI_PATH,
        T_PATH_N => &TI_PATH_N,
        T_PATH_S => &TI_PATH_S,
        T_PATH_E => &TI_PATH_E,
        T_PATH_W => &TI_PATH_W,
        T_DIRT => &TI_DIRT,
        T_DIRT_ASH => &TI_DIRT_ASH,
        T_SAND => &TI_SAND,
        T_WATER_DEEP => &TI_WATER_DEEP,
        T_WATER_SHALLOW => &TI_WATER_SHALLOW,
        T_WATER_SHIMMER => &TI_WATER_SHIMMER,
        T_SHORE_N => &TI_SHORE_N,
        T_SHORE_S => &TI_SHORE_S,
        T_SHORE_E => &TI_SHORE_E,
        T_SHORE_W => &TI_SHORE_W,
        T_SHORE_NE => &TI_SHORE_NE,
        T_SHORE_NW => &TI_SHORE_NW,
        T_SHORE_SE => &TI_SHORE_SE,
        T_SHORE_SW => &TI_SHORE_SW,
        T_CLIFF_TOP => &TI_CLIFF_TOP,
        T_CLIFF_FACE => &TI_CLIFF_FACE,
        T_CLIFF_EDGE_N => &TI_CLIFF_EDGE_N,
        T_CLIFF_EDGE_S => &TI_CLIFF_EDGE_S,
        T_CLIFF_EDGE_E => &TI_CLIFF_EDGE_E,
        T_CLIFF_EDGE_W => &TI_CLIFF_EDGE_W,
        T_CLIFF_STAIRS => &TI_CLIFF_STAIRS,
        T_LEDGE_S => &TI_LEDGE_S,
        T_LEDGE_N => &TI_LEDGE_N,
        T_LEDGE_E => &TI_LEDGE_E,
        T_LEDGE_W => &TI_LEDGE_W,
        T_TREE_TRUNK => &TI_TREE_TRUNK,
        T_CANOPY_NW => &TI_CANOPY_NW,
        T_CANOPY_NE => &TI_CANOPY_NE,
        T_CANOPY_SW => &TI_CANOPY_SW,
        T_CANOPY_SE => &TI_CANOPY_SE,
        T_BRIDGE_H => &TI_BRIDGE_H,
        T_BRIDGE_V => &TI_BRIDGE_V,
        T_BRIDGE_BROKEN => &TI_BRIDGE_BROKEN,
        T_FENCE => &TI_FENCE,
        T_SHRINE_STONE => &TI_SHRINE_STONE,
        T_DOOR_SEALED => &TI_DOOR_SEALED,
        T_CAVE_MOUTH => &TI_CAVE_MOUTH,
        T_HOUSE_WALL => &TI_HOUSE_WALL,
        T_HOUSE_DOOR => &TI_HOUSE_DOOR,
        T_COLUMN => &TI_COLUMN,
        T_ARCH_TOP => &TI_ARCH_TOP,
        T_FLOOR_A => &TI_FLOOR_A,
        T_FLOOR_B => &TI_FLOOR_B,
        T_WALL => &TI_WALL,
        T_WALL_TOP => &TI_WALL_TOP,
        T_FOUNTAIN => &TI_FOUNTAIN,
        T_PILLAR => &TI_PILLAR,
        T_INT_FLOOR => &TI_INT_FLOOR,
        T_INT_WALL => &TI_INT_WALL,
        T_LANTERN => &TI_LANTERN,
        T_FLOWER_BED => &TI_FLOWER_BED,
        T_STALL => &TI_STALL,
        T_BASIN => &TI_BASIN,
        T_TENT => &TI_TENT,
        T_BONFIRE => &TI_BONFIRE,
        T_CRACKED_WALL => &TI_CRACKED,
        T_CHIME => &TI_CHIME,
        T_BIRDS => &TI_BIRDS,
        T_RUBBLE => &TI_RUBBLE,
        T_BRAZIER => &TI_BRAZIER,
        T_PALE_TREE => &TI_PALE_TREE,
        T_PEDESTAL => &TI_PEDESTAL,
        T_RUG => &TI_RUG,
        T_TABLE => &TI_TABLE,
        T_BED => &TI_BED,
        T_SHELF => &TI_SHELF,
        T_COUNTER => &TI_COUNTER,
        T_POT => &TI_POT,
        T_DOOR_OPEN => &TI_DOOR_OPEN,
        T_GATE => &TI_GATE,
        T_BLOCK => &TI_BLOCK,
        T_PLATE_UP => &TI_PLATE_UP,
        T_PLATE_DOWN => &TI_PLATE_DOWN,
        T_BARRICADE => &TI_BARRICADE,
        T_CRANK => &TI_CRANK,
        T_BRIDGE_LOWERED => &TI_BRIDGE_LOWERED,
        D_FLOOR_A => &TI_D_FLOOR_A,
        D_FLOOR_B => &TI_D_FLOOR_B,
        D_FLOOR_RUNE => &TI_D_FLOOR_RUNE,
        D_WALL => &TI_D_WALL,
        D_WALL_TOP => &TI_D_WALL_TOP,
        D_PIT => &TI_D_PIT,
        D_WATER => &TI_D_WATER,
        D_WATER_EDGE => &TI_D_WATER_EDGE,
        D_STAIRS => &TI_D_STAIRS,
        D_DOOR_OPEN => &TI_D_DOOR_OPEN,
        D_DOOR_LOCKED => &TI_D_DOOR_LOCKED,
        D_DOOR_BOSS => &TI_D_DOOR_BOSS,
        D_SHUTTER => &TI_D_SHUTTER,
        D_LIFT => &TI_D_LIFT,
        D_CRYSTAL_BLUE => &TI_D_CRYSTAL_BLUE,
        D_CRYSTAL_AMBER => &TI_D_CRYSTAL_AMBER,
        D_GATE_BLUE_UP => &TI_D_GATE_BLUE_UP,
        D_GATE_BLUE_DOWN => &TI_D_GATE_BLUE_DOWN,
        D_GATE_AMBER_UP => &TI_D_GATE_AMBER_UP,
        D_GATE_AMBER_DOWN => &TI_D_GATE_AMBER_DOWN,
        D_TORCH_LIT => &TI_D_TORCH_LIT,
        D_TORCH_UNLIT => &TI_D_TORCH_UNLIT,
        D_BRAZIER_ETERNAL => &TI_D_BRAZIER_ETERNAL,
        D_RUNE_1 => &TI_D_RUNE_1,
        D_RUNE_2 => &TI_D_RUNE_2,
        D_RUNE_3 => &TI_D_RUNE_3,
        D_SEAL_DOOR => &TI_D_SEAL_DOOR,
        D_SEAL_BROKEN => &TI_D_SEAL_BROKEN,
        _ => &VOID,
    }
}

/// All non-void tile ids that have sprites (for bake assertion).
pub fn all_tile_ids() -> &'static [u16] {
    &[
        T_GRASS_A,
        T_GRASS_B,
        T_GRASS_FLOWER,
        T_GRASS_PEBBLE,
        T_PATH,
        T_PATH_N,
        T_PATH_S,
        T_PATH_E,
        T_PATH_W,
        T_DIRT,
        T_DIRT_ASH,
        T_SAND,
        T_WATER_DEEP,
        T_WATER_SHALLOW,
        T_WATER_SHIMMER,
        T_SHORE_N,
        T_SHORE_S,
        T_SHORE_E,
        T_SHORE_W,
        T_SHORE_NE,
        T_SHORE_NW,
        T_SHORE_SE,
        T_SHORE_SW,
        T_CLIFF_TOP,
        T_CLIFF_FACE,
        T_CLIFF_EDGE_N,
        T_CLIFF_EDGE_S,
        T_CLIFF_EDGE_E,
        T_CLIFF_EDGE_W,
        T_CLIFF_STAIRS,
        T_LEDGE_S,
        T_LEDGE_N,
        T_LEDGE_E,
        T_LEDGE_W,
        T_TREE_TRUNK,
        T_CANOPY_NW,
        T_CANOPY_NE,
        T_CANOPY_SW,
        T_CANOPY_SE,
        T_BRIDGE_H,
        T_BRIDGE_V,
        T_BRIDGE_BROKEN,
        T_FENCE,
        T_SHRINE_STONE,
        T_DOOR_SEALED,
        T_CAVE_MOUTH,
        T_HOUSE_WALL,
        T_HOUSE_DOOR,
        T_COLUMN,
        T_ARCH_TOP,
        T_FLOOR_A,
        T_FLOOR_B,
        T_WALL,
        T_WALL_TOP,
        T_FOUNTAIN,
        T_PILLAR,
        T_INT_FLOOR,
        T_INT_WALL,
        T_LANTERN,
        T_FLOWER_BED,
        T_STALL,
        T_BASIN,
        T_TENT,
        T_BONFIRE,
        T_CRACKED_WALL,
        T_CHIME,
        T_BIRDS,
        T_RUBBLE,
        T_BRAZIER,
        T_PALE_TREE,
        T_PEDESTAL,
        T_RUG,
        T_TABLE,
        T_BED,
        T_SHELF,
        T_COUNTER,
        T_POT,
        T_DOOR_OPEN,
        T_GATE,
        T_BLOCK,
        T_PLATE_UP,
        T_PLATE_DOWN,
        T_BARRICADE,
        T_CRANK,
        T_BRIDGE_LOWERED,
        D_FLOOR_A,
        D_FLOOR_B,
        D_FLOOR_RUNE,
        D_WALL,
        D_WALL_TOP,
        D_PIT,
        D_WATER,
        D_WATER_EDGE,
        D_STAIRS,
        D_DOOR_OPEN,
        D_DOOR_LOCKED,
        D_DOOR_BOSS,
        D_SHUTTER,
        D_LIFT,
        D_CRYSTAL_BLUE,
        D_CRYSTAL_AMBER,
        D_GATE_BLUE_UP,
        D_GATE_BLUE_DOWN,
        D_GATE_AMBER_UP,
        D_GATE_AMBER_DOWN,
        D_TORCH_LIT,
        D_TORCH_UNLIT,
        D_BRAZIER_ETERNAL,
        D_RUNE_1,
        D_RUNE_2,
        D_RUNE_3,
        D_SEAL_DOOR,
        D_SEAL_BROKEN,
    ]
}

static TI_GRASS_A: TileInfo = walk("grass_a");
static TI_GRASS_B: TileInfo = walk("grass_b");
static TI_GRASS_FLOWER: TileInfo = walk("grass_flower");
static TI_GRASS_PEBBLE: TileInfo = walk("grass_pebble");
static TI_PATH: TileInfo = walk("path");
static TI_PATH_N: TileInfo = walk("path_n");
static TI_PATH_S: TileInfo = walk("path_s");
static TI_PATH_E: TileInfo = walk("path_e");
static TI_PATH_W: TileInfo = walk("path_w");
static TI_DIRT: TileInfo = walk("dirt");
static TI_DIRT_ASH: TileInfo = walk("dirt_ash");
static TI_SAND: TileInfo = walk("sand");
static TI_WATER_DEEP: TileInfo = water("water_deep");
static TI_WATER_SHALLOW: TileInfo = water("water_shallow");
static TI_WATER_SHIMMER: TileInfo = anim_water("water_shimmer", 2, 16);
static TI_SHORE_N: TileInfo = walk("shore_n");
static TI_SHORE_S: TileInfo = walk("shore_s");
static TI_SHORE_E: TileInfo = walk("shore_e");
static TI_SHORE_W: TileInfo = walk("shore_w");
static TI_SHORE_NE: TileInfo = walk("shore_ne");
static TI_SHORE_NW: TileInfo = walk("shore_nw");
static TI_SHORE_SE: TileInfo = walk("shore_se");
static TI_SHORE_SW: TileInfo = walk("shore_sw");
static TI_CLIFF_TOP: TileInfo = solid("cliff_top");
static TI_CLIFF_FACE: TileInfo = solid("cliff_face");
static TI_CLIFF_EDGE_N: TileInfo = solid("cliff_edge_n");
static TI_CLIFF_EDGE_S: TileInfo = solid("cliff_edge_s");
static TI_CLIFF_EDGE_E: TileInfo = solid("cliff_edge_e");
static TI_CLIFF_EDGE_W: TileInfo = solid("cliff_edge_w");
static TI_CLIFF_STAIRS: TileInfo = walk("cliff_stairs");
static TI_LEDGE_S: TileInfo = TileInfo {
    sprite: "ledge_s",
    frames: 1,
    anim_rate: 0,
    flags: flags::SOLID | flags::LEDGE_S,
};
static TI_LEDGE_N: TileInfo = TileInfo {
    sprite: "ledge_n",
    frames: 1,
    anim_rate: 0,
    flags: flags::SOLID | flags::LEDGE_N,
};
static TI_LEDGE_E: TileInfo = TileInfo {
    sprite: "ledge_e",
    frames: 1,
    anim_rate: 0,
    flags: flags::SOLID | flags::LEDGE_E,
};
static TI_LEDGE_W: TileInfo = TileInfo {
    sprite: "ledge_w",
    frames: 1,
    anim_rate: 0,
    flags: flags::SOLID | flags::LEDGE_W,
};
static TI_TREE_TRUNK: TileInfo = solid("tree_trunk");
static TI_CANOPY_NW: TileInfo = walk("canopy_nw");
static TI_CANOPY_NE: TileInfo = walk("canopy_ne");
static TI_CANOPY_SW: TileInfo = walk("canopy_sw");
static TI_CANOPY_SE: TileInfo = walk("canopy_se");
static TI_BRIDGE_H: TileInfo = walk("bridge_h");
static TI_BRIDGE_V: TileInfo = walk("bridge_v");
static TI_BRIDGE_BROKEN: TileInfo = solid("bridge_broken");
static TI_FENCE: TileInfo = solid("fence");
static TI_SHRINE_STONE: TileInfo = solid("shrine_stone");
static TI_DOOR_SEALED: TileInfo = solid("door_sealed");
static TI_CAVE_MOUTH: TileInfo = walk("cave_mouth");
static TI_HOUSE_WALL: TileInfo = solid("house_wall");
static TI_HOUSE_DOOR: TileInfo = walk("house_door");
static TI_COLUMN: TileInfo = solid("column");
static TI_ARCH_TOP: TileInfo = walk("arch_top");
static TI_FLOOR_A: TileInfo = walk("floor_a");
static TI_FLOOR_B: TileInfo = walk("floor_b");
static TI_WALL: TileInfo = solid("wall_face");
static TI_WALL_TOP: TileInfo = solid("wall_top");
static TI_FOUNTAIN: TileInfo = TileInfo {
    sprite: "fountain",
    frames: 2,
    anim_rate: 16,
    flags: 0,
};
static TI_PILLAR: TileInfo = solid("pillar");
static TI_INT_FLOOR: TileInfo = walk("int_floor");
static TI_INT_WALL: TileInfo = solid("int_wall");
static TI_LANTERN: TileInfo = TileInfo {
    sprite: "prop_lantern",
    frames: 2,
    anim_rate: 20,
    flags: 0,
};
static TI_FLOWER_BED: TileInfo = walk("prop_flower_bed");
static TI_STALL: TileInfo = solid("prop_stall");
static TI_BASIN: TileInfo = walk("prop_basin");
static TI_TENT: TileInfo = solid("prop_tent");
static TI_BONFIRE: TileInfo = TileInfo {
    sprite: "prop_bonfire",
    frames: 2,
    anim_rate: 12,
    flags: flags::SOLID,
};
static TI_CRACKED: TileInfo = solid("prop_cracked");
static TI_CHIME: TileInfo = walk("prop_chime");
static TI_BIRDS: TileInfo = TileInfo {
    sprite: "prop_birds",
    frames: 2,
    anim_rate: 18,
    flags: 0,
};
static TI_RUBBLE: TileInfo = walk("prop_rubble");
static TI_BRAZIER: TileInfo = TileInfo {
    sprite: "prop_brazier",
    frames: 2,
    anim_rate: 14,
    flags: flags::SOLID,
};
static TI_PALE_TREE: TileInfo = solid("prop_pale_tree");
static TI_PEDESTAL: TileInfo = solid("prop_pedestal");
static TI_RUG: TileInfo = walk("prop_rug");
static TI_TABLE: TileInfo = solid("prop_table");
static TI_BED: TileInfo = solid("prop_bed");
static TI_SHELF: TileInfo = solid("prop_shelf");
static TI_COUNTER: TileInfo = solid("prop_counter");
static TI_POT: TileInfo = solid("prop_pot");
static TI_DOOR_OPEN: TileInfo = walk("cave_mouth");
static TI_GATE: TileInfo = solid("prop_gate");
static TI_BLOCK: TileInfo = solid("prop_block");
static TI_PLATE_UP: TileInfo = walk("prop_plate_up");
static TI_PLATE_DOWN: TileInfo = walk("prop_plate_down");
static TI_BARRICADE: TileInfo = solid("prop_barricade");
static TI_CRANK: TileInfo = solid("prop_crank");
static TI_BRIDGE_LOWERED: TileInfo = walk("prop_bridge_lowered");

static TI_D_FLOOR_A: TileInfo = walk("d_floor_a");
static TI_D_FLOOR_B: TileInfo = walk("d_floor_b");
static TI_D_FLOOR_RUNE: TileInfo = walk("d_floor_rune");
static TI_D_WALL: TileInfo = solid("d_wall");
static TI_D_WALL_TOP: TileInfo = solid("d_wall_top");
static TI_D_PIT: TileInfo = solid("d_pit");
static TI_D_WATER: TileInfo = anim_water("d_water", 2, 16);
static TI_D_WATER_EDGE: TileInfo = walk("d_water_edge");
static TI_D_STAIRS: TileInfo = walk("d_stairs");
static TI_D_DOOR_OPEN: TileInfo = walk("d_door_open");
static TI_D_DOOR_LOCKED: TileInfo = solid("d_door_locked");
static TI_D_DOOR_BOSS: TileInfo = solid("d_door_boss");
static TI_D_SHUTTER: TileInfo = solid("d_shutter");
static TI_D_LIFT: TileInfo = TileInfo { sprite: "d_lift", frames: 2, anim_rate: 12, flags: 0 };
static TI_D_CRYSTAL_BLUE: TileInfo = solid("d_crystal_blue");
static TI_D_CRYSTAL_AMBER: TileInfo = solid("d_crystal_amber");
static TI_D_GATE_BLUE_UP: TileInfo = solid("d_gate_blue_up");
static TI_D_GATE_BLUE_DOWN: TileInfo = walk("d_gate_blue_down");
static TI_D_GATE_AMBER_UP: TileInfo = solid("d_gate_amber_up");
static TI_D_GATE_AMBER_DOWN: TileInfo = walk("d_gate_amber_down");
static TI_D_TORCH_LIT: TileInfo = TileInfo { sprite: "d_torch_lit", frames: 2, anim_rate: 10, flags: flags::SOLID };
static TI_D_TORCH_UNLIT: TileInfo = solid("d_torch_unlit");
static TI_D_BRAZIER_ETERNAL: TileInfo = TileInfo { sprite: "d_brazier_eternal", frames: 2, anim_rate: 10, flags: flags::SOLID };
static TI_D_RUNE_1: TileInfo = walk("d_rune_1");
static TI_D_RUNE_2: TileInfo = walk("d_rune_2");
static TI_D_RUNE_3: TileInfo = walk("d_rune_3");
static TI_D_SEAL_DOOR: TileInfo = solid("d_seal_door");
static TI_D_SEAL_BROKEN: TileInfo = walk("d_seal_broken");
