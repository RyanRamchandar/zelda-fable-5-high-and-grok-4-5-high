//! Atlas-backed map + entity drawing.

use content::maps::{FOUNTAIN, TILE_PX, WALL};
use engine::render::Draw;

use crate::assets::SpriteMap;
use crate::math::Dir4;
use crate::world::entity::{Entity, EntityData, EntityKind, PlayerState};
use crate::world::World;

pub fn render_map(d: &mut Draw, world: &World, sprites: &SpriteMap) {
    let map = &world.map;
    let cam = world.camera.offset();
    let x0 = ((cam.x / TILE_PX).floor() as i32).max(0) as u32;
    let y0 = ((cam.y / TILE_PX).floor() as i32).max(0) as u32;
    let x1 = ((cam.x + 480.0) / TILE_PX).ceil() as u32 + 1;
    let y1 = ((cam.y + 270.0) / TILE_PX).ceil() as u32 + 1;
    let x1 = x1.min(map.width);
    let y1 = y1.min(map.height);

    let floor_a = sprites.get("floor_a");
    let floor_b = sprites.get("floor_b");
    let wall_face = sprites.get("wall_face");
    let wall_top = sprites.get("wall_top");
    let fountain = sprites.get("fountain");
    let pillar = sprites.get("pillar");

    for ty in y0..y1 {
        for tx in x0..x1 {
            let tile = map.ground[map.idx(tx, ty)];
            let x = tx as f32 * TILE_PX;
            let y = ty as f32 * TILE_PX;
            let checker = ((tx + ty) % 2) == 0;
            match tile {
                WALL => {
                    // pillar-sized interior blocks use pillar art when 2x2 cluster
                    let is_pillar = is_interior_pillar(map, tx, ty);
                    if is_pillar {
                        if let Some(h) = pillar {
                            d.sprite(h, 0, x, y, false);
                        } else if let Some(h) = wall_face {
                            d.sprite(h, 0, x, y, false);
                        }
                    } else if ty > 0 && map.ground[map.idx(tx, ty - 1)] != WALL {
                        if let Some(h) = wall_top {
                            d.sprite(h, 0, x, y, false);
                        }
                    } else if let Some(h) = wall_face {
                        d.sprite(h, 0, x, y, false);
                    }
                }
                FOUNTAIN => {
                    let frame = ((world.tick / 16) % 2) as u32;
                    if let Some(h) = fountain {
                        d.sprite(h, frame, x, y, false);
                    }
                }
                _ => {
                    let h = if checker { floor_a } else { floor_b };
                    if let Some(h) = h {
                        d.sprite(h, 0, x, y, false);
                    }
                }
            }
        }
    }
}

fn is_interior_pillar(map: &content::maps::MapDef, tx: u32, ty: u32) -> bool {
    // Arena pillars are 2×2 WALL blocks inset from border
    if tx == 0 || ty == 0 || tx + 1 >= map.width || ty + 1 >= map.height {
        return false;
    }
    map.ground[map.idx(tx, ty)] == WALL
        && map.collision[map.idx(tx, ty)]
        && tx > 2
        && ty > 2
        && tx < map.width - 3
        && ty < map.height - 3
}

