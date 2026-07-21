//! AABB vs tile grid and entity overlap queries.

use content::maps::TILE_PX;

use crate::math::Vec2;

use super::entity::{Body, Entity, EntityId};
use super::World;

const KB_DECAY: f32 = 0.7;
const KB_EPS: f32 = 0.05;

pub fn move_entity(world: &World, entity: &mut Entity) {
    let Some(body) = entity.body else {
        entity.pos = entity.pos.add(entity.vel).add(entity.knockback);
        decay_knockback(entity);
        return;
    };

    let mut dx = entity.vel.x + entity.knockback.x;
    let mut dy = entity.vel.y + entity.knockback.y;

    if body.solid {
        // Axis-separated sweep against solid tiles.
        let center = entity_center(entity, &body);
        let (mut cx, mut cy) = (center.x, center.y);

        cx += dx;
        if collides_tiles(world, cx, cy, body.half) {
            cx -= dx;
            dx = 0.0;
            entity.vel.x = 0.0;
            entity.knockback.x = 0.0;
        }

        cy += dy;
        if collides_tiles(world, cx, cy, body.half) {
            cy -= dy;
            dy = 0.0;
            entity.vel.y = 0.0;
            entity.knockback.y = 0.0;
        }

        // Reconstruct top-left pos from center (player bottom-aligned 16×16).
        entity.pos = pos_from_center(entity, &body, Vec2::new(cx, cy));
        let _ = (dx, dy);
    } else {
        entity.pos = entity.pos.add(Vec2::new(dx, dy));
    }

    decay_knockback(entity);
}

fn entity_center(entity: &Entity, body: &Body) -> Vec2 {
    match entity.kind {
        super::entity::EntityKind::Player => {
            Vec2::new(entity.pos.x + body.half.x, entity.pos.y + body.half.y)
        }
        _ => Vec2::new(entity.pos.x + body.half.x, entity.pos.y + body.half.y),
    }
}

fn pos_from_center(entity: &Entity, body: &Body, center: Vec2) -> Vec2 {
    let _ = entity;
    Vec2::new(center.x - body.half.x, center.y - body.half.y)
}

fn collides_tiles(world: &World, cx: f32, cy: f32, half: Vec2) -> bool {
    let min_x = cx - half.x;
    let max_x = cx + half.x;
    let min_y = cy - half.y;
    let max_y = cy + half.y;
    let tx0 = (min_x / TILE_PX).floor() as i32;
    let ty0 = (min_y / TILE_PX).floor() as i32;
    let tx1 = ((max_x - 0.001) / TILE_PX).floor() as i32;
    let ty1 = ((max_y - 0.001) / TILE_PX).floor() as i32;
    for ty in ty0..=ty1 {
        for tx in tx0..=tx1 {
            if world.map.solid_at(tx, ty) {
                return true;
            }
        }
    }
    false
}

pub fn decay_knockback(entity: &mut Entity) {
    entity.knockback = entity.knockback.scale(KB_DECAY);
    if entity.knockback.len() < KB_EPS {
        entity.knockback = Vec2::ZERO;
    }
}

pub fn aabb_overlap(center: Vec2, half: Vec2, entity: &Entity) -> bool {
    let Some(body) = entity.body else {
        return false;
    };
    let c = entity_center(entity, &body);
    (center.x - c.x).abs() <= half.x + body.half.x
        && (center.y - c.y).abs() <= half.y + body.half.y
}

pub fn query_aabb(world: &World, center: Vec2, half: Vec2, mask: u8) -> Vec<EntityId> {
    let mut out = Vec::new();
    for (id, e) in world.iter_alive() {
        let Some(body) = e.body else {
            continue;
        };
        if body.layer & mask == 0 {
            continue;
        }
        if aabb_overlap(center, half, e) {
            out.push(id);
        }
    }
    out
}

pub fn circle_hits_entity(center: Vec2, radius: f32, entity: &Entity) -> bool {
    let Some(body) = entity.body else {
        return false;
    };
    let c = entity_center(entity, &body);
    let dx = (c.x - center.x).abs() - body.half.x;
    let dy = (c.y - center.y).abs() - body.half.y;
    let ox = dx.max(0.0);
    let oy = dy.max(0.0);
    ox * ox + oy * oy <= radius * radius
}
