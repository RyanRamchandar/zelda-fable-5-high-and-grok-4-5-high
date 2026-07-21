//! Sword state machine: slash combo, charge spin, beam spawn.

use crate::combat::tuning;
use crate::fx::FxKind;
use crate::math::{Dir4, Vec2};
use crate::world::entity::{
    layer, AnimState, BeamData, Body, Entity, EntityData, EntityId, EntityKind, PlayerData,
    PlayerState,
};
use crate::world::{ActiveAttack, AttackKind, World, WorldEvent};
use content::audio::sfx::SfxId;
use engine::input::{InputState, BUTTON_ATTACK};

pub fn update_sword(
    world: &mut World,
    pid: EntityId,
    input: &InputState,
    facing: Dir4,
    center: Vec2,
) {
    let attack_pressed = input.buttons[BUTTON_ATTACK].pressed;
    let attack_held = input.buttons[BUTTON_ATTACK].held;

    let mut spawn_beam = false;
    let mut clear_swing = None;
    let mut toast_finisher = false;
    let mut energy_denied = false;
    let mut sfx = Vec::new();
    let mut fx = Vec::new();
    let mut active: Option<ActiveAttack> = None;
    let mut lunge = Vec2::ZERO;

    {
        let Some(player) = world.get_mut(pid) else {
            return;
        };
        let EntityData::Player(pd) = &mut player.data else {
            return;
        };

        if pd.combo_drop > 0 {
            pd.combo_drop -= 1;
        }
        if attack_pressed {
            pd.buffer_attack = true;
        }

        // Advance dash recovery here so slash-cancel can transition same tick.
        if let PlayerState::DashRecovery { tick } = pd.state {
            let t = tick + 1;
            if t >= tuning::DASH_RECOVERY {
                pd.state = PlayerState::Idle;
            } else {
                pd.state = PlayerState::DashRecovery { tick: t };
            }
        }

        match pd.state {
            PlayerState::Idle | PlayerState::DashRecovery { .. } => {
                // Combo continue / dash-cancel slash use the buffer; fresh press starts charge.
                if pd.buffer_attack && !pd.shield_held && pd.combo_drop > 0 {
                    pd.buffer_attack = false;
                    let stage = player.anim.sheet.min(2) as u8;
                    begin_swing(pd, &mut player.anim, stage);
                    clear_swing = Some(pd.swing_id);
                } else if pd.buffer_attack
                    && !pd.shield_held
                    && matches!(pd.state, PlayerState::DashRecovery { .. })
                {
                    pd.buffer_attack = false;
                    begin_swing(pd, &mut player.anim, 0);
                    clear_swing = Some(pd.swing_id);
                } else if attack_pressed && !pd.shield_held && matches!(pd.state, PlayerState::Idle)
                {
                    pd.buffer_attack = false;
                    pd.state = PlayerState::Charging { tick: 0 };
                    pd.charge_ready_sfx = false;
                }
            }
            PlayerState::Charging { tick } => {
                let t = tick + 1;
                fx.push(FxKind::ChargeShimmer { pos: center });
                if t >= tuning::CHARGE_TICKS && !pd.charge_ready_sfx {
                    pd.charge_ready_sfx = true;
                    sfx.push(SfxId::ChargeReady);
                }
                if !attack_held {
                    if t >= tuning::CHARGE_TICKS {
                        if pd.energy >= tuning::SPIN_ENERGY {
                            pd.energy -= tuning::SPIN_ENERGY;
                            pd.ticks_since_spend = 0;
                            pd.swing_id = pd.swing_id.wrapping_add(1);
                            clear_swing = Some(pd.swing_id);
                            pd.state = PlayerState::Spin { tick: 0 };
                            sfx.push(SfxId::SpinRelease);
                            fx.push(FxKind::SlashArc {
                                pos: center,
                                facing,
                                spin: true,
                                life: 12,
                            });
                        } else {
                            energy_denied = true;
                            pd.energy_deny_flash = tuning::ENERGY_DENY_FLASH;
                            begin_swing(pd, &mut player.anim, 0);
                            clear_swing = Some(pd.swing_id);
                        }
                    } else {
                        begin_swing(pd, &mut player.anim, 0);
                        clear_swing = Some(pd.swing_id);
                    }
                } else {
                    pd.state = PlayerState::Charging { tick: t };
                }
            }
            PlayerState::Swing { stage, tick } => {
                let t = tick + 1;
                if pd.lunge_ticks > 0 {
                    lunge = facing
                        .unit()
                        .scale(tuning::FINISHER_HOP_PX / f32::from(tuning::FINISHER_HOP_TICKS));
                    pd.lunge_ticks -= 1;
                }
                if t >= tuning::SLASH_TICKS {
                    if stage < 2 && pd.buffer_attack {
                        pd.buffer_attack = false;
                        begin_swing(pd, &mut player.anim, stage + 1);
                        clear_swing = Some(pd.swing_id);
                    } else {
                        pd.state = PlayerState::Idle;
                        pd.combo_drop = tuning::COMBO_DROP_WINDOW;
                        player.anim.sheet = u16::from(stage) + 1;
                        pd.buffer_attack = false;
                    }
                } else {
                    if t + tuning::SLASH_BUFFER_TICKS >= tuning::SLASH_TICKS && attack_pressed {
                        pd.buffer_attack = true;
                    }
                    pd.state = PlayerState::Swing { stage, tick: t };
                    if (tuning::SLASH_ACTIVE_START..=tuning::SLASH_ACTIVE_END).contains(&t) {
                        let (dmg, kb, kind) = match stage {
                            0 => (tuning::SLASH_DAMAGE, tuning::KB_QUICK, AttackKind::Slash),
                            1 => (
                                tuning::SLASH_DAMAGE,
                                tuning::KB_QUICK,
                                AttackKind::Backslash,
                            ),
                            _ => (
                                tuning::SLASH_DAMAGE * tuning::FINISHER_DAMAGE_MULT,
                                tuning::KB_FINISHER * tuning::FINISHER_KB_MULT,
                                AttackKind::Finisher,
                            ),
                        };
                        if stage == 2 && t == tuning::SLASH_ACTIVE_START {
                            toast_finisher = true;
                            pd.lunge_ticks = tuning::FINISHER_HOP_TICKS;
                            sfx.push(SfxId::Finisher);
                        }
                        let reach = facing.unit().scale(tuning::SLASH_REACH);
                        active = Some(ActiveAttack {
                            owner: pid,
                            kind,
                            swing_id: pd.swing_id,
                            center: center.add(reach),
                            half: Vec2::new(tuning::SLASH_HIT_HALF, tuning::SLASH_HIT_HALF),
                            radius: 0.0,
                            use_radius: false,
                            dir: facing.unit(),
                            damage: dmg,
                            knockback: kb,
                        });
                        if t == tuning::SLASH_ACTIVE_START {
                            fx.push(FxKind::SlashArc {
                                pos: center,
                                facing,
                                spin: false,
                                life: 8,
                            });
                            match stage {
                                0 => {
                                    sfx.push(SfxId::Slash1);
                                    if pd.at_full_hearts() {
                                        spawn_beam = true;
                                    }
                                }
                                1 => sfx.push(SfxId::Slash2),
                                _ => {}
                            }
                        }
                    }
                }
            }
            PlayerState::Spin { tick } => {
                let t = tick + 1;
                if t >= tuning::SPIN_TICKS {
                    pd.state = PlayerState::Idle;
                    pd.combo_drop = 0;
                } else {
                    pd.state = PlayerState::Spin { tick: t };
                    active = Some(ActiveAttack {
                        owner: pid,
                        kind: AttackKind::Spin,
                        swing_id: pd.swing_id,
                        center,
                        half: Vec2::ZERO,
                        radius: tuning::SPIN_RADIUS,
                        use_radius: true,
                        dir: facing.unit(),
                        damage: tuning::SLASH_DAMAGE * tuning::SPIN_DAMAGE_MULT,
                        knockback: tuning::KB_SPIN,
                    });
                }
            }
            PlayerState::Dash { tick } => {
                if tick >= tuning::DASH_CANCEL_SLASH_FROM && pd.buffer_attack {
                    pd.buffer_attack = false;
                    begin_swing(pd, &mut player.anim, 0);
                    clear_swing = Some(pd.swing_id);
                    player.vel = Vec2::ZERO;
                }
            }
            PlayerState::LedgeHop { .. } => {
                // Input locked during ledge hop.
            }
        }
    }

    if let Some(sid) = clear_swing {
        world.clear_swing_hits(sid);
    }
    if let Some(a) = active {
        world.active_attacks.push(a);
    }
    for s in sfx {
        world.push_event(WorldEvent::Sfx(s));
    }
    for f in fx {
        world.push_event(WorldEvent::FxRequest(f));
    }
    if toast_finisher {
        world.push_event(WorldEvent::FxRequest(FxKind::Toast {
            text: "COMBO FINISH!",
        }));
    }
    if energy_denied {
        world.push_event(WorldEvent::EnergyDenied);
        world.push_event(WorldEvent::Sfx(SfxId::Refused));
    }
    if lunge.len_sq() > 0.0 {
        if let Some(p) = world.get_mut(pid) {
            p.vel = p.vel.add(lunge);
        }
    }
    if spawn_beam {
        spawn_sword_beam(world, pid, facing, center);
    }
}

