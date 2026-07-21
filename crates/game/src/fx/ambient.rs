//! Region ambient particles (leaves, lantern embers, fountain sparkle).

use content::maps::{catalog, MapId, TILE_PX};

use crate::fx::particles::{ParticleKind, Particles};
use crate::math::Vec2;
use crate::world::World;

const AMBIENT_CAP: usize = 24;

/// Spawn capped ambient FX near the camera. Skip when chunk path fell back to direct draw.
pub fn tick(world: &mut World, particles: &mut Particles, map_id: MapId, direct: bool) {
    if direct || map_id != MapId::Overworld {
        return;
    }
    let ambient_live = particles.ambient_count();
    if ambient_live >= AMBIENT_CAP {
        return;
    }
    let Some(player) = world.get(world.player_id) else {
        return;
    };
    let ppos = player.center();
    let region = region_at(&world.map.regions, ppos);
    let budget = AMBIENT_CAP - ambient_live;

    match region {
        // Mosslight Village
        Some(1) => {
            if world.tick.is_multiple_of(18) && budget > 0 {
                spawn_near_tile(
                    world,
                    particles,
                    ppos,
                    catalog::T_LANTERN,
                    ParticleKind::Ember,
                    48.0,
                );
            }
            if world.tick.is_multiple_of(22) && budget > 1 {
                spawn_near_tile(
                    world,
                    particles,
                    ppos,
                    catalog::T_FOUNTAIN,
                    ParticleKind::Fountain,
                    40.0,
                );
            }
            if world.tick.is_multiple_of(28) && budget > 2 {
                particles.spawn_ambient_leaf(ppos, &mut world.rng);
            }
        }
        // Whispering Grove
        Some(2) if world.tick.is_multiple_of(14) && budget > 0 => {
            particles.spawn_ambient_leaf(ppos, &mut world.rng);
        }
        _ => {}
    }
}

fn region_at(regions: &[content::maps::RegionDef], pos: Vec2) -> Option<u8> {
    let tx = (pos.x / TILE_PX) as u32;
    let ty = (pos.y / TILE_PX) as u32;
    for (i, r) in regions.iter().enumerate() {
        let (x0, y0, x1, y1) = r.rect;
        if tx >= x0 && tx <= x1 && ty >= y0 && ty <= y1 {
            return Some(i as u8);
        }
    }
    None
}

fn spawn_near_tile(
    world: &mut World,
    particles: &mut Particles,
    ppos: Vec2,
    tile: u16,
    kind: ParticleKind,
    radius: f32,
) {
    let cam = world.camera.offset();
    let tx0 = ((cam.x / TILE_PX).floor() as i32).max(0) as u32;
    let ty0 = ((cam.y / TILE_PX).floor() as i32).max(0) as u32;
    let tx1 = ((cam.x + 480.0) / TILE_PX).ceil() as u32;
    let ty1 = ((cam.y + 270.0) / TILE_PX).ceil() as u32;
    let tx1 = tx1.min(world.map.width.saturating_sub(1));
    let ty1 = ty1.min(world.map.height.saturating_sub(1));
    for ty in ty0..=ty1 {
        for tx in tx0..=tx1 {
            let id = world.map.get(tx, ty, content::maps::TileLayer::Detail);
            let id = if id == tile {
                id
            } else {
                world.map.get(tx, ty, content::maps::TileLayer::Ground)
            };
            if id != tile {
                continue;
            }
            let pos = Vec2::new(tx as f32 * TILE_PX + 8.0, ty as f32 * TILE_PX + 8.0);
            if pos.sub(ppos).len() > radius + 80.0 {
                continue;
            }
            match kind {
                ParticleKind::Ember => particles.spawn_ambient_ember(pos, &mut world.rng),
                ParticleKind::Fountain => particles.spawn_fountain(pos, &mut world.rng),
                ParticleKind::Leaf => particles.spawn_ambient_leaf(pos, &mut world.rng),
                _ => {}
            }
            return;
        }
    }
}
