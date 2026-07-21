//! Pickup magnetism, collection, and kill drops.

use crate::combat::energy;
use crate::combat::style;
use crate::combat::tuning;
use crate::math::Vec2;
use crate::world::entity::{
    layer, AnimState, Body, Entity, EntityData, EntityKind, PickupData, PickupKind,
};
use crate::world::{World, WorldEvent};
use content::audio::sfx::SfxId;

pub fn update(world: &mut World) {
    let pid = world.player_id;
    let (pcenter, phalf) = {
        let Some(p) = world.get(pid) else {
            return;
        };
        let half = p.body.map(|b| b.half).unwrap_or(Vec2::new(8.0, 8.0));
        (p.center(), half)
    };

    let ids = world.alive_ids();
    let mut despawn = Vec::new();
    let mut collect = Vec::new();

    for id in ids {
        let Some(e) = world.get_mut(id) else {
            continue;
        };
        if e.kind != EntityKind::Pickup {
            continue;
        }
        let center = e.center();
        let (life, kind) = match &mut e.data {
            EntityData::Pickup(pd) => {
                pd.life = pd.life.saturating_sub(1);
                (pd.life, pd.kind)
            }
            _ => continue,
        };
        if life == 0 {
            despawn.push(id);
            continue;
        }
        let dist = center.sub(pcenter).len();
        if dist < tuning::PICKUP_MAGNET {
            let dir = pcenter.sub(center).normalize_or_zero();
            e.vel = e.vel.add(dir.scale(0.35)).scale(0.9);
            e.pos = e.pos.add(e.vel);
        }
        if dist < phalf.x + 4.0 {
            collect.push((id, kind));
        }
    }

    for id in despawn {
        world.despawn(id);
    }

    for (id, kind) in collect {
        apply_pickup(world, kind);
        world.despawn(id);
    }
}

fn apply_pickup(world: &mut World, kind: PickupKind) {
    let pid = world.player_id;
    let sfx = {
        let Some(p) = world.get_mut(pid) else {
            return;
        };
        let EntityData::Player(pd) = &mut p.data else {
            return;
        };
        match kind {
            PickupKind::Rupee => {
                pd.rupees = pd.rupees.saturating_add(1);
                SfxId::PickupRupee
            }
            PickupKind::Heart => {
                if let Some(h) = p.health.as_mut() {
                    h.hp = (h.hp + 1).min(h.max);
                    pd.hearts = h.hp;
                }
                SfxId::PickupHeart
            }
            PickupKind::Energy => {
                energy::fill(pd, tuning::ENERGY_ORB);
                SfxId::PickupEnergy
            }
        }
    };
    world.push_event(WorldEvent::Sfx(sfx));
}

pub fn spawn_drops(world: &mut World, pos: Vec2) {
    // 1B: slightly richer drops so wave combat doesn't starve energy (was 70% nothing).
    let roll = world.rng.f32();
    if roll < 0.55 {
        // nothing
    } else if roll < 0.72 {
        spawn_one(world, pos, PickupKind::Rupee);
    } else if roll < 0.84 {
        let low_hp = world
            .get(world.player_id)
            .and_then(|p| p.health)
            .map(|h| h.hp <= h.max / 2)
            .unwrap_or(false);
        if low_hp {
            spawn_one(world, pos, PickupKind::Heart);
        } else {
            spawn_one(world, pos, PickupKind::Rupee);
        }
    } else {
        spawn_one(world, pos, PickupKind::Energy);
    }

    let s_bonus = world
        .get(world.player_id)
        .and_then(|p| match &p.data {
            EntityData::Player(pd) => Some(style::s_rank_bonus_drops(pd)),
            _ => None,
        })
        .unwrap_or(false);
    if s_bonus {
        spawn_one(world, pos.add(Vec2::new(4.0, 0.0)), PickupKind::Rupee);
        spawn_one(world, pos.add(Vec2::new(-4.0, 2.0)), PickupKind::Energy);
    }
}

fn spawn_one(world: &mut World, pos: Vec2, kind: PickupKind) {
    world.spawn(Entity {
        kind: EntityKind::Pickup,
        pos: pos.sub(Vec2::new(3.0, 3.0)),
        vel: Vec2::ZERO,
        facing: crate::math::Dir4::Down,
        body: Some(Body {
            half: Vec2::new(3.0, 3.0),
            solid: false,
            layer: layer::PICKUP,
            mask: layer::PLAYER_BODY,
        }),
        health: None,
        knockback: Vec2::ZERO,
        anim: AnimState::default(),
        data: EntityData::Pickup(PickupData {
            kind,
            life: tuning::PICKUP_LIFE,
        }),
        alive: true,
    });
}
