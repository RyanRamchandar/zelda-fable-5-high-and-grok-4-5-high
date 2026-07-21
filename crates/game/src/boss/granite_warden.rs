//! Granite Warden fight: crystal prime → gale → core windows, 3 phases.

use content::audio::sfx::SfxId;
use content::maps::TILE_PX;

use crate::combat::tuning;
use crate::fx::FxKind;
use crate::math::{Dir4, Vec2};
use crate::world::entity::{
    layer, AnimState, Body, Entity, EntityData, EntityId, EntityKind, Health, WardenAttack,
    WardenData, WardenPhase, WindCrystalData,
};
use crate::world::{World, WorldEvent};
use crate::Game;

use super::{BossPhase, BossState, CutsceneKind};

pub(crate) fn arena_cx() -> f32 {
    69.0 * TILE_PX
}
pub(crate) fn arena_cy() -> f32 {
    7.0 * TILE_PX
}
pub(crate) fn perch_w() -> Vec2 {
    Vec2::new(62.0 * TILE_PX, 7.0 * TILE_PX)
}
pub(crate) fn perch_e() -> Vec2 {
    Vec2::new(76.0 * TILE_PX, 7.0 * TILE_PX)
}

pub fn spawn_fight(game: &mut Game) {
    let warden = spawn_warden(&mut game.world);
    let c0 = spawn_crystal(&mut game.world, perch_w(), 0);
    let c1 = spawn_crystal(&mut game.world, perch_e(), 1);
    game.boss = Some(BossState {
        warden,
        crystals: [c0, c1],
        cutscene: None,
        phase: BossPhase::Fight,
        bar_visible: false,
        rim_crumbled: false,
        name_plate: 0,
        victory_step: 0,
        short_intro: false,
    });
}

fn spawn_warden(world: &mut World) -> EntityId {
    let pos = Vec2::new(arena_cx() - 24.0, arena_cy() - 24.0);
    world.spawn(Entity {
        kind: EntityKind::GraniteWarden,
        pos,
        vel: Vec2::ZERO,
        facing: Dir4::Down,
        body: Some(Body {
            half: Vec2::new(20.0, 20.0),
            solid: false,
            layer: layer::ENEMY_BODY,
            mask: layer::PLAYER_HIT,
        }),
        health: Some(Health {
            hp: tuning::WARDEN_HP as i32,
            max: tuning::WARDEN_HP as i32,
            iframes: 0,
            flash: 0,
        }),
        knockback: Vec2::ZERO,
        anim: AnimState::default(),
        data: EntityData::GraniteWarden(WardenData {
            phase: WardenPhase::One,
            attack: WardenAttack::Idle,
            timer: 60,
            core_exposed: 0,
            hp: tuning::WARDEN_HP,
            max_hp: tuning::WARDEN_HP,
            fake_armed: false,
        }),
        alive: true,
    })
}

fn spawn_crystal(world: &mut World, pos: Vec2, perch: u8) -> EntityId {
    world.spawn(Entity {
        kind: EntityKind::WindCrystal,
        pos: Vec2::new(pos.x - 8.0, pos.y - 8.0),
        vel: Vec2::ZERO,
        facing: Dir4::Down,
        body: Some(Body {
            half: Vec2::new(8.0, 8.0),
            solid: false,
            layer: layer::ENEMY_BODY,
            mask: 0,
        }),
        health: None,
        knockback: Vec2::ZERO,
        anim: AnimState::default(),
        data: EntityData::WindCrystal(WindCrystalData {
            perch,
            primed: 0,
            throw_id_seen: 0,
            orbit_angle: if perch == 0 { 0.0 } else { std::f32::consts::PI },
        }),
        alive: true,
    })
}

pub fn on_damaged(world: &mut World, id: EntityId) {
    let (hp, phase) = {
        let Some(e) = world.get(id) else {
            return;
        };
        let EntityData::GraniteWarden(d) = &e.data else {
            return;
        };
        (d.hp, d.phase)
    };
    let next = if hp <= tuning::WARDEN_PHASE3_HP {
        WardenPhase::Three
    } else if hp <= tuning::WARDEN_PHASE2_HP {
        WardenPhase::Two
    } else {
        WardenPhase::One
    };
    if next != phase {
        phase_break(world, id, next);
    }
}

