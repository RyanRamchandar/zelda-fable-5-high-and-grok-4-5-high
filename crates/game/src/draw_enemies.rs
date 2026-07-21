//! Enemy / projectile sprite drawing (keeps `draw_world` under the file cap).

use engine::render::Draw;

use crate::assets::SpriteMap;
use crate::math::Dir4;
use crate::world::entity::{Entity, EntityData, EntityKind, WispState};

pub fn try_render_enemy(d: &mut Draw, e: &Entity, sprites: &SpriteMap) -> bool {
    match e.kind {
        EntityKind::Dummy => {
            if let EntityData::Dummy(dd) = &e.data {
                if dd.dead_ticks.is_some() {
                    return true;
                }
            }
            if flashing(e) {
                return true;
            }
            if let Some(h) = sprites.get("slime_dummy") {
                let frame = ((e.anim.timer / 16) % 2) as u32;
                d.sprite(h, frame, e.pos.x, e.pos.y, false);
            }
            true
        }
        EntityKind::Slime => {
            if e.body.is_none() {
                d.circle(e.pos.x + 8.0, e.pos.y + 8.0, 6.0, "rgba(120,220,140,0.35)");
                return true;
            }
            if flashing(e) {
                return true;
            }
            let angry = e.health.map(|h| h.hp <= 1).unwrap_or(false);
            let key = if angry { "slime_angry" } else { "slime" };
            if let Some(h) = sprites.get(key) {
                d.sprite(h, e.anim.frame as u32, e.pos.x, e.pos.y, false);
            }
            true
        }
        EntityKind::Bat => {
            if e.body.is_none() {
                d.circle(e.pos.x + 8.0, e.pos.y + 8.0, 5.0, "rgba(160,120,200,0.35)");
                return true;
            }
            if flashing(e) {
                return true;
            }
            if let Some(h) = sprites.get("bat") {
                d.sprite(h, e.anim.frame as u32, e.pos.x, e.pos.y, e.facing == Dir4::Left);
            }
            true
        }
        EntityKind::Octorok => {
            if e.body.is_none() {
                d.circle(e.pos.x + 8.0, e.pos.y + 8.0, 6.0, "rgba(220,120,80,0.35)");
                return true;
            }
            if flashing(e) {
                return true;
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
            true
        }
        EntityKind::OctorokRock => {
            let key = match &e.data {
                EntityData::Rock(r) if r.from_player => "octorok_rock_warm",
                _ => "octorok_rock",
            };
            if let Some(h) = sprites.get(key) {
                d.sprite(h, e.anim.frame as u32, e.pos.x, e.pos.y, false);
            }
            true
        }
        EntityKind::RaiderSpear => {
            render_tall(d, e, sprites, "raider_spear", 5, "rgba(200,80,60,0.35)");
            true
        }
        EntityKind::RaiderTorch => {
            render_tall(d, e, sprites, "raider_torch", 2, "rgba(220,120,40,0.35)");
            true
        }
        EntityKind::Skeleton => {
            render_tall(d, e, sprites, "skeleton", 4, "rgba(220,220,200,0.35)");
            true
        }
        EntityKind::Ironshell => {
            if e.body.is_none() {
                d.circle(e.pos.x + 12.0, e.pos.y + 12.0, 10.0, "rgba(160,160,180,0.35)");
                return true;
            }
            if flashing(e) {
                return true;
            }
            if let Some(h) = sprites.get("ironshell") {
                d.sprite(
                    h,
                    e.anim.frame.min(3) as u32,
                    e.pos.x,
                    e.pos.y,
                    e.facing == Dir4::Left,
                );
            }
            true
        }
        EntityKind::GraniteWarden => {
            if flashing(e) {
                return true;
            }
            if let Some(h) = sprites.get("granite_warden") {
                d.sprite(
                    h,
                    e.anim.frame.min(3) as u32,
                    e.pos.x,
                    e.pos.y,
                    false,
                );
            } else {
                d.rect(e.pos.x, e.pos.y, 48.0, 48.0, "#6a6058");
            }
            true
        }
        EntityKind::WindCrystal => {
            if let Some(h) = sprites.get("wind_crystal") {
                d.sprite(h, e.anim.frame.min(1) as u32, e.pos.x, e.pos.y, false);
            } else {
                d.circle(e.pos.x + 8.0, e.pos.y + 8.0, 6.0, "#60c0ff");
            }
            true
        }
        EntityKind::PebbleCrawler => {
            if flashing(e) {
                return true;
            }
            if let Some(h) = sprites.get("pebble") {
                d.sprite(h, e.anim.frame.min(1) as u32, e.pos.x, e.pos.y, false);
            } else {
                d.circle(e.pos.x + 8.0, e.pos.y + 8.0, 5.0, "#8a8070");
            }
            true
        }
        EntityKind::Wisp => {
            let (telegraph, phased) = match &e.data {
                EntityData::Wisp(d) => (
                    d.spawn_telegraph > 0,
                    matches!(d.state, WispState::Phased | WispState::FadeOut),
                ),
                _ => (false, false),
            };
            if telegraph {
                d.circle(e.pos.x + 8.0, e.pos.y + 8.0, 5.0, "rgba(255,180,60,0.35)");
                return true;
            }
            if flashing(e) {
                return true;
            }
            if let Some(h) = sprites.get("wisp") {
                if phased {
                    d.sprite(h, 2, e.pos.x, e.pos.y, false);
                } else {
                    d.sprite(h, e.anim.frame.min(1) as u32, e.pos.x, e.pos.y, false);
                }
            }
            true
        }
        EntityKind::TorchProj => {
            if let Some(h) = sprites.get("torch_proj") {
                d.sprite(h, e.anim.frame.min(1) as u32, e.pos.x, e.pos.y, false);
            } else {
                d.circle(e.pos.x + 4.0, e.pos.y + 4.0, 3.0, "#ff8020");
            }
            true
        }
        EntityKind::TorchFlame => {
            if let Some(h) = sprites.get("torch_flame") {
                d.sprite(h, e.anim.frame.min(1) as u32, e.pos.x, e.pos.y, false);
            } else {
                d.circle(e.pos.x + 8.0, e.pos.y + 8.0, 8.0, "rgba(255,100,20,0.55)");
            }
            true
        }
        _ => false,
    }
}

fn flashing(e: &Entity) -> bool {
    let flash = e.health.map(|h| h.flash > 0).unwrap_or(false);
    flash && e.health.unwrap().flash.is_multiple_of(2)
}

fn render_tall(
    d: &mut Draw,
    e: &Entity,
    sprites: &SpriteMap,
    key: &str,
    max_frame: u16,
    telegraph: &str,
) {
    if e.body.is_none() {
        d.circle(e.pos.x + 8.0, e.pos.y + 8.0, 6.0, telegraph);
        return;
    }
    if flashing(e) {
        return;
    }
    if let Some(h) = sprites.get(key) {
        d.sprite(
            h,
            e.anim.frame.min(max_frame) as u32,
            e.pos.x,
            e.pos.y - 8.0,
            e.facing == Dir4::Left,
        );
    }
}
