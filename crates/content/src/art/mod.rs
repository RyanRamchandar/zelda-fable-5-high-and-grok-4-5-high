//! Indexed-color sprite/tile grids → baked by `game::assets` into `engine::atlas`.

pub mod enemies;
pub mod npcs;
pub mod palette;
pub mod player_actions;
pub mod player_base;
pub mod props_interior;
pub mod props_puzzle;
pub mod props_village;
pub mod props_wild;
pub mod tiles;
pub mod tiles_forest;
pub mod tiles_terrain;
pub mod tiles_water;
pub mod ui;

use palette::PaletteSwap;

#[derive(Clone, Copy, Debug)]
pub struct SpriteDef {
    pub name: &'static str,
    pub w: u32,
    pub h: u32,
    pub frames: u32,
    pub grid: &'static str,
}

/// Optional tint applied at bake time (separate atlas strip).
#[derive(Clone, Copy, Debug)]
pub struct SpriteBake {
    pub def: &'static SpriteDef,
    pub swap: Option<&'static PaletteSwap>,
    pub key: &'static str,
}

pub fn all_sprites() -> Vec<&'static SpriteDef> {
    all_bakes().into_iter().map(|b| b.def).collect()
}

pub fn all_bakes() -> Vec<SpriteBake> {
    vec![
        // Player
        bake(&player_base::PLAYER_IDLE),
        bake(&player_base::PLAYER_WALK_D),
        bake(&player_base::PLAYER_WALK_U),
        bake(&player_base::PLAYER_WALK_R),
        bake(&player_base::PLAYER_DASH_D),
        bake(&player_base::PLAYER_DASH_U),
        bake(&player_base::PLAYER_DASH_R),
        bake(&player_base::PLAYER_SHIELD),
        bake(&player_base::PLAYER_HURT),
        bake(&player_base::PLAYER_CHARGE_GLOW),
        bake(&player_actions::PLAYER_SLASH_D),
        bake(&player_actions::PLAYER_SLASH_U),
        bake(&player_actions::PLAYER_SLASH_R),
        bake(&player_actions::PLAYER_BSLASH_D),
        bake(&player_actions::PLAYER_BSLASH_U),
        bake(&player_actions::PLAYER_BSLASH_R),
        bake(&player_actions::PLAYER_LUNGE_D),
        bake(&player_actions::PLAYER_LUNGE_U),
        bake(&player_actions::PLAYER_LUNGE_R),
        bake(&player_actions::PLAYER_SPIN),
        // Enemies + variants
        bake(&enemies::SLIME),
        SpriteBake {
            def: &enemies::SLIME,
            swap: Some(&palette::SLIME_ANGRY),
            key: "slime_angry",
        },
        SpriteBake {
            def: &enemies::SLIME,
            swap: Some(&palette::SLIME_DUMMY),
            key: "slime_dummy",
        },
        bake(&enemies::BAT),
        bake(&enemies::OCTOROK),
        bake(&enemies::OCTOROK_ROCK),
        SpriteBake {
            def: &enemies::OCTOROK_ROCK,
            swap: Some(&palette::ROCK_WARM),
            key: "octorok_rock_warm",
        },
        // Arena / interior legacy tiles
        bake(&tiles::TILE_FLOOR_A),
        bake(&tiles::TILE_FLOOR_B),
        bake(&tiles::TILE_WALL_TOP),
        bake(&tiles::TILE_WALL_FACE),
        bake(&tiles::TILE_FOUNTAIN),
        bake(&tiles::TILE_PILLAR),
        // Overworld terrain
        bake(&tiles_terrain::TILE_GRASS_A),
        bake(&tiles_terrain::TILE_GRASS_B),
        bake(&tiles_terrain::TILE_GRASS_FLOWER),
        bake(&tiles_terrain::TILE_GRASS_PEBBLE),
        bake(&tiles_terrain::TILE_PATH),
        bake(&tiles_terrain::TILE_PATH_N),
        bake(&tiles_terrain::TILE_PATH_S),
        bake(&tiles_terrain::TILE_PATH_E),
        bake(&tiles_terrain::TILE_PATH_W),
        bake(&tiles_terrain::TILE_DIRT),
        bake(&tiles_terrain::TILE_DIRT_ASH),
        bake(&tiles_terrain::TILE_SAND),
        bake(&tiles_terrain::TILE_CLIFF_TOP),
        bake(&tiles_terrain::TILE_CLIFF_FACE),
        bake(&tiles_terrain::TILE_CLIFF_EDGE_N),
        bake(&tiles_terrain::TILE_CLIFF_EDGE_S),
        bake(&tiles_terrain::TILE_CLIFF_EDGE_E),
        bake(&tiles_terrain::TILE_CLIFF_EDGE_W),
        bake(&tiles_terrain::TILE_CLIFF_STAIRS),
        bake(&tiles_terrain::TILE_LEDGE_S),
        bake(&tiles_terrain::TILE_LEDGE_N),
        bake(&tiles_terrain::TILE_LEDGE_E),
        bake(&tiles_terrain::TILE_LEDGE_W),
        // Water
        bake(&tiles_water::TILE_WATER_DEEP),
        bake(&tiles_water::TILE_WATER_SHALLOW),
        bake(&tiles_water::TILE_WATER_SHIMMER),
        bake(&tiles_water::TILE_SHORE_N),
        bake(&tiles_water::TILE_SHORE_S),
        bake(&tiles_water::TILE_SHORE_E),
        bake(&tiles_water::TILE_SHORE_W),
        bake(&tiles_water::TILE_SHORE_NE),
        bake(&tiles_water::TILE_SHORE_NW),
        bake(&tiles_water::TILE_SHORE_SE),
        bake(&tiles_water::TILE_SHORE_SW),
        // Forest + structures
        bake(&tiles_forest::TILE_TREE_TRUNK),
        bake(&tiles_forest::TILE_CANOPY_NW),
        bake(&tiles_forest::TILE_CANOPY_NE),
        bake(&tiles_forest::TILE_CANOPY_SW),
        bake(&tiles_forest::TILE_CANOPY_SE),
        bake(&tiles_forest::TILE_BRIDGE_H),
        bake(&tiles_forest::TILE_BRIDGE_V),
        bake(&tiles_forest::TILE_BRIDGE_BROKEN),
        bake(&tiles_forest::TILE_FENCE),
        bake(&tiles_forest::TILE_SHRINE_STONE),
        bake(&tiles_forest::TILE_DOOR_SEALED),
        bake(&tiles_forest::TILE_CAVE_MOUTH),
        bake(&tiles_forest::TILE_HOUSE_WALL),
        bake(&tiles_forest::TILE_HOUSE_DOOR),
        bake(&tiles_forest::TILE_COLUMN),
        bake(&tiles_forest::TILE_ARCH_TOP),
        bake(&tiles_forest::TILE_INT_FLOOR),
        bake(&tiles_forest::TILE_INT_WALL),
        // Phase 2B props + NPCs + POIs
        bake(&props_village::PROP_LANTERN),
        bake(&props_village::PROP_FLOWER_BED),
        bake(&props_village::PROP_STALL),
        bake(&props_village::PROP_BASIN),
        bake(&props_village::PROP_SIGN),
        bake(&props_village::PROP_CHEST),
        bake(&props_village::PROP_GEM),
        bake(&props_village::PROP_PEDESTAL),
        bake(&props_village::POI_SHOP),
        bake(&props_village::POI_FOUNTAIN),
        bake(&props_village::POI_SHRINE),
        bake(&props_village::POI_GEM),
        bake(&props_village::POI_CHECK),
        bake(&props_village::POI_STAR),
        bake(&props_village::POI_SECRET),
        bake(&props_wild::PROP_TENT),
        bake(&props_wild::PROP_BONFIRE),
        bake(&props_wild::PROP_CRACKED),
        bake(&props_wild::PROP_CHIME),
        bake(&props_wild::PROP_BIRDS),
        bake(&props_wild::PROP_RUBBLE),
        bake(&props_wild::PROP_BRAZIER),
        bake(&props_wild::PROP_PALE_TREE),
        bake(&props_puzzle::PROP_GATE),
        bake(&props_puzzle::PROP_BLOCK),
        bake(&props_puzzle::PROP_PLATE_UP),
        bake(&props_puzzle::PROP_PLATE_DOWN),
        bake(&props_puzzle::PROP_BARRICADE),
        bake(&props_puzzle::PROP_CRANK),
        bake(&props_puzzle::PROP_BRIDGE_LOWERED),
        bake(&props_puzzle::PROP_BOMB),
        bake(&props_interior::PROP_RUG),
        bake(&props_interior::PROP_TABLE),
        bake(&props_interior::PROP_BED),
        bake(&props_interior::PROP_SHELF),
        bake(&props_interior::PROP_COUNTER),
        bake(&props_interior::PROP_POT),
        bake(&npcs::NPC_ELDER),
        bake(&npcs::NPC_SHOP),
        bake(&npcs::NPC_VILLAGER_A),
        bake(&npcs::NPC_VILLAGER_B),
        bake(&npcs::NPC_VILLAGER_C),
        bake(&npcs::NPC_KID),
        // UI
        bake(&ui::UI_HEART_FULL),
        bake(&ui::UI_HEART_HALF),
        bake(&ui::UI_HEART_EMPTY),
        bake(&ui::UI_ENERGY_FRAME),
        bake(&ui::UI_ENERGY_FILL),
        bake(&ui::UI_STYLE_CHIP),
        bake(&ui::UI_ITEM_SLOT),
        bake(&ui::UI_TOAST_PANEL),
    ]
}

fn bake(def: &'static SpriteDef) -> SpriteBake {
    SpriteBake {
        def,
        swap: None,
        key: def.name,
    }
}
