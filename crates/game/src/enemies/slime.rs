//! Slime: chase + lunge.

use content::audio::sfx::SfxId;

use crate::combat::tuning;
use crate::fx::FxKind;
use crate::math::{Dir4, Vec2};
use crate::world::entity::{
    layer, AnimState, Body, Entity, EntityData, EntityId, EntityKind, Health, SlimeData, SlimeState,
};
use crate::world::{World, WorldEvent};

use super::ai;

pub fn spawn(world: &mut World, pos: Vec2) -> EntityId {
    world.spawn(Entity {
        kind: EntityKind::Slime,
        pos,
        vel: Vec2::ZERO,
        facing: Dir4::Down,
        body: None,
        health: Some(Health {
            hp: tuning::SLIME_HP,
            max: tuning::SLIME_HP,
            iframes: 0,
            flash: 0,
        }),
        knockback: Vec2::ZERO,
        anim: AnimState::default(),
        data: EntityData::Slime(SlimeData {
            spawn_telegraph: tuning::SPAWN_TELEGRAPH,
            state: SlimeState::Idle,
            timer: 0,
            hop_phase: 0,
        }),
        alive: true,
    })
}

pub fn update_one(world: &mut World, id: EntityId) {
    let telegraph = {
        let Some(e) = world.get(id) else {
            return;
        };
        match &e.data {
            EntityData::Slime(d) => d.spawn_telegraph,
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

    let (state, timer, hop, epos, low_hp) = {
        let Some(e) = world.get(id) else {
            return;
        };
        let EntityData::Slime(d) = &e.data else {
            return;
        };
        let low = e.health.map(|h| h.hp <= 1).unwrap_or(false);
        (d.state, d.timer, d.hop_phase, e.center(), low)
    };

    let dist = epos.sub(ppos).len();
    let los = ai::has_los(world, epos, ppos);

    let mut next_state = state;
    let mut next_timer = timer.saturating_add(1);
    let mut next_hop = hop.saturating_add(1);
    let mut vel = Vec2::ZERO;
    let facing = Dir4::from_vec(ppos.sub(epos), Dir4::Down);
    let anim_frame;
    let mut sfx = None;
    let mut contact = false;

    match state {
        SlimeState::Idle => {
            anim_frame = (next_timer / 16) % 2;
            if next_timer % 40 < 20 {
                let ang = (world.tick as f32) * 0.05 + id.index as f32;
                vel = Vec2::new(ang.cos(), ang.sin()).scale(tuning::SLIME_WANDER);
            }
            if dist < tuning::SLIME_CHASE_RANGE && los {
                next_state = SlimeState::Chase;
                next_timer = 0;
                next_hop = 0;
            }
        }
        SlimeState::Chase => {
            let hop_cycle = tuning::SLIME_HOP_MOVE + tuning::SLIME_HOP_REST;
            let phase = next_hop % hop_cycle;
            if phase < tuning::SLIME_HOP_MOVE {
                vel = ai::steer_toward(epos, ppos, tuning::SLIME_CHASE);
                anim_frame = 2 + (phase / 6) % 2;
            } else {
                anim_frame = 0;
            }
            contact = true;
            if dist < tuning::SLIME_LUNGE_RANGE && los {
                next_state = SlimeState::LungeWindup;
                next_timer = 0;
                sfx = Some(SfxId::SlimeSquish);
            } else if dist > tuning::SLIME_CHASE_RANGE * 1.25 || !los {
                next_state = SlimeState::Idle;
                next_timer = 0;
            }
        }
        SlimeState::LungeWindup => {
            anim_frame = 4;
            if next_timer >= tuning::SLIME_WINDUP {
                next_state = SlimeState::Lunge;
                next_timer = 0;
                sfx = Some(SfxId::SlimeLunge);
            }
        }
        SlimeState::Lunge => {
            vel = ai::steer_toward(epos, ppos, tuning::SLIME_LUNGE_SPEED);
            // lock aim from windup end — use current player
            anim_frame = 3;
            contact = true;
            if next_timer >= tuning::SLIME_LUNGE_TICKS {
                next_state = SlimeState::Recover;
                next_timer = 0;
            }
        }
        SlimeState::Recover => {
            anim_frame = 1;
            if next_timer >= tuning::SLIME_RECOVER {
                next_state = SlimeState::Chase;
                next_timer = 0;
                next_hop = 0;
            }
        }
    }

    {
        let Some(e) = world.get_mut(id) else {
            return;
        };
        e.vel = vel;
        e.facing = facing;
        e.anim.frame = anim_frame;
        e.anim.sheet = if low_hp { 1 } else { 0 };
        if let EntityData::Slime(d) = &mut e.data {
            d.state = next_state;
            d.timer = next_timer;
            d.hop_phase = next_hop;
        }
    }

    ai::move_enemy(world, id);

    if let Some(id_sfx) = sfx {
        world.push_event(WorldEvent::Sfx(id_sfx));
    }

    if contact {
        if let Some(e) = world.get(id) {
            if ai::overlaps_player(world, e) {
                let dir = ai::player_pos(world)
                    .map(|p| p.sub(e.center()).normalize_or_zero())
                    .unwrap_or(Vec2::ZERO);
                world.push_event(WorldEvent::DamagedPlayer {
                    amount: tuning::SLIME_CONTACT,
                    dir,
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
        let EntityData::Slime(d) = &mut e.data else {
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
