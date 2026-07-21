//! Raider spear (melee poke + guard) and torch (arc lob + flame zone).

use content::audio::sfx::SfxId;

use crate::combat::tuning;
use crate::fx::FxKind;
use crate::math::{Dir4, Vec2};
use crate::world::entity::{
    layer, AnimState, Body, Entity, EntityData, EntityId, EntityKind, Health, RaiderSpearData,
    RaiderSpearState, RaiderTorchData, RaiderTorchState, TorchFlameData, TorchProjData,
};
use crate::world::{World, WorldEvent};

use super::ai;

pub fn spawn_spear(world: &mut World, pos: Vec2) -> EntityId {
    world.spawn(Entity {
        kind: EntityKind::RaiderSpear,
        pos,
        vel: Vec2::ZERO,
        facing: Dir4::Down,
        body: None,
        health: Some(Health {
            hp: tuning::RAIDER_SPEAR_HP,
            max: tuning::RAIDER_SPEAR_HP,
            iframes: 0,
            flash: 0,
        }),
        knockback: Vec2::ZERO,
        anim: AnimState::default(),
        data: EntityData::RaiderSpear(RaiderSpearData {
            spawn_telegraph: tuning::SPAWN_TELEGRAPH,
            state: RaiderSpearState::Idle,
            timer: 0,
            patrol_phase: 0.0,
        }),
        alive: true,
    })
}

pub fn spawn_torch(world: &mut World, pos: Vec2) -> EntityId {
    world.spawn(Entity {
        kind: EntityKind::RaiderTorch,
        pos,
        vel: Vec2::ZERO,
        facing: Dir4::Down,
        body: None,
        health: Some(Health {
            hp: tuning::RAIDER_TORCH_HP,
            max: tuning::RAIDER_TORCH_HP,
            iframes: 0,
            flash: 0,
        }),
        knockback: Vec2::ZERO,
        anim: AnimState::default(),
        data: EntityData::RaiderTorch(RaiderTorchData {
            spawn_telegraph: tuning::SPAWN_TELEGRAPH,
            state: RaiderTorchState::Idle,
            timer: 0,
        }),
        alive: true,
    })
}

