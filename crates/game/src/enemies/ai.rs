//! Shared enemy helpers.

use content::maps::TILE_PX;

use crate::math::Vec2;
use crate::world::entity::{Entity, EntityId, EntityKind};
use crate::world::physics::{aabb_overlap, move_entity};
use crate::world::World;

pub fn player_pos(world: &World) -> Option<Vec2> {
    world.get(world.player_id).map(|p| p.center())
}

/// Cheap LOS: no solid tiles along a coarse line.
pub fn has_los(world: &World, from: Vec2, to: Vec2) -> bool {
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    let steps = ((dx.abs() + dy.abs()) / 8.0).ceil().max(1.0) as i32;
    for i in 1..=steps {
        let t = i as f32 / steps as f32;
        let x = from.x + dx * t;
        let y = from.y + dy * t;
        let tx = (x / TILE_PX).floor() as i32;
        let ty = (y / TILE_PX).floor() as i32;
        if world.map.solid_at(tx, ty) {
            return false;
        }
    }
    true
}

pub fn steer_toward(from: Vec2, to: Vec2, speed: f32) -> Vec2 {
    to.sub(from).normalize_or_zero().scale(speed)
}

pub fn move_enemy(world: &mut World, id: EntityId) {
    let slot = id.index as usize;
    if slot >= world.arena.len() || world.arena[slot].gen != id.gen {
        return;
    }
    let mut entity = match world.arena[slot].entity.take() {
        Some(e) => e,
        None => return,
    };
    move_entity(world, &mut entity);
    world.arena[slot].entity = Some(entity);
}

/// Fly without tile collision (bat): apply velocity + knockback only.
pub fn fly_enemy(world: &mut World, id: EntityId) {
    let max_x = world.map.width as f32 * TILE_PX - 16.0;
    let max_y = world.map.height as f32 * TILE_PX - 16.0;
    let Some(e) = world.get_mut(id) else {
        return;
    };
    let kb = e.knockback;
    e.pos = e.pos.add(e.vel).add(kb);
    e.knockback = kb.scale(0.75);
    if e.knockback.len() < 0.05 {
        e.knockback = Vec2::ZERO;
    }
    e.pos.x = e.pos.x.clamp(16.0, max_x);
    e.pos.y = e.pos.y.clamp(16.0, max_y);
}

pub fn overlaps_player(world: &World, e: &Entity) -> bool {
    let Some(p) = world.get(world.player_id) else {
        return false;
    };
    let Some(pb) = p.body else {
        return false;
    };
    aabb_overlap(p.center(), pb.half, e)
}

pub fn count_alive_enemies(world: &World) -> usize {
    // Count telegraphing spawns too (body is None until SPAWN_TELEGRAPH ends).
    world
        .iter_alive()
        .filter(|(_, e)| {
            matches!(
                e.kind,
                EntityKind::Slime
                    | EntityKind::Bat
                    | EntityKind::Octorok
                    | EntityKind::RaiderSpear
                    | EntityKind::RaiderTorch
                    | EntityKind::Wisp
                    | EntityKind::Skeleton
            )
        })
        .count()
}

pub fn edge_spawn_points(world: &World) -> [Vec2; 6] {
    let w = world.map.width as f32 * TILE_PX;
    let h = world.map.height as f32 * TILE_PX;
    [
        Vec2::new(48.0, h * 0.5),
        Vec2::new(w - 64.0, h * 0.5),
        Vec2::new(w * 0.5, 48.0),
        Vec2::new(w * 0.5, h - 64.0),
        Vec2::new(80.0, 80.0),
        Vec2::new(w - 96.0, h - 96.0),
    ]
}

pub fn spawn_clear_of_player(world: &World, candidates: &[Vec2]) -> Vec2 {
    let p = player_pos(world).unwrap_or(Vec2::new(480.0, 300.0));
    let mut best = candidates[0];
    let mut best_d = 0.0;
    for &c in candidates {
        let d = c.sub(p).len();
        if d > best_d {
            best_d = d;
            best = c;
        }
    }
    best
}
