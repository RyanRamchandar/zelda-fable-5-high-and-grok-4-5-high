//! Octorok: ranged lob + hide. Rock is reflectable.

use content::audio::sfx::SfxId;
use content::maps::TILE_PX;

use crate::combat::tuning;
use crate::fx::FxKind;
use crate::math::{Dir4, Vec2};
use crate::world::entity::{
    layer, AnimState, Body, Entity, EntityData, EntityId, EntityKind, Health, OctorokData,
    OctorokState, RockData,
};
use crate::world::physics::move_entity;
use crate::world::{World, WorldEvent};

use super::ai;

pub fn spawn(world: &mut World, pos: Vec2) -> EntityId {
    world.spawn(Entity {
        kind: EntityKind::Octorok,
        pos,
        vel: Vec2::ZERO,
        facing: Dir4::Down,
        body: None,
        health: Some(Health {
            hp: tuning::OCTOROK_HP,
            max: tuning::OCTOROK_HP,
            iframes: 0,
            flash: 0,
        }),
        knockback: Vec2::ZERO,
        anim: AnimState::default(),
        data: EntityData::Octorok(OctorokData {
            spawn_telegraph: tuning::SPAWN_TELEGRAPH,
            state: OctorokState::Idle,
            timer: 0,
            cycle: 0,
            stun_ticks: 0,
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
            EntityData::Octorok(d) => d.spawn_telegraph,
            _ => return,
        }
    };
    if telegraph > 0 {
        tick_spawn(world, id);
        return;
    }
    {
        use crate::world::entity::EntityData as ED;
        let stunned = matches!(world.get(id).map(|e| &e.data), Some(ED::Octorok(d)) if d.stun_ticks > 0);
        if stunned {
            if let Some(e) = world.get_mut(id) {
                e.vel = crate::math::Vec2::ZERO;
                if let ED::Octorok(d) = &mut e.data {
                    d.stun_ticks = d.stun_ticks.saturating_sub(1);
                }
            }
            return;
        }
    }

    let Some(ppos) = ai::player_pos(world) else {
        return;
    };

    let (state, timer, cycle, epos) = {
        let Some(e) = world.get(id) else {
            return;
        };
        let EntityData::Octorok(d) = &e.data else {
            return;
        };
        (d.state, d.timer, d.cycle, e.center())
    };

    let mut next_state = state;
    let mut next_timer = timer.saturating_add(1);
    let mut next_cycle = cycle.saturating_add(1);
    let anim;
    let mut sfx = None;
    let mut fire = false;
    let facing = Dir4::from_vec(ppos.sub(epos), Dir4::Down);

    match state {
        OctorokState::Idle => {
            anim = (next_timer / 20) % 2;
            if next_cycle >= tuning::OCTOROK_CYCLE {
                next_state = OctorokState::SpitTelegraph;
                next_timer = 0;
                next_cycle = 0;
            }
        }
        OctorokState::SpitTelegraph => {
            anim = 4;
            if next_timer >= tuning::OCTOROK_SPIT_TELEGRAPH {
                next_state = OctorokState::Spit;
                next_timer = 0;
                fire = true;
                sfx = Some(SfxId::OctorokSpit);
            }
        }
        OctorokState::Spit => {
            anim = 5;
            if next_timer >= 8 {
                next_state = OctorokState::Hide;
                next_timer = 0;
                sfx = Some(SfxId::OctorokDuck);
            }
        }
        OctorokState::Hide => {
            anim = 2 + (next_timer / 30) % 2;
            if next_timer >= tuning::OCTOROK_HIDE {
                next_state = OctorokState::Idle;
                next_timer = 0;
                next_cycle = 0;
            }
        }
    }

    {
        let Some(e) = world.get_mut(id) else {
            return;
        };
        e.vel = Vec2::ZERO;
        e.facing = facing;
        e.anim.frame = anim;
        if let EntityData::Octorok(d) = &mut e.data {
            d.state = next_state;
            d.timer = next_timer;
            d.cycle = next_cycle;
        }
    }
    ai::move_enemy(world, id);

    if fire {
        spawn_rock(world, id, ppos);
    }
    if let Some(s) = sfx {
        world.push_event(WorldEvent::Sfx(s));
    }
}

fn spawn_rock(world: &mut World, owner: EntityId, target: Vec2) {
    let Some(e) = world.get(owner) else {
        return;
    };
    let origin = e.center();
    let dir = target.sub(origin).normalize_or_zero();
    let pos = origin.add(dir.scale(10.0)).sub(Vec2::new(4.0, 4.0));
    let swing_id = world.tick as u32 ^ (owner.index << 16);
    world.spawn(Entity {
        kind: EntityKind::OctorokRock,
        pos,
        vel: dir.scale(tuning::OCTOROK_ROCK_SPEED),
        facing: Dir4::from_vec(dir, Dir4::Down),
        body: Some(Body {
            half: Vec2::new(4.0, 4.0),
            solid: false,
            layer: layer::ENEMY_HIT,
            mask: layer::PLAYER_BODY,
        }),
        health: None,
        knockback: Vec2::ZERO,
        anim: AnimState::default(),
        data: EntityData::Rock(RockData {
            dir,
            damage: tuning::OCTOROK_ROCK_DAMAGE,
            from_player: false,
            hit: false,
            swing_id,
        }),
        alive: true,
    });
}

pub fn update_rocks(world: &mut World) {
    let ids: Vec<_> = world
        .alive_ids()
        .into_iter()
        .filter(|&id| world.get(id).map(|e| e.kind == EntityKind::OctorokRock).unwrap_or(false))
        .collect();

    let mut kill = Vec::new();
    for id in ids {
        let slot = id.index as usize;
        if slot >= world.arena.len() || world.arena[slot].gen != id.gen {
            continue;
        }
        let mut entity = match world.arena[slot].entity.take() {
            Some(e) => e,
            None => continue,
        };
        entity.anim.frame = ((world.tick / 4) % 2) as u16;
        let prev = entity.pos;
        move_entity(world, &mut entity);
        // wall hit: little movement
        if entity.pos.sub(prev).len() < 0.05 && entity.vel.len() > 0.1 {
            kill.push(id);
        }
        // out of map
        let max_x = world.map.width as f32 * TILE_PX;
        let max_y = world.map.height as f32 * TILE_PX;
        if entity.pos.x < 0.0
            || entity.pos.y < 0.0
            || entity.pos.x > max_x
            || entity.pos.y > max_y
        {
            kill.push(id);
        }
        world.arena[slot].entity = Some(entity);
    }
    for id in kill {
        world.despawn(id);
    }
}

fn tick_spawn(world: &mut World, id: EntityId) {
    let left = {
        let Some(e) = world.get_mut(id) else {
            return;
        };
        let EntityData::Octorok(d) = &mut e.data else {
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