pub fn update_spear(world: &mut World, id: EntityId) {
    let telegraph = {
        let Some(e) = world.get(id) else {
            return;
        };
        match &e.data {
            EntityData::RaiderSpear(d) => d.spawn_telegraph,
            _ => return,
        }
    };
    if telegraph > 0 {
        tick_spawn(world, id, true);
        return;
    }

    let Some(ppos) = ai::player_pos(world) else {
        return;
    };
    let (state, timer, patrol, epos) = {
        let Some(e) = world.get(id) else {
            return;
        };
        let EntityData::RaiderSpear(d) = &e.data else {
            return;
        };
        (d.state, d.timer, d.patrol_phase, e.center())
    };

    let dist = epos.sub(ppos).len();
    let mut next_state = state;
    let mut next_timer = timer.saturating_add(1);
    let next_patrol = patrol + 0.04;
    let mut vel = Vec2::ZERO;
    let facing = Dir4::from_vec(ppos.sub(epos), Dir4::Down);
    let anim;
    let mut sfx = None;
    let mut contact = false;
    let mut poke = false;

    match state {
        RaiderSpearState::Idle => {
            anim = (next_timer / 12) % 2;
            let ang = next_patrol + id.index as f32;
            vel = Vec2::new(ang.cos(), ang.sin()).scale(tuning::RAIDER_SPEAR_WALK * 0.6);
            if dist < 96.0 && ai::has_los(world, epos, ppos) {
                next_state = RaiderSpearState::Approach;
                next_timer = 0;
            }
        }
        RaiderSpearState::Approach => {
            anim = 1 + (next_timer / 10) % 2;
            vel = ai::steer_toward(epos, ppos, tuning::RAIDER_SPEAR_WALK);
            contact = true;
            if dist <= tuning::RAIDER_SPEAR_APPROACH {
                next_state = RaiderSpearState::PokeTelegraph;
                next_timer = 0;
                sfx = Some(SfxId::RaiderPoke);
            } else if dist > 120.0 {
                next_state = RaiderSpearState::Idle;
                next_timer = 0;
            }
        }
        RaiderSpearState::PokeTelegraph => {
            anim = 3;
            if next_timer >= tuning::RAIDER_SPEAR_POKE_TELE {
                next_state = RaiderSpearState::Thrust;
                next_timer = 0;
                poke = true;
            }
        }
        RaiderSpearState::Thrust => {
            anim = 4;
            if next_timer >= tuning::RAIDER_SPEAR_THRUST {
                next_state = RaiderSpearState::Guard;
                next_timer = 0;
            }
        }
        RaiderSpearState::Guard => {
            anim = 5;
            if next_timer >= tuning::RAIDER_SPEAR_GUARD {
                next_state = RaiderSpearState::Approach;
                next_timer = 0;
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
        if let EntityData::RaiderSpear(d) = &mut e.data {
            d.state = next_state;
            d.timer = next_timer;
            d.patrol_phase = next_patrol;
        }
    }
    ai::move_enemy(world, id);

    if let Some(id_sfx) = sfx {
        world.push_event(WorldEvent::Sfx(id_sfx));
    }

    if poke {
        let tip = {
            let Some(e) = world.get(id) else {
                return;
            };
            e.center()
                .add(e.facing.unit().scale(tuning::RAIDER_SPEAR_REACH))
        };
        if let Some(p) = world.get(world.player_id) {
            if let Some(pb) = p.body {
                let dx = (p.center().x - tip.x).abs();
                let dy = (p.center().y - tip.y).abs();
                if dx < pb.half.x + 6.0 && dy < pb.half.y + 6.0 {
                    let dir = p
                        .center()
                        .sub(tip)
                        .normalize_or_zero();
                    world.push_event(WorldEvent::DamagedPlayer {
                        amount: tuning::RAIDER_SPEAR_POKE,
                        dir,
                        source: Some(id),
                    });
                }
            }
        }
        world.push_event(WorldEvent::FxRequest(FxKind::Impact { pos: tip }));
    }

    if contact {
        if let Some(e) = world.get(id) {
            if ai::overlaps_player(world, e) {
                let dir = ai::player_pos(world)
                    .map(|p| p.sub(e.center()).normalize_or_zero())
                    .unwrap_or(Vec2::ZERO);
                world.push_event(WorldEvent::DamagedPlayer {
                    amount: tuning::RAIDER_SPEAR_CONTACT,
                    dir,
                    source: Some(id),
                });
            }
        }
    }
}

pub fn update_torch(world: &mut World, id: EntityId) {
    let telegraph = {
        let Some(e) = world.get(id) else {
            return;
        };
        match &e.data {
            EntityData::RaiderTorch(d) => d.spawn_telegraph,
            _ => return,
        }
    };
    if telegraph > 0 {
        tick_spawn(world, id, false);
        return;
    }

    let Some(ppos) = ai::player_pos(world) else {
        return;
    };
    let (state, timer, epos) = {
        let Some(e) = world.get(id) else {
            return;
        };
        let EntityData::RaiderTorch(d) = &e.data else {
            return;
        };
        (d.state, d.timer, e.center())
    };

    let dist = epos.sub(ppos).len();
    let mut next_state = state;
    let mut next_timer = timer.saturating_add(1);
    let mut vel = Vec2::ZERO;
    let facing = Dir4::from_vec(ppos.sub(epos), Dir4::Down);
    let anim;
    let mut sfx = None;
    let mut lob = false;
    let mut contact = false;

    match state {
        RaiderTorchState::Idle => {
            anim = (next_timer / 14) % 2;
            if dist < tuning::RAIDER_TORCH_KEEP_MIN {
                vel = ai::steer_toward(epos, ppos, -tuning::RAIDER_TORCH_WALK);
            } else if dist > tuning::RAIDER_TORCH_KEEP_MAX {
                vel = ai::steer_toward(epos, ppos, tuning::RAIDER_TORCH_WALK);
            } else {
                let side = Vec2::new(-(ppos.y - epos.y), ppos.x - epos.x).normalize_or_zero();
                vel = side.scale(tuning::RAIDER_TORCH_WALK * 0.4);
            }
            contact = dist < 18.0;
            let throw_band =
                tuning::RAIDER_TORCH_KEEP_MIN - 8.0..=tuning::RAIDER_TORCH_KEEP_MAX + 16.0;
            if throw_band.contains(&dist) && ai::has_los(world, epos, ppos) && next_timer >= 40 {
                next_state = RaiderTorchState::ThrowTelegraph;
                next_timer = 0;
            }
        }
        RaiderTorchState::ThrowTelegraph => {
            anim = 2;
            if next_timer >= tuning::RAIDER_TORCH_TELE {
                next_state = RaiderTorchState::Cooldown;
                next_timer = 0;
                lob = true;
                sfx = Some(SfxId::TorchThrow);
            }
        }
        RaiderTorchState::Cooldown => {
            anim = 0;
            if dist < tuning::RAIDER_TORCH_KEEP_MIN {
                vel = ai::steer_toward(epos, ppos, -tuning::RAIDER_TORCH_WALK);
            }
            if next_timer >= tuning::RAIDER_TORCH_COOLDOWN {
                next_state = RaiderTorchState::Idle;
                next_timer = 0;
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
        if let EntityData::RaiderTorch(d) = &mut e.data {
            d.state = next_state;
            d.timer = next_timer;
        }
    }
    ai::move_enemy(world, id);

    if let Some(id_sfx) = sfx {
        world.push_event(WorldEvent::Sfx(id_sfx));
    }

    if lob {
        let origin = world.get(id).map(|e| e.center()).unwrap_or(epos);
        let dir = ppos.sub(origin).normalize_or_zero();
        spawn_torch_proj(world, origin, dir);
    }

    if contact {
        if let Some(e) = world.get(id) {
            if ai::overlaps_player(world, e) {
                let dir = ai::player_pos(world)
                    .map(|p| p.sub(e.center()).normalize_or_zero())
                    .unwrap_or(Vec2::ZERO);
                world.push_event(WorldEvent::DamagedPlayer {
                    amount: tuning::RAIDER_TORCH_CONTACT,
                    dir,
                    source: Some(id),
                });
            }
        }
    }
}

pub fn update_projectiles(world: &mut World) {
    let ids: Vec<_> = world
        .alive_ids()
        .into_iter()
        .filter(|&id| {
            world
                .get(id)
                .map(|e| matches!(e.kind, EntityKind::TorchProj | EntityKind::TorchFlame))
                .unwrap_or(false)
        })
        .collect();

    let mut land = Vec::new();
    let mut kill = Vec::new();

    for id in ids {
        let kind = world.get(id).map(|e| e.kind);
        match kind {
            Some(EntityKind::TorchProj) => {
                let (done, pos) = {
                    let Some(e) = world.get_mut(id) else {
                        continue;
                    };
                    let EntityData::TorchProj(d) = &mut e.data else {
                        continue;
                    };
                    d.age = d.age.saturating_add(1);
                    d.life = d.life.saturating_sub(1);
                    let t = d.age as f32 / tuning::TORCH_PROJ_LIFE as f32;
                    let arc = (t * std::f32::consts::PI).sin() * 18.0;
                    e.pos = e.pos.add(d.dir.scale(tuning::TORCH_PROJ_SPEED));
                    e.pos.y -= arc * 0.15;
                    e.anim.frame = (d.age / 4) % 2;
                    (d.life == 0 || d.hit, e.center())
                };
                if done {
                    land.push(pos);
                    kill.push(id);
                }
            }
            Some(EntityKind::TorchFlame) => {
                let (dead, hurt, pos) = {
                    let Some(e) = world.get_mut(id) else {
                        continue;
                    };
                    let EntityData::TorchFlame(d) = &mut e.data else {
                        continue;
                    };
                    d.life = d.life.saturating_sub(1);
                    d.tick = d.tick.saturating_add(1);
                    e.anim.frame = (d.tick / 6) % 2;
                    let hurt = d.tick > 0 && d.tick.is_multiple_of(tuning::TORCH_FLAME_TICK);
                    (d.life == 0, hurt, e.center())
                };
                if dead {
                    kill.push(id);
                } else if hurt {
                    if let Some(p) = world.get(world.player_id) {
                        if p.center().sub(pos).len() < 18.0 {
                            let dir = p.center().sub(pos).normalize_or_zero();
                            world.push_event(WorldEvent::DamagedPlayer {
                                amount: tuning::TORCH_FLAME_DAMAGE,
                                dir,
                                source: None,
                            });
                        }
                    }
                }
            }
            _ => {}
        }
    }

    for id in kill {
        world.despawn(id);
    }
    for pos in land {
        spawn_flame(world, pos);
        world.push_event(WorldEvent::Sfx(SfxId::FlameBurst));
        world.push_event(WorldEvent::FxRequest(FxKind::Impact { pos }));
    }
}

fn spawn_torch_proj(world: &mut World, pos: Vec2, dir: Vec2) {
    world.spawn(Entity {
        kind: EntityKind::TorchProj,
        pos: Vec2::new(pos.x - 4.0, pos.y - 4.0),
        vel: dir.scale(tuning::TORCH_PROJ_SPEED),
        facing: Dir4::Down,
        body: Some(Body {
            half: Vec2::new(4.0, 4.0),
            solid: false,
            layer: layer::ENEMY_HIT,
            mask: layer::PLAYER_BODY,
        }),
        health: None,
        knockback: Vec2::ZERO,
        anim: AnimState::default(),
        data: EntityData::TorchProj(TorchProjData {
            dir,
            life: tuning::TORCH_PROJ_LIFE,
            age: 0,
            hit: false,
        }),
        alive: true,
    });
}

fn spawn_flame(world: &mut World, pos: Vec2) {
    world.spawn(Entity {
        kind: EntityKind::TorchFlame,
        pos: Vec2::new(pos.x - 8.0, pos.y - 8.0),
        vel: Vec2::ZERO,
        facing: Dir4::Down,
        body: None,
        health: None,
        knockback: Vec2::ZERO,
        anim: AnimState::default(),
        data: EntityData::TorchFlame(TorchFlameData {
            life: tuning::TORCH_FLAME_LIFE,
            tick: 0,
        }),
        alive: true,
    });
}

fn tick_spawn(world: &mut World, id: EntityId, spear: bool) {
    let left = {
        let Some(e) = world.get_mut(id) else {
            return;
        };
        match &mut e.data {
            EntityData::RaiderSpear(d) => {
                d.spawn_telegraph = d.spawn_telegraph.saturating_sub(1);
                d.spawn_telegraph
            }
            EntityData::RaiderTorch(d) => {
                d.spawn_telegraph = d.spawn_telegraph.saturating_sub(1);
                d.spawn_telegraph
            }
            _ => return,
        }
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
            let _ = spear;
        }
    }
}
