//! Player sprite selection + tunic/shield flash polish.

use engine::render::Draw;

use crate::assets::SpriteMap;
use crate::combat::tuning;
use crate::math::Dir4;
use crate::world::entity::{Entity, EntityData, PlayerState};

pub fn render_player(d: &mut Draw, e: &Entity, sprites: &SpriteMap, tunic: bool) {
    let iframes = e.health.map(|h| h.iframes > 0).unwrap_or(false);
    if iframes && (e.health.unwrap().iframes / 2).is_multiple_of(2) {
        return;
    }
    let EntityData::Player(pd) = &e.data else {
        return;
    };

    let (key, frame, flip) = player_sprite(e.facing, pd, e);
    if matches!(pd.state, PlayerState::Charging { tick } if tick >= 20) {
        if let Some(h) = resolve(sprites, "player_charge_glow", tunic) {
            d.sprite(h, 0, e.pos.x, e.pos.y - 8.0, flip);
        }
    }
    // Perfect-block readability: brief glow while window is open.
    if pd.shield_held && pd.shield_ticks > 0 && pd.shield_ticks <= tuning::PERFECT_BLOCK_WINDOW {
        if let Some(h) = resolve(sprites, "player_charge_glow", tunic) {
            d.sprite(h, 0, e.pos.x, e.pos.y - 8.0, flip);
        }
    }
    if let Some(h) = resolve(sprites, key, tunic) {
        d.sprite(h, frame, e.pos.x, e.pos.y - 8.0, flip);
    }
}

fn resolve(sprites: &SpriteMap, key: &str, tunic: bool) -> Option<engine::atlas::SpriteHandle> {
    if tunic {
        match key {
            "player_idle" => sprites.get("player_idle_tunic"),
            "player_walk_d" => sprites.get("player_walk_d_tunic"),
            "player_walk_u" => sprites.get("player_walk_u_tunic"),
            "player_walk_r" => sprites.get("player_walk_r_tunic"),
            "player_dash_d" => sprites.get("player_dash_d_tunic"),
            "player_dash_u" => sprites.get("player_dash_u_tunic"),
            "player_dash_r" => sprites.get("player_dash_r_tunic"),
            "player_shield" => sprites.get("player_shield_tunic"),
            "player_hurt" => sprites.get("player_hurt_tunic"),
            "player_charge_glow" => sprites.get("player_charge_glow_tunic"),
            "player_slash_d" => sprites.get("player_slash_d_tunic"),
            "player_slash_u" => sprites.get("player_slash_u_tunic"),
            "player_slash_r" => sprites.get("player_slash_r_tunic"),
            "player_bslash_d" => sprites.get("player_bslash_d_tunic"),
            "player_bslash_u" => sprites.get("player_bslash_u_tunic"),
            "player_bslash_r" => sprites.get("player_bslash_r_tunic"),
            "player_lunge_d" => sprites.get("player_lunge_d_tunic"),
            "player_lunge_u" => sprites.get("player_lunge_u_tunic"),
            "player_lunge_r" => sprites.get("player_lunge_r_tunic"),
            "player_spin" => sprites.get("player_spin_tunic"),
            _ => None,
        }
        .or_else(|| sprites.get(key))
    } else {
        sprites.get(key)
    }
}

fn player_sprite(
    facing: Dir4,
    pd: &crate::world::entity::PlayerData,
    e: &Entity,
) -> (&'static str, u32, bool) {
    let flip = facing == Dir4::Left;
    let dir_slot = match facing {
        Dir4::Down => 0,
        Dir4::Up => 1,
        Dir4::Right | Dir4::Left => 2,
    };

    if e.health.map(|h| h.flash > 0).unwrap_or(false) && pd.state == PlayerState::Idle {
        return ("player_hurt", 0, flip);
    }

    match pd.state {
        PlayerState::Swing { stage, tick } => {
            let frames = 3u32;
            let f = ((tick as u32 * frames) / crate::combat::tuning::SLASH_TICKS as u32).min(2);
            let key = match (stage, dir_slot) {
                (0, 0) => "player_slash_d",
                (0, 1) => "player_slash_u",
                (0, _) => "player_slash_r",
                (1, 0) => "player_bslash_d",
                (1, 1) => "player_bslash_u",
                (1, _) => "player_bslash_r",
                (_, 0) => "player_lunge_d",
                (_, 1) => "player_lunge_u",
                _ => "player_lunge_r",
            };
            let f = if stage >= 2 { f.min(1) } else { f };
            (key, f, flip)
        }
        PlayerState::Spin { tick } => ("player_spin", (tick as u32 / 2) % 8, false),
        PlayerState::Dash { tick } | PlayerState::DashRecovery { tick } => {
            let key = match dir_slot {
                0 => "player_dash_d",
                1 => "player_dash_u",
                _ => "player_dash_r",
            };
            (key, (tick as u32 / 4) % 2, flip)
        }
        PlayerState::Charging { .. } => {
            let base = idle_frame(dir_slot, pd.walk_timer);
            ("player_idle", base, flip)
        }
        PlayerState::LedgeHop { .. } => {
            let key = match dir_slot {
                0 => "player_dash_d",
                1 => "player_dash_u",
                _ => "player_dash_r",
            };
            (key, 0, flip)
        }
        PlayerState::Idle => {
            if pd.shield_held {
                ("player_shield", dir_slot.min(2) as u32, flip)
            } else if pd.move_blend > 0.2 || e.vel.len() > 0.2 {
                let key = match dir_slot {
                    0 => "player_walk_d",
                    1 => "player_walk_u",
                    _ => "player_walk_r",
                };
                (key, ((pd.walk_timer / 8) % 4) as u32, flip)
            } else {
                ("player_idle", idle_frame(dir_slot, e.anim.timer), flip)
            }
        }
    }
}

fn idle_frame(dir_slot: i32, timer: u16) -> u32 {
    let breath = ((timer / 24) % 2) as u32;
    match dir_slot {
        0 => breath,
        1 => 2 + breath,
        _ => 4 + breath,
    }
}
