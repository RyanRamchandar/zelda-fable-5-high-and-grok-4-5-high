//! Apply resolved hits to entities.

use crate::combat::style::{self, StyleVerb};
use crate::combat::tuning;
use crate::fx::FxKind;
use crate::math::Vec2;
use crate::world::entity::{
    layer, EntityData, EntityId, EntityKind, Health, OctorokState, RaiderSpearState,
};
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
    let refuse = {
        let Some(entity) = world.get(target) else {
            return;
        };
        if !entity.is_enemy() {
            return;
        }
        if let EntityData::Dummy(d) = &entity.data {
            if d.dead_ticks.is_some() {
                return;
            }
        }
        let hidden = matches!(
            &entity.data,
            EntityData::Octorok(d)
                if d.spawn_telegraph == 0 && d.state == OctorokState::Hide
        );
        let frontal = entity.facing.unit().dot(dir.normalize_or_zero()) > 0.35;
        let guarded = match &entity.data {
            EntityData::RaiderSpear(d)
                if d.spawn_telegraph == 0 && d.state == RaiderSpearState::Guard && frontal =>
            {
                true
            }
            EntityData::Skeleton(d) if d.spawn_telegraph == 0 && d.shield_up && frontal => true,
            _ => false,
        };
        let ironshell_front = match &entity.data {
            EntityData::Ironshell(d) => {
                if matches!(attack, AttackKind::Bomb) {
                    false
                } else if d.spawn_telegraph > 0 {
                    true
                } else if matches!(d.state, crate::world::entity::IronshellState::Stagger)
                    || d.stun_ticks > 0
                {
                    false
                } else {
                    crate::enemies::ironshell::is_frontal_hit(entity.facing, dir)
                }
            }
            _ => false,
        };
        let warden_closed = match &entity.data {
            EntityData::GraniteWarden(d) => d.core_exposed == 0,
            _ => false,
        };
        if matches!(entity.kind, EntityKind::WindCrystal) {
            return;
        }
        if entity.health.map(|h| h.iframes > 0).unwrap_or(true)
            && !hidden
            && !guarded
            && !ironshell_front
            && !warden_closed
        {
            return;
        }
        if (hidden || guarded) && attack != AttackKind::Boomerang {
            Some(if guarded {
                SfxId::GuardClank
            } else {
                SfxId::Refused
            })
        } else if ironshell_front || warden_closed {
            Some(SfxId::GuardClank)
        } else {
            None
        }
    };
    if let Some(sfx) = refuse {
        if let Some(e) = world.get_mut(target) {
            if let Some(h) = e.health.as_mut() {
                h.flash = tuning::FLASH_TICKS;
            }
            // Light self-knock for frontal refuse rhythm.
            if e.kind == EntityKind::Ironshell {
                e.knockback = dir.normalize_or_zero().scale(0.6);
            }
        }
        world.push_event(WorldEvent::FxRequest(FxKind::BlockSpark { pos }));
        world.push_event(WorldEvent::Sfx(sfx));
        return;
    }

    // Warden damage goes through boss HP tracker (entity health mirrors).
    let is_warden = matches!(
        world.get(target).map(|e| e.kind),
        Some(EntityKind::GraniteWarden)
    );
    let dmg = if is_warden {
        if attack == AttackKind::Boomerang {
            0.5
        } else {
            tuning::WARDEN_CORE_HIT.min(damage.max(tuning::WARDEN_CORE_HIT * 0.5))
        }
    } else {
        damage
    };

    let (killed, kind, death_pos, heavy, is_dummy) = {
        let Some(entity) = world.get_mut(target) else {
            return;
        };
        let Some(h) = entity.health.as_mut() else {
            return;
        };
        h.hp = (h.hp as f32 - dmg).ceil().max(0.0) as i32;
        h.flash = tuning::FLASH_TICKS;
        h.iframes = tuning::ENEMY_IFRAMES;
        // Warden / ironshell: no / soft knockback.
        if entity.kind != EntityKind::GraniteWarden {
            entity.knockback = dir.normalize_or_zero().scale(knockback);
        }
        if let EntityData::GraniteWarden(wd) = &mut entity.data {
            wd.hp = (wd.hp - dmg).max(0.0);
        }
        let killed = h.hp <= 0;
        let kind = entity.kind;
        let death_pos = entity.center();
        let heavy = matches!(attack, AttackKind::Finisher | AttackKind::Spin);
        let is_dummy = kind == EntityKind::Dummy;
        (killed, kind, death_pos, heavy, is_dummy)
    };

    if is_warden {
        world.push_event(WorldEvent::Sfx(SfxId::CoreHit));
        crate::boss::on_warden_damaged(world, target);
    }

    if attack == AttackKind::Boomerang {
        let kind_now = world.get(target).map(|e| e.kind);
        if matches!(kind_now, Some(EntityKind::Ironshell)) {
            // Back-hit: damage already applied; stun + open shell.
            crate::enemies::stun(world, target, 60);
            crate::enemies::ironshell::stagger(world, target);
            world.push_event(WorldEvent::StyleAction(StyleVerb::BoomerangStun));
            world.push_event(WorldEvent::Sfx(SfxId::IronshellCrack));
        } else if !matches!(
            kind_now,
            Some(EntityKind::GraniteWarden | EntityKind::WindCrystal)
        ) {
            crate::enemies::stun(world, target, 60);
            world.push_event(WorldEvent::StyleAction(StyleVerb::BoomerangStun));
            if matches!(kind_now, Some(EntityKind::Skeleton)) {
                crate::enemies::skeleton::stagger(world, target);
            }
        }
    }
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
    world.push_event(WorldEvent::Sfx(if kind == EntityKind::Dummy {
        SfxId::HitEnemy
    } else {
        SfxId::EnemyHurt
    }));

    if let Some(verb) = match attack {
        AttackKind::Finisher => Some(StyleVerb::Finisher),
        AttackKind::Spin => Some(StyleVerb::ChargeSpin),
        AttackKind::Slash | AttackKind::Backslash | AttackKind::Beam | AttackKind::DebugShot => {
            Some(StyleVerb::Slash)
        }
        AttackKind::Bomb | AttackKind::Boomerang => None,
    } {
        world.push_event(WorldEvent::StyleAction(verb));
    }

    if killed {
        if is_dummy {
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
        } else {
            world.despawn(target);
        }
        world.push_event(WorldEvent::Killed {
            kind,
            pos: death_pos,
        });
    }
}