fn begin_swing(pd: &mut PlayerData, anim: &mut AnimState, stage: u8) {
    pd.swing_id = pd.swing_id.wrapping_add(1);
    pd.hit_mask = 0;
    pd.state = PlayerState::Swing { stage, tick: 0 };
    anim.sheet = u16::from(stage);
    anim.frame = 0;
    anim.timer = 0;
}

fn spawn_sword_beam(world: &mut World, pid: EntityId, facing: Dir4, center: Vec2) {
    let dir = facing.unit();
    let swing_id = world
        .get(pid)
        .and_then(|p| match &p.data {
            EntityData::Player(pd) => Some(pd.swing_id),
            _ => None,
        })
        .unwrap_or(0);
    let pos = center.add(dir.scale(12.0)).sub(Vec2::new(3.0, 3.0));
    world.spawn(Entity {
        kind: EntityKind::SwordBeam,
        pos,
        vel: dir.scale(tuning::BEAM_SPEED),
        facing,
        body: Some(Body {
            half: Vec2::new(3.0, 3.0),
            solid: false,
            layer: layer::PLAYER_HIT,
            mask: layer::ENEMY_BODY,
        }),
        health: None,
        knockback: Vec2::ZERO,
        anim: AnimState::default(),
        data: EntityData::Beam(BeamData {
            dir,
            traveled: 0.0,
            damage: tuning::SLASH_DAMAGE * tuning::BEAM_DAMAGE_MULT,
            knockback: tuning::KB_QUICK * 0.8,
            from_player: true,
            swing_id,
            hit: false,
        }),
        alive: true,
    });
    world.push_event(WorldEvent::Sfx(SfxId::Beam));
}

pub fn update_beams(world: &mut World) {
    let ids = world.alive_ids();
    let mut kill = Vec::new();
    for id in ids {
        let traveled = {
            let Some(e) = world.get_mut(id) else {
                continue;
            };
            match e.kind {
                EntityKind::SwordBeam | EntityKind::DebugShot => {}
                EntityKind::Player
                | EntityKind::Dummy
                | EntityKind::Pickup
                | EntityKind::FairyFountain
                | EntityKind::Slime
                | EntityKind::Bat
                | EntityKind::Octorok
                | EntityKind::OctorokRock
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
                | EntityKind::Boomerang => continue,
            }
            let EntityData::Beam(b) = &mut e.data else {
                continue;
            };
            e.pos = e.pos.add(e.vel);
            b.traveled += e.vel.len();
            b.traveled
        };
        let max = match world.get(id).map(|e| e.kind) {
            Some(EntityKind::DebugShot) => 220.0,
            _ => tuning::BEAM_RANGE,
        };
        if traveled >= max {
            kill.push(id);
        }
    }
    for id in kill {
        world.despawn(id);
    }
}
