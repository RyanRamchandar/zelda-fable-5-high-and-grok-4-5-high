//! Indexed-color sprite/tile grids → baked by `game::assets` into `engine::atlas`.

pub mod enemies;
pub mod palette;
pub mod player_actions;
pub mod player_base;
pub mod tiles;
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
        // Tiles
        bake(&tiles::TILE_FLOOR_A),
        bake(&tiles::TILE_FLOOR_B),
        bake(&tiles::TILE_WALL_TOP),
        bake(&tiles::TILE_WALL_FACE),
        bake(&tiles::TILE_FOUNTAIN),
        bake(&tiles::TILE_PILLAR),
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
