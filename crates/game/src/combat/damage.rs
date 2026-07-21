//! Apply resolved hits to entities.

use crate::combat::style::{self, StyleVerb};
use crate::combat::tuning;
use crate::fx::FxKind;
use crate::math::Vec2;
use crate::world::entity::{layer, EntityData, EntityId, EntityKind, Health};
use crate::world::{AttackKind, World, WorldEvent};
use content::audio::sfx::SfxId;

pub fn apply_attack_hit(
    world: &mut World,
    target: EntityId,
    attack: AttackKind,
    dir: Vec2,
    pos: Vec2,
    damage: f32,
    knockback: f32,
) {
    {
        let Some(entity) = world.get(target) else {
            return;
        };
        if entity.kind != EntityKind::Dummy {
            return;
        }
        if let EntityData::Dummy(d) = &entity.data {
            if d.dead_ticks.is_some() {
                return;
            }
        }
        if entity.health.map(|h| h.iframes > 0).unwrap_or(true) {
            return;
        }
    }

    let (killed, kind, death_pos, heavy) = {
        let Some(entity) = world.get_mut(target) else {
            return;
        };
        let Some(h) = entity.health.as_mut() else {
            return;
        };
        h.hp = (h.hp as f32 - damage).ceil().max(0.0) as i32;
        h.flash = tuning::FLASH_TICKS;
        entity.knockback = dir.normalize_or_zero().scale(knockback);
        let killed = h.hp <= 0;
        let kind = entity.kind;
        let death_pos = entity.center();
        let heavy = matches!(attack, AttackKind::Finisher | AttackKind::Spin);
        (killed, kind, death_pos, heavy)
    };

    world.hitstop = if heavy {
        tuning::HITSTOP_HEAVY
    } else {
        tuning::HITSTOP_NORMAL
    };
    if heavy || killed {
        world.camera.add_shake(if heavy { 2.5 } else { 1.5 }, 6);
    }

    world.push_event(WorldEvent::FxRequest(FxKind::Impact { pos }));
    world.push_event(WorldEvent::FxRequest(FxKind::DamageNumber {
        pos,
        amount: damage.ceil() as i32,
        gold: heavy,
    }));
    world.push_event(WorldEvent::Sfx(SfxId::HitEnemy));

    let verb = match attack {
        AttackKind::Finisher => StyleVerb::Finisher,
        AttackKind::Spin => StyleVerb::ChargeSpin,
        AttackKind::Slash | AttackKind::Backslash | AttackKind::Beam | AttackKind::DebugShot => {
            StyleVerb::Slash
        }
    };
    world.push_event(WorldEvent::StyleAction(verb));

    if killed {
        if let Some(e) = world.get_mut(target) {
            if let EntityData::Dummy(d) = &mut e.data {
                d.dead_ticks = Some(0);
            }
            e.health = Some(Health {
                hp: 0,
                max: tuning::DUMMY_HP,
                iframes: 0,
                flash: 0,
            });
            e.body = None;
        }
        world.push_event(WorldEvent::Killed {
            kind,
            pos: death_pos,
        });
    }
}

/// Player damage from enemy contact / projectiles (1B enemies + 1A debug shot).
pub fn apply_player_damage(world: &mut World, amount: i32, dir: Vec2) {
    let pid = world.player_id;
    let (iframes, shield_held, shield_ticks, facing, center) = {
        let Some(p) = world.get(pid) else {
            return;
        };
        let iframes = p.health.map(|h| h.iframes).unwrap_or(0);
        let (sh, st) = match &p.data {
            EntityData::Player(pd) => (pd.shield_held, pd.shield_ticks),
            _ => (false, 0),
        };
        (iframes, sh, st, p.facing, p.center())
    };
    if iframes > 0 {
        return;
    }

    let incoming_from = dir.normalize_or_zero().scale(-1.0);
    let front = facing.unit().dot(incoming_from) > 0.0;

    if shield_held && front {
        let perfect = shield_ticks > 0 && shield_ticks <= tuning::PERFECT_BLOCK_WINDOW;
        {
            let Some(p) = world.get_mut(pid) else {
                return;
            };
            if let EntityData::Player(pd) = &mut p.data {
                if perfect {
                    crate::combat::energy::refund(pd, tuning::PERFECT_BLOCK_REFUND);
                } else if pd.energy >= tuning::BLOCK_ENERGY {
                    pd.energy -= tuning::BLOCK_ENERGY;
                    pd.ticks_since_spend = 0;
                }
            }
            p.knockback = dir.normalize_or_zero().scale(tuning::BLOCK_PUSHBACK);
        }
        world.push_event(WorldEvent::FxRequest(FxKind::BlockSpark { pos: center }));
        if perfect {
            world.push_event(WorldEvent::FxRequest(FxKind::Toast {
                text: "PERFECT BLOCK",
            }));
            world.push_event(WorldEvent::StyleAction(StyleVerb::PerfectBlock));
            world.push_event(WorldEvent::Sfx(SfxId::PerfectBlock));
            reflect_projectiles_near(world, center);
        } else {
            world.push_event(WorldEvent::Sfx(SfxId::ShieldBlock));
        }
        return;
    }

    let mut style_events = Vec::new();
    {
        let Some(p) = world.get_mut(pid) else {
            return;
        };
        if let Some(h) = p.health.as_mut() {
            h.hp = (h.hp - amount).max(0);
            h.iframes = tuning::PLAYER_IFRAMES;
            h.flash = tuning::FLASH_TICKS;
        }
        p.knockback = dir.normalize_or_zero().scale(2.5);
        let hp = p.health.map(|h| h.hp).unwrap_or(0);
        if let EntityData::Player(pd) = &mut p.data {
            pd.hearts = hp;
            style_events = style::on_player_damaged(pd);
        }
    }
    for ev in style_events {
        world.push_event(ev);
    }

    world.camera.add_shake(2.0, 8);
    world.push_event(WorldEvent::Sfx(SfxId::HurtPlayer));
}

fn reflect_projectiles_near(world: &mut World, center: Vec2) {
    let ids = world.alive_ids();
    for id in ids {
        let Some(e) = world.get_mut(id) else {
            continue;
        };
        match e.kind {
            EntityKind::SwordBeam | EntityKind::DebugShot => {
                if e.center().sub(center).len() < 40.0 {
                    e.vel = e.vel.scale(-1.0);
                    if let EntityData::Beam(b) = &mut e.data {
                        b.dir = b.dir.scale(-1.0);
                        b.from_player = true;
                        b.hit = false;
                    }
                    if let Some(body) = e.body.as_mut() {
                        body.layer = layer::PLAYER_HIT;
                        body.mask = layer::ENEMY_BODY;
                    }
                }
            }
            EntityKind::Player
            | EntityKind::Dummy
            | EntityKind::Pickup
            | EntityKind::FairyFountain => {}
        }
    }
}
