//! Ironshell: front-armored shell-bash duo miniboss.

use content::audio::sfx::SfxId;

use crate::combat::tuning;
use crate::fx::FxKind;
use crate::math::{Dir4, Vec2};
use crate::world::entity::{
    layer, AnimState, Body, Entity, EntityData, EntityId, EntityKind, Health, IronshellData,
    IronshellState,
};
use crate::world::{World, WorldEvent};

use super::ai;

pub fn spawn(world: &mut World, pos: Vec2) -> EntityId {
    world.spawn(Entity {
        kind: EntityKind::Ironshell,
        pos,
        vel: Vec2::ZERO,
        facing: Dir4::Down,
        body: None,
        health: Some(Health {
            hp: tuning::IRONSHELL_HP,
            max: tuning::IRONSHELL_HP,
            iframes: 0,
            flash: 0,
        }),
        knockback: Vec2::ZERO,
        anim: AnimState::default(),
        data: EntityData::Ironshell(IronshellData {
            spawn_telegraph: tuning::SPAWN_TELEGRAPH,
            state: IronshellState::Advance,
            timer: 0,
            stun_ticks: 0,
            stagger_len: 0,
        }),
        alive: true,
    })
}

pub fn try_perfect_block_stagger(world: &mut World, id: EntityId) {
    let ok = matches!(
        world.get(id).map(|e| &e.data),
        Some(EntityData::Ironshell(d))
            if d.spawn_telegraph == 0
                && matches!(d.state, IronshellState::Bash | IronshellState::BashTelegraph)
    );
    if ok {
        stagger_for(world, id, tuning::IRONSHELL_STAGGER);
        world.push_event(WorldEvent::Sfx(SfxId::IronshellCrack));
    }
}

pub fn stagger(world: &mut World, id: EntityId) {
    stagger_for(world, id, tuning::IRONSHELL_STAGGER);
}

fn stagger_for(world: &mut World, id: EntityId, ticks: u16) {
    if let Some(e) = world.get_mut(id) {
        if let EntityData::Ironshell(d) = &mut e.data {
            d.state = IronshellState::Stagger;
            d.timer = 0;
            d.stagger_len = ticks;
            d.stun_ticks = 0;
        }
    }
}

/// True when the attack arrives from the armored front.
pub fn is_frontal_hit(facing: Dir4, attack_dir: Vec2) -> bool {
    facing.unit().dot(attack_dir.normalize_or_zero()) < -0.35
}

pub fn update_one(world: &mut World, id: EntityId) {
    let telegraph = {
        let Some(e) = world.get(id) else {
            return;
        };
        match &e.data {
            EntityData::Ironshell(d) => d.spawn_telegraph,
            _ => return,
        }
    };
    if telegraph > 0 {
        tick_spawn(world, id);
        return;
    }

    // Stun freeze (boomerang back-hit / generic stun).
    {
        let Some(e) = world.get_mut(id) else {
            return;
        };
        if let EntityData::Ironshell(d) = &mut e.data {
            if d.stun_ticks > 0 {
                d.stun_ticks = d.stun_ticks.saturating_sub(1);
                e.vel = Vec2::ZERO;
                e.anim.frame = 3;
                return;
            }
        }
    }

    let Some(ppos) = ai::player_pos(world) else {
        return;
    };
    let (state, timer, epos, stagger_len) = {
        let Some(e) = world.get(id) else {
            return;
        };
        let EntityData::Ironshell(d) = &e.data else {
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
    let mut bash_hit = false;

    match state {
        IronshellState::Advance => {
            anim = (next_timer / 14) % 2;
            vel = ai::steer_toward(epos, ppos, tuning::IRONSHELL_WALK);
            contact = true;
            if dist <= tuning::IRONSHELL_BASH_RANGE && ai::has_los(world, epos, ppos) {
                next_state = IronshellState::BashTelegraph;
                next_timer = 0;
                sfx = Some(SfxId::GuardClank);
            }
        }
        IronshellState::BashTelegraph => {
            anim = 2;
            if next_timer >= tuning::IRONSHELL_BASH_TELE {
                next_state = IronshellState::Bash;
                next_timer = 0;
                sfx = Some(SfxId::IronshellBash);
            }
        }
        IronshellState::Bash => {
            anim = 2;
            vel = ai::steer_toward(epos, ppos, tuning::IRONSHELL_BASH_SPEED);
            if next_timer >= tuning::IRONSHELL_BASH_TICKS {
                next_state = IronshellState::Recover;
                next_timer = 0;
            } else if next_timer == 1 {
                bash_hit = true;
            }
        }
        IronshellState::Recover => {
            anim = 0;
            if next_timer >= tuning::IRONSHELL_RECOVER {
                next_state = IronshellState::Advance;
                next_timer = 0;
            }
        }
        IronshellState::Stagger => {
            anim = 3;
            let dur = if next_stagger_len == 0 {
                tuning::IRONSHELL_STAGGER
            } else {
                next_stagger_len
            };
            if next_timer >= dur {
                next_state = IronshellState::Advance;
                next_timer = 0;
                next_stagger_len = 0;
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
        // Massive: damp knockback hard.
        e.knockback = e.knockback.scale(0.4);
        if let EntityData::Ironshell(d) = &mut e.data {
            d.state = next_state;
            d.timer = next_timer;
            d.stagger_len = next_stagger_len;
        }
    }
    ai::move_enemy(world, id);

    if let Some(id_sfx) = sfx {
        world.push_event(WorldEvent::Sfx(id_sfx));
    }

    if bash_hit {
        if let Some(e) = world.get(id) {
            if ai::overlaps_player(world, e)
                || e.center().sub(ppos).len() < tuning::IRONSHELL_BASH_RANGE
            {
                let dir = ppos.sub(e.center()).normalize_or_zero();
                world.push_event(WorldEvent::DamagedPlayer {
                    amount: tuning::IRONSHELL_BASH,
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
                    amount: tuning::IRONSHELL_CONTACT,
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
        let EntityData::Ironshell(d) = &mut e.data else {
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
                half: Vec2::new(10.0, 10.0),
                solid: false,
                layer: layer::ENEMY_BODY,
                mask: layer::PLAYER_HIT,
            });
        }
    }
}
