//! AABB vs tile grid and entity overlap queries.

use content::maps::{flags, TILE_PX};

use crate::math::Vec2;
use crate::world::entity::{Body, Entity, EntityId, EntityKind, EntityData, PlayerState};
use crate::world::World;

const KB_DECAY: f32 = 0.7;
const KB_EPS: f32 = 0.05;
pub const LEDGE_HOP_TICKS: u16 = 18;
pub const LEDGE_HOP_DIST_TILES: i32 = 2;

pub fn move_entity(world: &World, entity: &mut Entity) {
    let Some(body) = entity.body else {
        entity.pos = entity.pos.add(entity.vel).add(entity.knockback);
        decay_knockback(entity);
        return;
    };

    // Scripted ledge hop (player only).
    if entity.kind == EntityKind::Player {
        if let EntityData::Player(pd) = &mut entity.data {
            if let PlayerState::LedgeHop { tick, from, to } = pd.state {
                let t = tick + 1;
                let u = (t as f32 / LEDGE_HOP_TICKS as f32).min(1.0);
                let arc = (u * std::f32::consts::PI).sin() * 10.0;
                entity.pos.x = from.x + (to.x - from.x) * u;
                entity.pos.y = from.y + (to.y - from.y) * u - arc;
                entity.vel = Vec2::ZERO;
                entity.knockback = Vec2::ZERO;
                if t >= LEDGE_HOP_TICKS {
                    entity.pos = to;
                    pd.state = PlayerState::Idle;
                } else {
                    pd.state = PlayerState::LedgeHop { tick: t, from, to };
                }
                return;
            }
        }
    }

    let mut dx = entity.vel.x + entity.knockback.x;
    let mut dy = entity.vel.y + entity.knockback.y;

    if body.solid {
        let center = entity_center(entity, &body);
        let (mut cx, mut cy) = (center.x, center.y);
        let is_player = entity.kind == EntityKind::Player;

        cx += dx;
        if let Some(block) = tile_block(world, cx, cy, body.half, is_player, dx, 0.0) {
            if is_player {
                if let Some(hop) = try_ledge_hop(world, entity, &body, block, dx, dy) {
                    if let EntityData::Player(pd) = &mut entity.data {
                        pd.state = PlayerState::LedgeHop {
                            tick: 0,
                            from: hop.0,
                            to: hop.1,
                        };
                    }
                    entity.vel = Vec2::ZERO;
                    entity.knockback = Vec2::ZERO;
                    return;
                }
            }
            cx -= dx;
            dx = 0.0;
            entity.vel.x = 0.0;
            entity.knockback.x = 0.0;
        }

        cy += dy;
        if let Some(block) = tile_block(world, cx, cy, body.half, is_player, 0.0, dy) {
            if is_player {
                if let Some(hop) = try_ledge_hop(world, entity, &body, block, dx, dy) {
                    if let EntityData::Player(pd) = &mut entity.data {
                        pd.state = PlayerState::LedgeHop {
                            tick: 0,
                            from: hop.0,
                            to: hop.1,
                        };
                    }
                    entity.vel = Vec2::ZERO;
                    entity.knockback = Vec2::ZERO;
                    return;
                }
            }
            cy -= dy;
            dy = 0.0;
            entity.vel.y = 0.0;
            entity.knockback.y = 0.0;
        }

        entity.pos = pos_from_center(entity, &body, Vec2::new(cx, cy));
        let _ = (dx, dy);
    } else {
        entity.pos = entity.pos.add(Vec2::new(dx, dy));
    }

    decay_knockback(entity);
}

struct BlockInfo {
    tx: i32,
    ty: i32,
    f: u8,
}

fn tile_block(
    world: &World,
    cx: f32,
    cy: f32,
    half: Vec2,
    _is_player: bool,
    _dx: f32,
    _dy: f32,
) -> Option<BlockInfo> {
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
            let f = world.map.flags_at(tx, ty);
            // SOLID (includes water in Act 1) blocks walkers.
            if f & flags::SOLID != 0 {
                return Some(BlockInfo { tx, ty, f });
            }
        }
    }
    None
}

fn try_ledge_hop(
    world: &World,
    entity: &Entity,
    body: &Body,
    block: BlockInfo,
    dx: f32,
    dy: f32,
) -> Option<(Vec2, Vec2)> {
    let ledge_dir = if block.f & flags::LEDGE_S != 0 && dy > 0.1 {
        (0, 1)
    } else if block.f & flags::LEDGE_N != 0 && dy < -0.1 {
        (0, -1)
    } else if block.f & flags::LEDGE_E != 0 && dx > 0.1 {
        (1, 0)
    } else if block.f & flags::LEDGE_W != 0 && dx < -0.1 {
        (-1, 0)
    } else {
        return None;
    };

    let land_tx = block.tx + ledge_dir.0 * LEDGE_HOP_DIST_TILES;
    let land_ty = block.ty + ledge_dir.1 * LEDGE_HOP_DIST_TILES;
    if world.map.solid_at(land_tx, land_ty) {
        // Scan one more tile.
        let land_tx = block.tx + ledge_dir.0 * (LEDGE_HOP_DIST_TILES + 1);
        let land_ty = block.ty + ledge_dir.1 * (LEDGE_HOP_DIST_TILES + 1);
        if world.map.solid_at(land_tx, land_ty) {
            return None;
        }
        let from = entity.pos;
        let to = Vec2::new(land_tx as f32 * TILE_PX, land_ty as f32 * TILE_PX);
        let _ = body;
        return Some((from, to));
    }
    let from = entity.pos;
    let to = Vec2::new(land_tx as f32 * TILE_PX, land_ty as f32 * TILE_PX);
    Some((from, to))
}

fn entity_center(entity: &Entity, body: &Body) -> Vec2 {
    Vec2::new(entity.pos.x + body.half.x, entity.pos.y + body.half.y)
}

fn pos_from_center(_entity: &Entity, body: &Body, center: Vec2) -> Vec2 {
    Vec2::new(center.x - body.half.x, center.y - body.half.y)
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
