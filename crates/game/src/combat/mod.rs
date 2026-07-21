//! Damage pipeline, energy, style.
//!
//! # 1B hit contract
//! Enemies and projectiles hurt the player by either:
//! 1. Pushing `WorldEvent::DamagedPlayer { amount, dir }` (preferred for contact), or
//! 2. Spawning entities on `layer::ENEMY_HIT` that overlap the player body —
//!    [`resolve_hits`] converts those overlaps into `DamagedPlayer`.
//!
//! Player attacks register [`crate::world::ActiveAttack`] entries each tick while
//! a swing/spin is active; [`resolve_hits`] overlaps them against `ENEMY_BODY`,
//! dedupes via `(swing_id, target_index)`, then pushes `AttackHit` for routing.

pub mod damage;
pub mod energy;
pub mod style;
pub mod tuning;

use crate::math::Vec2;
use crate::world::entity::{layer, EntityData, EntityKind};
use crate::world::physics::{aabb_overlap, circle_hits_entity};
use crate::world::{AttackKind, World, WorldEvent};

use damage::{apply_attack_hit, apply_player_damage};

pub fn resolve_hits(world: &mut World) {
    // Player attack hitboxes → enemies
    let attacks: Vec<_> = world.active_attacks.clone();
    for atk in attacks {
        let targets = if atk.use_radius {
            let ids = world.alive_ids();
            ids.into_iter()
                .filter(|&id| {
                    world
                        .get(id)
                        .map(|e| {
                            e.body
                                .map(|b| b.layer & layer::ENEMY_BODY != 0)
                                .unwrap_or(false)
                                && circle_hits_entity(atk.center, atk.radius, e)
                        })
                        .unwrap_or(false)
                })
                .collect::<Vec<_>>()
        } else {
            crate::world::physics::query_aabb(world, atk.center, atk.half, layer::ENEMY_BODY)
        };

        for tid in targets {
            if world.already_hit(atk.swing_id, tid.index) {
                continue;
            }
            world.mark_hit(atk.swing_id, tid.index);
            world.push_event(WorldEvent::AttackHit {
                target: tid,
                attack: atk.kind,
                dir: atk.dir,
                pos: atk.center,
                damage: atk.damage,
                knockback: atk.knockback,
                source: atk.owner,
            });
        }
    }
    world.active_attacks.clear();

    // Enemy/debug projectiles → player (and player beams → enemies)
    let ids = world.alive_ids();
    let pid = world.player_id;
    for id in ids {
        if id == pid {
            continue;
        }
        let (kind, from_player, center, dir, damage, knockback, swing_id, hit) = {
            let Some(e) = world.get(id) else {
                continue;
            };
            match e.kind {
                EntityKind::SwordBeam | EntityKind::DebugShot => {
                    let EntityData::Beam(b) = &e.data else {
                        continue;
                    };
                    (
                        e.kind,
                        b.from_player,
                        e.center(),
                        b.dir,
                        b.damage,
                        b.knockback,
                        b.swing_id,
                        b.hit,
                    )
                }
                EntityKind::Player
                | EntityKind::Dummy
                | EntityKind::Pickup
                | EntityKind::FairyFountain => continue,
            }
        };
        if hit {
            continue;
        }

        if from_player {
            let targets =
                crate::world::physics::query_aabb(world, center, Vec2::new(4.0, 4.0), layer::ENEMY_BODY);
            for tid in targets {
                if world.already_hit(swing_id, tid.index) {
                    continue;
                }
                world.mark_hit(swing_id, tid.index);
                if let Some(e) = world.get_mut(id) {
                    if let EntityData::Beam(b) = &mut e.data {
                        b.hit = true;
                    }
                }
                world.push_event(WorldEvent::AttackHit {
                    target: tid,
                    attack: AttackKind::Beam,
                    dir,
                    pos: center,
                    damage,
                    knockback,
                    source: id,
                });
                world.despawn(id);
                break;
            }
        } else {
            // Hostile projectile vs player
            let hit_player = world
                .get(pid)
                .zip(world.get(id))
                .map(|(p, shot)| {
                    let Some(pb) = p.body else {
                        return false;
                    };
                    aabb_overlap(p.center(), pb.half, shot)
                })
                .unwrap_or(false);
            if hit_player {
                if let Some(e) = world.get_mut(id) {
                    if let EntityData::Beam(b) = &mut e.data {
                        b.hit = true;
                    }
                }
                let pcenter = world.get(pid).map(|p| p.center()).unwrap_or(Vec2::ZERO);
                let knock_dir = pcenter.sub(center).normalize_or_zero();
                world.push_event(WorldEvent::DamagedPlayer {
                    amount: damage.ceil() as i32,
                    dir: knock_dir,
                });
                if kind == EntityKind::DebugShot || kind == EntityKind::SwordBeam {
                    world.despawn(id);
                }
            }
        }
    }
}

pub fn route_combat_events(world: &mut World, events: Vec<WorldEvent>) -> Vec<WorldEvent> {
    let mut rest = Vec::new();
    for ev in events {
        match ev {
            WorldEvent::AttackHit {
                target,
                attack,
                dir,
                pos,
                damage,
                knockback,
                source: _,
            } => {
                apply_attack_hit(world, target, attack, dir, pos, damage, knockback);
            }
            WorldEvent::DamagedPlayer { amount, dir } => {
                apply_player_damage(world, amount, dir);
            }
            other => rest.push(other),
        }
    }
    rest
}

pub fn tick_dummies(world: &mut World) {
    let ids = world.alive_ids();
    for id in ids {
        let (home, dead) = {
            let Some(e) = world.get(id) else {
                continue;
            };
            match &e.data {
                EntityData::Dummy(d) => (d.home, d.dead_ticks),
                _ => continue,
            }
        };
        let Some(ticks) = dead else {
            continue;
        };
        let next = ticks + 1;
        if next >= tuning::DUMMY_RESPAWN {
            if let Some(e) = world.get_mut(id) {
                e.pos = home;
                e.knockback = Vec2::ZERO;
                e.health = Some(crate::world::entity::Health {
                    hp: tuning::DUMMY_HP,
                    max: tuning::DUMMY_HP,
                    iframes: 0,
                    flash: 0,
                });
                e.body = Some(crate::world::entity::Body {
                    half: Vec2::new(8.0, 8.0),
                    solid: false,
                    layer: layer::ENEMY_BODY,
                    mask: layer::PLAYER_HIT,
                });
                if let EntityData::Dummy(d) = &mut e.data {
                    d.dead_ticks = None;
                }
            }
        } else if let Some(e) = world.get_mut(id) {
            if let EntityData::Dummy(d) = &mut e.data {
                d.dead_ticks = Some(next);
            }
        }
    }
}