fn phase_break(world: &mut World, id: EntityId, next: WardenPhase) {
    if let Some(e) = world.get_mut(id) {
        if let EntityData::GraniteWarden(d) = &mut e.data {
            d.phase = next;
            d.attack = WardenAttack::Idle;
            d.timer = 90;
            d.core_exposed = 0;
            d.fake_armed = false;
            e.anim.frame = 2;
        }
    }
    world.hitstop = tuning::HITSTOP_BOSS_BREAK;
    world.camera.add_shake(3.0, 18);
    let pos = world.get(id).map(|e| e.center()).unwrap_or(Vec2::ZERO);
    world.push_event(WorldEvent::FxRequest(FxKind::KillPoof { pos }));
    world.push_event(WorldEvent::FxRequest(FxKind::Toast {
        text: "PHASE BREAK",
    }));
    world.push_event(WorldEvent::Sfx(SfxId::PhaseBreak));
    world.push_event(WorldEvent::Sfx(SfxId::WardenRoar));
}

pub fn tick_fight(game: &mut Game) {
    let Some(boss) = game.boss.as_ref() else {
        return;
    };
    let warden = boss.warden;
    let crystals = boss.crystals;
    if game.world.get(warden).is_none() {
        return;
    }

    super::warden_crystals::update_crystals(game, crystals, warden);
    super::warden_crystals::maybe_gale(game, crystals, warden);
    tick_warden_ai(game, warden);
    super::warden_crystals::tick_core_timer(game, warden);
    super::warden_crystals::tick_fake_flash(game, warden);

    // Defeat?
    let dead = game
        .world
        .get(warden)
        .and_then(|e| match &e.data {
            EntityData::GraniteWarden(d) => Some(d.hp <= 0.0),
            _ => None,
        })
        .unwrap_or(true);
    if dead {
        begin_collapse(game);
    }
}

fn tick_warden_ai(game: &mut Game, warden: EntityId) {
    let (phase, attack, timer, exposed) = {
        let Some(e) = game.world.get(warden) else {
            return;
        };
        let EntityData::GraniteWarden(d) = &e.data else {
            return;
        };
        (d.phase, d.attack, d.timer, d.core_exposed)
    };
    if exposed > 0 && !matches!(attack, WardenAttack::FakeFlash) {
        // Staggered — no attacks during real core window.
        if let Some(e) = game.world.get_mut(warden) {
            if let EntityData::GraniteWarden(d) = &mut e.data {
                d.timer = d.timer.saturating_add(1);
            }
        }
        return;
    }

    let mut next_attack = attack;
    let mut next_timer = timer.saturating_add(1);
    let mut sfx = None;
    let mut slam = false;
    let mut rocks = 0u8;
    let mut sweep = false;
    let mut spawn_pebbles = false;

    match attack {
        WardenAttack::Idle => {
            if next_timer >= 90 {
                // Pick attack.
                next_timer = 0;
                next_attack = match (phase, (game.world.tick / 17) % 3) {
                    (WardenPhase::One, 0) | (WardenPhase::One, 2) => WardenAttack::SlamTele,
                    (WardenPhase::One, _) => WardenAttack::RockFanTele,
                    (WardenPhase::Two, 0) => WardenAttack::SlamTele,
                    (WardenPhase::Two, 1) => WardenAttack::SweepTele,
                    (WardenPhase::Two, _) => WardenAttack::RockFanTele,
                    (WardenPhase::Three, 0) => WardenAttack::SlamTele,
                    (WardenPhase::Three, 1) => WardenAttack::SweepTele,
                    (WardenPhase::Three, _) => WardenAttack::RockFanTele,
                };
                if next_attack == WardenAttack::SlamTele {
                    sfx = Some(SfxId::WardenRoar);
                } else if next_attack == WardenAttack::SweepTele {
                    sfx = Some(SfxId::WardenSweep);
                } else {
                    sfx = Some(SfxId::RockFan);
                }
            }
        }
        WardenAttack::SlamTele => {
            if next_timer >= tuning::WARDEN_SLAM_TELE {
                next_attack = WardenAttack::Slam;
                next_timer = 0;
                sfx = Some(SfxId::WardenSlam);
            }
        }
        WardenAttack::Slam => {
            if next_timer == 1 {
                slam = true;
                if matches!(phase, WardenPhase::Two | WardenPhase::Three) {
                    spawn_pebbles = true;
                }
            }
            if next_timer >= 20 {
                next_attack = WardenAttack::Idle;
                next_timer = 0;
            }
        }
        WardenAttack::RockFanTele => {
            if next_timer >= 30 {
                next_attack = WardenAttack::RockFan;
                next_timer = 0;
            }
        }
        WardenAttack::RockFan => {
            if next_timer == 1 {
                rocks = if phase == WardenPhase::Three { 5 } else { 3 };
            }
            if next_timer >= 16 {
                next_attack = WardenAttack::Idle;
                next_timer = 0;
            }
        }
        WardenAttack::SweepTele => {
            if next_timer >= tuning::WARDEN_SWEEP_TELE {
                next_attack = WardenAttack::Sweep;
                next_timer = 0;
                sfx = Some(SfxId::WardenSweep);
            }
        }
        WardenAttack::Sweep => {
            if next_timer == 1 {
                sweep = true;
            }
            if next_timer >= 18 {
                next_attack = WardenAttack::Idle;
                next_timer = 0;
            }
        }
        WardenAttack::FakeFlash => {
            if next_timer >= 70 {
                // Counter-slam sooner.
                next_attack = WardenAttack::SlamTele;
                next_timer = tuning::WARDEN_SLAM_TELE.saturating_sub(12);
                if let Some(e) = game.world.get_mut(warden) {
                    e.anim.frame = 0;
                }
            }
        }
        WardenAttack::Collapse => {}
    }

    {
        let Some(e) = game.world.get_mut(warden) else {
            return;
        };
        e.anim.frame = match next_attack {
            WardenAttack::SlamTele | WardenAttack::SweepTele | WardenAttack::RockFanTele => 2,
            WardenAttack::FakeFlash => 1,
            _ if exposed > 0 => 1,
            _ => 0,
        };
        if let EntityData::GraniteWarden(d) = &mut e.data {
            d.attack = next_attack;
            d.timer = next_timer;
        }
    }

    if let Some(s) = sfx {
        game.world.push_event(WorldEvent::Sfx(s));
    }
    if slam {
        super::warden_attacks::do_slam(game, warden);
    }
    if rocks > 0 {
        super::warden_attacks::do_rock_fan(game, warden, rocks);
    }
    if sweep {
        super::warden_attacks::do_sweep(game, warden);
    }
    if spawn_pebbles {
        super::warden_attacks::spawn_two_pebbles(game);
    }

    // Phase 3 rim crumble once.
    if phase == WardenPhase::Three {
        if let Some(boss) = game.boss.as_mut() {
            if !boss.rim_crumbled {
                boss.rim_crumbled = true;
                super::warden_attacks::crumble_rim(game);
            }
        }
    }
}

