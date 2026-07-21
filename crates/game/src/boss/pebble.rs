//! Pebble crawlers — phase-2+ Warden minions (slime-shaped chase/lunge).

use content::audio::sfx::SfxId;

use crate::combat::tuning;
use crate::enemies::ai;
use crate::fx::FxKind;
use crate::math::{Dir4, Vec2};
use crate::world::entity::{
    layer, AnimState, Body, Entity, EntityData, EntityId, EntityKind, Health, PebbleData, SlimeState,
};
use crate::world::{World, WorldEvent};

pub fn spawn(world: &mut World, pos: Vec2) -> EntityId {
    world.spawn(Entity {
        kind: EntityKind::PebbleCrawler,
        pos,
        vel: Vec2::ZERO,
        facing: Dir4::Down,
        body: Some(Body {
            half: Vec2::new(6.0, 6.0),
            solid: false,
            layer: layer::ENEMY_BODY,
            mask: layer::PLAYER_HIT,
        }),
        health: Some(Health {
            hp: tuning::PEBBLE_HP,
            max: tuning::PEBBLE_HP,
            iframes: 0,
            flash: 0,
        }),
        knockback: Vec2::ZERO,
        anim: AnimState::default(),
        data: EntityData::PebbleCrawler(PebbleData {
            spawn_telegraph: 20,
            state: SlimeState::Chase,
            timer: 0,
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
            EntityData::PebbleCrawler(d) => d.spawn_telegraph,
            _ => return,
        }
    };
    if telegraph > 0 {
        if let Some(e) = world.get_mut(id) {
            if let EntityData::PebbleCrawler(d) = &mut e.data {
                d.spawn_telegraph = d.spawn_telegraph.saturating_sub(1);
            }
        }
        return;
    }

    let Some(ppos) = ai::player_pos(world) else {
        return;
    };
    let (state, timer, epos) = {
        let Some(e) = world.get(id) else {
            return;
        };
        let EntityData::PebbleCrawler(d) = &e.data else {
            return;
        };
        (d.state, d.timer, e.center())
    };

    let dist = epos.sub(ppos).len();
    let mut next = state;
    let mut t = timer.saturating_add(1);
    let mut vel = Vec2::ZERO;
    let facing = Dir4::from_vec(ppos.sub(epos), Dir4::Down);
    let mut contact = false;
    let mut sfx = None;

    match state {
        SlimeState::Chase | SlimeState::Idle => {
            vel = ai::steer_toward(epos, ppos, tuning::PEBBLE_CHASE);
            contact = true;
            if dist < 28.0 {
                next = SlimeState::LungeWindup;
                t = 0;
                sfx = Some(SfxId::PebbleSkitter);
            }
        }
        SlimeState::LungeWindup => {
            if t >= 18 {
                next = SlimeState::Lunge;
                t = 0;
            }
        }
        SlimeState::Lunge => {
            vel = ai::steer_toward(epos, ppos, tuning::PEBBLE_LUNGE);
            contact = true;
            if t >= 8 {
                next = SlimeState::Recover;
                t = 0;
            }
        }
        SlimeState::Recover => {
            if t >= 20 {
                next = SlimeState::Chase;
                t = 0;
            }
        }
    }

    {
        let Some(e) = world.get_mut(id) else {
            return;
        };
        e.vel = vel;
        e.facing = facing;
        e.anim.frame = (t / 8) % 2;
        if let EntityData::PebbleCrawler(d) = &mut e.data {
            d.state = next;
            d.timer = t;
        }
    }
    ai::move_enemy(world, id);

    if let Some(s) = sfx {
        world.push_event(WorldEvent::Sfx(s));
    }
    if contact {
        let hit = world
            .get(id)
            .map(|e| (ai::overlaps_player(world, e), e.center()))
            .unwrap_or((false, Vec2::ZERO));
        if hit.0 {
            let dir = ppos.sub(hit.1).normalize_or_zero();
            world.push_event(WorldEvent::DamagedPlayer {
                amount: tuning::PEBBLE_CONTACT,
                dir,
                source: Some(id),
            });
            world.push_event(WorldEvent::FxRequest(FxKind::Impact { pos: hit.1 }));
        }
    }
}