/// Player damage from enemy contact / projectiles.
pub fn apply_player_damage(
    world: &mut World,
    amount: i32,
    dir: Vec2,
    source: Option<EntityId>,
) {
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
        destroy_hostile_projectiles_near(world, center);
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
            if let Some(src) = source {
                match world.get(src).map(|e| e.kind) {
                    Some(EntityKind::Skeleton) => {
                        crate::enemies::skeleton::try_perfect_block_stagger(world, src);
                    }
                    Some(EntityKind::Ironshell) => {
                        crate::enemies::ironshell::try_perfect_block_stagger(world, src);
                    }
                    _ => {}
                }
            }
        } else {
            world.push_event(WorldEvent::Sfx(SfxId::ShieldBlock));
            destroy_hostile_projectiles_near(world, center);
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

    destroy_hostile_projectiles_near(world, center);
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
            EntityKind::OctorokRock => {
                if e.center().sub(center).len() < 40.0 {
                    e.vel = e.vel.scale(-1.0);
                    if let EntityData::Rock(r) = &mut e.data {
                        r.dir = r.dir.scale(-1.0);
                        r.from_player = true;
                        r.hit = false;
                        r.damage = tuning::OCTOROK_ROCK_REFLECT_DAMAGE;
                    }
                    if let Some(body) = e.body.as_mut() {
                        body.layer = layer::PLAYER_HIT;
                        body.mask = layer::ENEMY_BODY;
                    }
                    world.push_event(WorldEvent::Sfx(SfxId::ReflectZing));
                }
            }
            EntityKind::Player
            | EntityKind::Dummy
            | EntityKind::Pickup
            | EntityKind::FairyFountain
            | EntityKind::Slime
            | EntityKind::Bat
            | EntityKind::Octorok
            | EntityKind::RaiderSpear
            | EntityKind::RaiderTorch
            | EntityKind::Wisp
            | EntityKind::Skeleton
            | EntityKind::Ironshell
            | EntityKind::GraniteWarden
            | EntityKind::WindCrystal
            | EntityKind::PebbleCrawler
            | EntityKind::TorchProj
            | EntityKind::TorchFlame
            | EntityKind::Sign
            | EntityKind::Npc
            | EntityKind::Chest
            | EntityKind::Gem
            | EntityKind::Bomb
            | EntityKind::Boomerang => {}
        }
    }
}

fn destroy_hostile_projectiles_near(world: &mut World, center: Vec2) {
    let ids = world.alive_ids();
    let mut kill = Vec::new();
    for id in ids {
        let Some(e) = world.get(id) else {
            continue;
        };
        let hostile = match e.kind {
            EntityKind::OctorokRock => matches!(&e.data, EntityData::Rock(r) if !r.from_player),
            EntityKind::DebugShot => matches!(&e.data, EntityData::Beam(b) if !b.from_player),
            EntityKind::TorchProj => true,
            EntityKind::SwordBeam => false,
            _ => false,
        };
        if hostile && e.center().sub(center).len() < 36.0 {
            kill.push(id);
        }
    }
    for id in kill {
        world.despawn(id);
    }
}