pub fn render_entity(d: &mut Draw, e: &Entity, sprites: &SpriteMap) {
    match e.kind {
        EntityKind::Player => render_player(d, e, sprites),
        EntityKind::Dummy => {
            if let EntityData::Dummy(dd) = &e.data {
                if dd.dead_ticks.is_some() {
                    return;
                }
            }
            let flash = e.health.map(|h| h.flash > 0).unwrap_or(false);
            if flash && e.health.unwrap().flash.is_multiple_of(2) {
                return;
            }
            if let Some(h) = sprites.get("slime_dummy") {
                let frame = ((e.anim.timer / 16) % 2) as u32;
                d.sprite(h, frame, e.pos.x, e.pos.y, false);
            }
        }
        EntityKind::Slime => {
            if e.body.is_none() {
                // telegraph shimmer
                d.circle(e.pos.x + 8.0, e.pos.y + 8.0, 6.0, "rgba(120,220,140,0.35)");
                return;
            }
            let flash = e.health.map(|h| h.flash > 0).unwrap_or(false);
            if flash && e.health.unwrap().flash.is_multiple_of(2) {
                return;
            }
            let angry = e.health.map(|h| h.hp <= 1).unwrap_or(false);
            let key = if angry { "slime_angry" } else { "slime" };
            if let Some(h) = sprites.get(key) {
                d.sprite(h, e.anim.frame as u32, e.pos.x, e.pos.y, false);
            }
        }
        EntityKind::Bat => {
            if e.body.is_none() {
                d.circle(e.pos.x + 8.0, e.pos.y + 8.0, 5.0, "rgba(160,120,200,0.35)");
                return;
            }
            let flash = e.health.map(|h| h.flash > 0).unwrap_or(false);
            if flash && e.health.unwrap().flash.is_multiple_of(2) {
                return;
            }
            if let Some(h) = sprites.get("bat") {
                d.sprite(h, e.anim.frame as u32, e.pos.x, e.pos.y, e.facing == Dir4::Left);
            }
        }
        EntityKind::Octorok => {
            if e.body.is_none() {
                d.circle(e.pos.x + 8.0, e.pos.y + 8.0, 6.0, "rgba(220,120,80,0.35)");
                return;
            }
            let flash = e.health.map(|h| h.flash > 0).unwrap_or(false);
            if flash && e.health.unwrap().flash.is_multiple_of(2) {
                return;
            }
            if let Some(h) = sprites.get("octorok") {
                d.sprite(
                    h,
                    e.anim.frame as u32,
                    e.pos.x,
                    e.pos.y,
                    e.facing == Dir4::Left,
                );
            }
        }
        EntityKind::OctorokRock => {
            let key = match &e.data {
                EntityData::Rock(r) if r.from_player => "octorok_rock_warm",
                _ => "octorok_rock",
            };
            if let Some(h) = sprites.get(key) {
                d.sprite(h, e.anim.frame as u32, e.pos.x, e.pos.y, false);
            }
        }
        EntityKind::Pickup => {
            if let EntityData::Pickup(pd) = &e.data {
                if pd.life < crate::combat::tuning::PICKUP_BLINK && (pd.life / 4) % 2 == 0 {
                    return;
                }
                match pd.kind {
                    crate::world::entity::PickupKind::Rupee => {
                        d.rect(e.pos.x, e.pos.y, 6.0, 8.0, "#40e080");
                    }
                    crate::world::entity::PickupKind::Heart => {
                        if let Some(h) = sprites.get("heart_full") {
                            d.sprite(h, 0, e.pos.x, e.pos.y, false);
                        }
                    }
                    crate::world::entity::PickupKind::Energy => {
                        d.circle(e.pos.x + 3.0, e.pos.y + 3.0, 3.0, "#40e0ff");
                    }
                }
            }
        }
        EntityKind::SwordBeam => {
            d.rect(e.pos.x, e.pos.y, 6.0, 6.0, "#c0e0ff");
        }
        EntityKind::DebugShot => {
            d.rect(e.pos.x, e.pos.y, 6.0, 6.0, "#ff6060");
        }
        EntityKind::FairyFountain => {}
    }
}

fn render_player(d: &mut Draw, e: &Entity, sprites: &SpriteMap) {
    let iframes = e.health.map(|h| h.iframes > 0).unwrap_or(false);
    if iframes && (e.health.unwrap().iframes / 2).is_multiple_of(2) {
        return;
    }
    let flash = e.health.map(|h| h.flash > 0).unwrap_or(false);
    let EntityData::Player(pd) = &e.data else {
        return;
    };

    let (key, frame, flip) = player_sprite(e.facing, pd, e);
    if flash {
        // white flash: draw slightly offset cream via charge glow underlay skip
    }
    if matches!(pd.state, PlayerState::Charging { tick } if tick >= 20) {
        if let Some(h) = sprites.get("player_charge_glow") {
            d.sprite(h, 0, e.pos.x, e.pos.y - 8.0, flip);
        }
    }
    if let Some(h) = sprites.get(key) {
        d.sprite(h, frame, e.pos.x, e.pos.y - 8.0, flip);
    }
}

fn player_sprite(facing: Dir4, pd: &crate::world::entity::PlayerData, e: &Entity) -> (&'static str, u32, bool) {
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
            let f = if stage >= 2 { (f).min(1) } else { f };
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
