//! Skeleton: front-shield walk, poke lunge, stagger (incl. perfect-block).

use content::audio::sfx::SfxId;

use crate::combat::tuning;
use crate::fx::FxKind;
use crate::math::{Dir4, Vec2};
use crate::world::entity::{
    layer, AnimState, Body, Entity, EntityData, EntityId, EntityKind, Health, SkeletonData,
    SkeletonState,
};
use crate::world::{World, WorldEvent};

use super::ai;

pub fn spawn(world: &mut World, pos: Vec2) -> EntityId {
    world.spawn(Entity {
        kind: EntityKind::Skeleton,
        pos,
        vel: Vec2::ZERO,
        facing: Dir4::Down,
        body: None,
        health: Some(Health {
            hp: tuning::SKELETON_HP,
            max: tuning::SKELETON_HP,
            iframes: 0,
            flash: 0,
        }),
        knockback: Vec2::ZERO,
        anim: AnimState::default(),
        data: EntityData::Skeleton(SkeletonData {
            spawn_telegraph: tuning::SPAWN_TELEGRAPH,
            state: SkeletonState::Walk,
            timer: 0,
            shield_up: true,
            stagger_len: 0,
        }),
        alive: true,
    })
}

/// Phase 3 boomerang / shared stagger entry point.
#[allow(dead_code)]
pub fn stagger(world: &mut World, id: EntityId) {
    stagger_for(world, id, tuning::SKELETON_PERFECT_STAGGER);
}

pub fn try_perfect_block_stagger(world: &mut World, id: EntityId) {
    let ok = matches!(
        world.get(id).map(|e| &e.data),
        Some(EntityData::Skeleton(d))
            if d.spawn_telegraph == 0
                && matches!(d.state, SkeletonState::Lunge | SkeletonState::PokeTelegraph)
    );
    if ok {
        stagger_for(world, id, tuning::SKELETON_PERFECT_STAGGER);
        world.push_event(WorldEvent::Sfx(SfxId::SkeletonRattle));
    }
}

fn stagger_for(world: &mut World, id: EntityId, ticks: u16) {
    if let Some(e) = world.get_mut(id) {
        if let EntityData::Skeleton(d) = &mut e.data {
            d.state = SkeletonState::Stagger;
            d.timer = 0;
            d.shield_up = false;
            d.stagger_len = ticks;
        }
    }
}

pub fn update_one(world: &mut World, id: EntityId) {
    let telegraph = {
        let Some(e) = world.get(id) else {
            return;
        };
        match &e.data {
            EntityData::Skeleton(d) => d.spawn_telegraph,
            _ => return,
        }
    };
    if telegraph > 0 {
        tick_spawn(world, id);
        return;
    }

    let Some(ppos) = ai::player_pos(world) else {
        return;
    };
    let (state, timer, epos, stagger_len) = {
        let Some(e) = world.get(id) else {
            return;
        };
        let EntityData::Skeleton(d) = &e.data else {
            return;
        };
        (d.state, d.timer, e.center(), d.stagger_len)
    };

    let dist = epos.sub(ppos).len();
    let mut next_state = state;
    let mut next_timer = timer.saturating_add(1);
    let mut next_stagger_len = stagger_len;
    let mut vel = Vec2::ZERO;
    let facing = Dir4::from_vec(ppos.sub(epos), Dir4::Down);
    let anim;
    let mut sfx = None;
    let mut contact = false;
    let mut poke = false;
    let mut shield_up;

    match state {
        SkeletonState::Walk => {
            anim = (next_timer / 12) % 2;
            vel = ai::steer_toward(epos, ppos, tuning::SKELETON_WALK);
            contact = true;
            shield_up = true;
            if dist <= tuning::SKELETON_POKE_RANGE && ai::has_los(world, epos, ppos) {
                next_state = SkeletonState::PokeTelegraph;
                next_timer = 0;
                sfx = Some(SfxId::SkeletonRattle);
            }
        }
        SkeletonState::PokeTelegraph => {
            anim = 2;
            shield_up = true;
            if next_timer >= tuning::SKELETON_POKE_TELE {
                next_state = SkeletonState::Lunge;
                next_timer = 0;
                poke = true;
            }
        }
        SkeletonState::Lunge => {
            anim = 3;
            vel = ai::steer_toward(epos, ppos, tuning::SKELETON_WALK * 2.2);
            shield_up = false;
            if next_timer >= tuning::SKELETON_LUNGE {
                next_state = SkeletonState::Stagger;
                next_timer = 0;
                next_stagger_len = tuning::SKELETON_POKE_STAGGER;
                shield_up = false;
            }
        }
        SkeletonState::Stagger => {
            anim = 4;
            shield_up = false;
            let dur = if next_stagger_len == 0 {
                tuning::SKELETON_POKE_STAGGER
            } else {
                next_stagger_len
            };
            if next_timer >= dur {
                next_state = SkeletonState::Walk;
                next_timer = 0;
                next_stagger_len = 0;
                shield_up = true;
            }
        }
    }

    {
        let Some(e) = world.get_mut(id) else {
            return;
        };
        e.vel = vel;
        e.facing = facing;
        e.anim.frame = anim;
        if let EntityData::Skeleton(d) = &mut e.data {
            d.state = next_state;
            d.timer = next_timer;
            d.shield_up = shield_up;
            d.stagger_len = next_stagger_len;
        }
    }
    ai::move_enemy(world, id);

    if let Some(id_sfx) = sfx {
        world.push_event(WorldEvent::Sfx(id_sfx));
    }

    if poke {
        if let Some(e) = world.get(id) {
            if ai::overlaps_player(world, e)
                || e.center().sub(ppos).len() < tuning::SKELETON_POKE_RANGE
            {
                let dir = ppos.sub(e.center()).normalize_or_zero();
                world.push_event(WorldEvent::DamagedPlayer {
                    amount: tuning::SKELETON_POKE,
                    dir,
                    source: Some(id),
                });
            }
        }
        let pos = world.get(id).map(|e| e.center()).unwrap_or(epos);
        world.push_event(WorldEvent::FxRequest(FxKind::Impact { pos }));
    }

    if contact {
        if let Some(e) = world.get(id) {
            if ai::overlaps_player(world, e) {
                let dir = ai::player_pos(world)
                    .map(|p| p.sub(e.center()).normalize_or_zero())
                    .unwrap_or(Vec2::ZERO);
                world.push_event(WorldEvent::DamagedPlayer {
                    amount: tuning::SKELETON_CONTACT,
                    dir,
                    source: Some(id),
                });
            }
        }
    }
}

fn tick_spawn(world: &mut World, id: EntityId) {
    let left = {
        let Some(e) = world.get_mut(id) else {
            return;
        };
        let EntityData::Skeleton(d) = &mut e.data else {
            return;
        };
        d.spawn_telegraph = d.spawn_telegraph.saturating_sub(1);
        d.spawn_telegraph
    };
    if left == tuning::SPAWN_TELEGRAPH - 1 {
        let pos = world.get(id).map(|e| e.center()).unwrap_or(Vec2::ZERO);
        world.push_event(WorldEvent::FxRequest(FxKind::Impact { pos }));
        world.push_event(WorldEvent::Sfx(SfxId::SpawnShimmer));
    }
    if left == 0 {
        if let Some(e) = world.get_mut(id) {
            e.body = Some(Body {
                half: Vec2::new(8.0, 8.0),
                solid: false,
                layer: layer::ENEMY_BODY,
                mask: layer::PLAYER_HIT,
            });
        }
    }
}
