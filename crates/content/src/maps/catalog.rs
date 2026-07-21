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
