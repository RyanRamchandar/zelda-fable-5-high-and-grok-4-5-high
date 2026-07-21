//! Bat: sine swoop harasser.

use content::audio::sfx::SfxId;

use crate::combat::tuning;
use crate::fx::FxKind;
use crate::math::{Dir4, Vec2};
use crate::world::entity::{
    layer, AnimState, BatData, BatState, Body, Entity, EntityData, EntityId, EntityKind, Health,
};
use crate::world::{World, WorldEvent};

use super::ai;

pub fn spawn(world: &mut World, pos: Vec2) -> EntityId {
    let hover_phase = world.rng.f32() * std::f32::consts::TAU;
    world.spawn(Entity {
        kind: EntityKind::Bat,
        pos,
        vel: Vec2::ZERO,
        facing: Dir4::Down,
        body: None,
        health: Some(Health {
            hp: tuning::BAT_HP,
            max: tuning::BAT_HP,
            iframes: 0,
            flash: 0,
        }),
        knockback: Vec2::ZERO,
        anim: AnimState::default(),
        data: EntityData::Bat(BatData {
            spawn_telegraph: tuning::SPAWN_TELEGRAPH,
            state: BatState::Hover,
            timer: 0,
            hover_phase,
            swoop_origin: pos,
            swoop_target: pos,
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
            EntityData::Bat(d) => d.spawn_telegraph,
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

    let (state, timer, phase, origin, target, epos) = {
        let Some(e) = world.get(id) else {
            return;
        };
        let EntityData::Bat(d) = &e.data else {
            return;
        };
        (
            d.state,
            d.timer,
            d.hover_phase,
            d.swoop_origin,
            d.swoop_target,
            e.center(),
        )
    };

    let mut next_state = state;
    let mut next_timer = timer.saturating_add(1);
    let next_phase = phase + 0.12;
    let mut next_origin = origin;
    let mut next_target = target;
    let mut vel;
    let mut anim = (next_timer / 6) % 3;
    let mut sfx = None;
    let mut contact = false;

    match state {
        BatState::Hover => {
            let ox = next_phase.sin() * 18.0;
            let oy = (next_phase * 1.7).cos() * 10.0;
            let hover = Vec2::new(epos.x + ox * 0.02, epos.y - 40.0 + oy * 0.02);
            // drift near player but stay elevated
            let desired = Vec2::new(ppos.x + ox, ppos.y - 48.0 + oy);
            vel = ai::steer_toward(epos, desired, tuning::BAT_HOVER_DRIFT);
            let _ = hover;
            if next_timer >= tuning::BAT_SWOOP_PERIOD {
                next_state = BatState::SwoopTelegraph;
                next_timer = 0;
                next_origin = epos;
                next_target = ppos;
                sfx = Some(SfxId::BatSqueak);
            }
        }
        BatState::SwoopTelegraph => {
            anim = 0;
            vel = Vec2::new((next_phase * 3.0).sin() * 0.4, 0.0);
            if next_timer >= tuning::BAT_TELEGRAPH {
                next_state = BatState::Swoop;
                next_timer = 0;
                sfx = Some(SfxId::BatSwoop);
            }
        }
        BatState::Swoop => {
            anim = 3;
            contact = true;
            let t = (next_timer as f32 / tuning::BAT_SWOOP_TICKS as f32).clamp(0.0, 1.0);
            let along = next_origin.add(next_target.sub(next_origin).scale(t));
            let sine = (t * std::f32::consts::PI).sin() * 12.0;
            let side = next_target.sub(next_origin).normalize_or_zero();
            let perp = Vec2::new(-side.y, side.x).scale(sine);
            let dest = along.add(perp);
            vel = dest.sub(epos).scale(0.85).normalize_or_zero().scale(tuning::BAT_SWOOP_SPEED);
            // ensure progress
            if vel.len() < 0.1 {
                vel = next_target.sub(next_origin).normalize_or_zero().scale(tuning::BAT_SWOOP_SPEED);
            }
            if next_timer >= tuning::BAT_SWOOP_TICKS {
                next_state = BatState::Climb;
                next_timer = 0;
            }
        }
        BatState::Climb => {
            anim = (next_timer / 5) % 3;
            vel = Vec2::new(0.0, -1.2);
            if next_timer >= tuning::BAT_CLIMB_TICKS {
                next_state = BatState::Hover;
                next_timer = 0;
            }
        }
    }

    {
        let Some(e) = world.get_mut(id) else {
            return;
        };
        e.vel = vel;
        e.facing = Dir4::from_vec(vel, e.facing);
        e.anim.frame = anim;
        if let EntityData::Bat(d) = &mut e.data {
            d.state = next_state;
            d.timer = next_timer;
            d.hover_phase = next_phase;
            d.swoop_origin = next_origin;
            d.swoop_target = next_target;
        }
    }

    ai::fly_enemy(world, id);

    if let Some(s) = sfx {
        world.push_event(WorldEvent::Sfx(s));
    }

    if contact {
        if let Some(e) = world.get(id) {
            if ai::overlaps_player(world, e) {
                let dir = ppos.sub(e.center()).normalize_or_zero();
                world.push_event(WorldEvent::DamagedPlayer {
                    amount: tuning::BAT_CONTACT,
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
        let EntityData::Bat(d) = &mut e.data else {
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
                half: Vec2::new(7.0, 7.0),
                solid: false,
                layer: layer::ENEMY_BODY,
                mask: layer::PLAYER_HIT,
            });
        }
    }
}