fn begin_collapse(game: &mut Game) {
    let Some(boss) = game.boss.as_mut() else {
        return;
    };
    boss.phase = BossPhase::Victory;
    boss.victory_step = 0;
    boss.bar_visible = false;
    boss.cutscene = Some(super::Cutscene {
        kind: CutsceneKind::Collapse,
        t: 0,
    });
    if let Some(e) = game.world.get_mut(boss.warden) {
        if let EntityData::GraniteWarden(d) = &mut e.data {
            d.attack = WardenAttack::Collapse;
            d.timer = 0;
        }
        e.anim.frame = 3;
        e.body = None;
    }
    for cid in boss.crystals {
        if let Some(e) = game.world.get_mut(cid) {
            e.alive = false;
        }
        game.world.despawn(cid);
    }
    // Clear pebbles.
    let ids: Vec<_> = game
        .world
        .alive_ids()
        .into_iter()
        .filter(|&id| {
            matches!(
                game.world.get(id).map(|e| e.kind),
                Some(EntityKind::PebbleCrawler | EntityKind::OctorokRock)
            )
        })
        .collect();
    for id in ids {
        game.world.despawn(id);
    }
    game.world
        .push_event(WorldEvent::Sfx(SfxId::WardenFall));
    game.world.camera.add_shake(3.0, 24);
}

pub fn tick_collapse(game: &mut Game) -> bool {
    let Some(boss) = game.boss.as_mut() else {
        return false;
    };
    let Some(cs) = boss.cutscene.as_mut() else {
        return false;
    };
    if cs.kind != CutsceneKind::Collapse {
        return false;
    }
    cs.t = cs.t.saturating_add(1);
    let t = cs.t;
    if t == 40 {
        game.world.camera.add_shake(3.0, 12);
    }
    if t >= 120 {
        boss.cutscene = None;
        boss.phase = BossPhase::Victory;
        boss.victory_step = 1;
        return false;
    }
    true // pause input
}
