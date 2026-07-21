//! Gale Boomerang: throw / return / stun / puzzle hits.

use content::audio::sfx::SfxId;
use content::maps::flags as tile_flags;
use content::maps::TILE_PX;

use crate::fx::FxKind;
use crate::math::{Dir4, Vec2};
use crate::world::entity::{
    layer, AnimState, Body, BoomerangData, BoomerangPhase, Entity, EntityData, EntityKind,
};
use crate::world::{World, WorldEvent};
use crate::Game;

const SPEED_OUT: f32 = 3.4;
const SPEED_RET: f32 = 4.2;
const MAX_RANGE: f32 = 112.0;
const CATCH_R: f32 = 10.0;

pub fn try_throw(world: &mut World) -> bool {
    if boomerang_in_flight(world) {
        return false;
    }
    let (center, dir, throw_id) = {
        let Some(p) = world.get_mut(world.player_id) else {
            return false;
        };
        let EntityData::Player(pd) = &mut p.data else {
            return false;
        };
        if pd.selected_item != 2 {
            return false;
        }
        pd.swing_id = pd.swing_id.wrapping_add(1);
        let throw_id = pd.swing_id;
        let mut dir = p.facing.unit();
        // Prefer 8-way from velocity if moving.
        if p.vel.len() > 0.4 {
            dir = p.vel.normalize_or_zero();
        }
        (p.center(), dir, throw_id)
    };
    let pos = center.sub(Vec2::new(8.0, 8.0));
    world.spawn(Entity {
        kind: EntityKind::Boomerang,
        pos,
        vel: dir.scale(SPEED_OUT),
        facing: Dir4::from_vec(dir, Dir4::Down),
        body: Some(Body {
            half: Vec2::new(6.0, 6.0),
            solid: false,
            layer: layer::PLAYER_HIT,
            mask: layer::ENEMY_BODY | layer::PICKUP,
        }),
        health: None,
        knockback: Vec2::ZERO,
        anim: AnimState::default(),
        data: EntityData::Boomerang(BoomerangData {
            dir,
            phase: BoomerangPhase::Out,
            traveled: 0.0,
            throw_id,
            flame: false,
            catch_buffer: false,
        }),
        alive: true,
    });
    world.push_event(WorldEvent::Sfx(SfxId::BoomerangThrow));
    true
}

fn boomerang_in_flight(world: &World) -> bool {
    world
        .iter_alive()
        .any(|(_, e)| e.kind == EntityKind::Boomerang)
}

pub fn update(game: &mut Game) {
    let ids: Vec<_> = game
        .world
        .alive_ids()
        .into_iter()
        .filter(|&id| {
            game.world
                .get(id)
                .map(|e| e.kind == EntityKind::Boomerang)
                .unwrap_or(false)
        })
        .collect();
    for id in ids {
        tick_one(game, id);
    }
}

fn tick_one(game: &mut Game, id: crate::world::EntityId) {
    let player_c = game
        .world
        .get(game.world.player_id)
        .map(|p| p.center())
        .unwrap_or(Vec2::ZERO);

    let (phase, traveled, dir, flame, throw_id, center) = {
        let Some(e) = game.world.get(id) else {
            return;
        };
        let EntityData::Boomerang(b) = &e.data else {
            return;
        };
        (b.phase, b.traveled, b.dir, b.flame, b.throw_id, e.center())
    };

    match phase {
        BoomerangPhase::Out => {
            let next = center.add(dir.scale(SPEED_OUT));
            let tx = (next.x / TILE_PX).floor() as i32;
            let ty = (next.y / TILE_PX).floor() as i32;
            let solid = game.world.map.flags_at(tx, ty) & tile_flags::SOLID != 0;
            // Flies over WATER (recommendation logged in WORKER_NOTES).
            let new_traveled = traveled + SPEED_OUT;
            if solid || new_traveled >= MAX_RANGE {
                if let Some(e) = game.world.get_mut(id) {
                    if let EntityData::Boomerang(b) = &mut e.data {
                        b.phase = BoomerangPhase::Return;
                    }
                }
            } else if let Some(e) = game.world.get_mut(id) {
                e.pos = next.sub(Vec2::new(6.0, 6.0));
                if let EntityData::Boomerang(b) = &mut e.data {
                    b.traveled = new_traveled;
                }
                e.anim.timer = e.anim.timer.wrapping_add(1);
                e.anim.frame = (e.anim.timer / 3) % 4;
            }
        }
        BoomerangPhase::Return => {
            let to = player_c.sub(center);
            let len = to.len().max(0.001);
            let steer = to.scale(1.0 / len);
            let next = center.add(steer.scale(SPEED_RET));
            if center.sub(player_c).len() <= CATCH_R {
                let buffer = game
                    .world
                    .get(id)
                    .and_then(|e| match &e.data {
                        EntityData::Boomerang(b) => Some(b.catch_buffer),
                        _ => None,
                    })
                    .unwrap_or(false);
                game.world.despawn(id);
                game.world
                    .push_event(WorldEvent::Sfx(SfxId::BoomerangCatch));
                game.world.push_event(WorldEvent::FxRequest(FxKind::Impact {
                    pos: player_c,
                }));
                if buffer {
                    let _ = try_throw(&mut game.world);
                }
                return;
            }
            if let Some(e) = game.world.get_mut(id) {
                e.pos = next.sub(Vec2::new(6.0, 6.0));
                if let EntityData::Boomerang(b) = &mut e.data {
                    b.dir = steer;
                }
                e.anim.timer = e.anim.timer.wrapping_add(1);
                e.anim.frame = (e.anim.timer / 3) % 4;
            }
        }
    }

    magnetize_pickups(game, id);
    let _ = (flame, throw_id);
}

fn magnetize_pickups(game: &mut Game, boom_id: crate::world::EntityId) {
    let Some(boom) = game.world.get(boom_id).map(|e| e.center()) else {
        return;
    };
    let ids = game.world.alive_ids();
    for id in ids {
        let Some(e) = game.world.get_mut(id) else {
            continue;
        };
        if e.kind != EntityKind::Pickup {
            continue;
        }
        if e.center().sub(boom).len() < 14.0 {
            e.pos = boom.sub(Vec2::new(4.0, 4.0));
        }
    }
}

/// Buffer a re-throw if tap lands during catch window.
pub fn note_catch_buffer(world: &mut World) {
    for (_, e) in world.iter_alive() {
        // can't mut through iter — collect
        let _ = e;
    }
    let ids = world.alive_ids();
    for id in ids {
        if let Some(e) = world.get_mut(id) {
            if let EntityData::Boomerang(b) = &mut e.data {
                if matches!(b.phase, BoomerangPhase::Return) {
                    b.catch_buffer = true;
                }
            }
        }
    }
}

