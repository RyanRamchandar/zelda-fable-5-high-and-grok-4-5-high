//! Player movement, sword, shield, dash.

mod sword;

use engine::input::{InputState, BUTTON_CYCLE, BUTTON_DASH, BUTTON_ITEM};

use crate::combat::energy;
use crate::combat::style::{self, StyleVerb};
use crate::combat::tuning;
use crate::fx::FxKind;
use crate::items::{bombs, boomerang};
use crate::math::{Dir4, Vec2};
use crate::save_data::has_flag;
use crate::world::entity::{EntityData, EntityId, EntityKind, PlayerState};
use crate::world::physics;
use crate::world::{World, WorldEvent};
use content::audio::sfx::SfxId;
use content::flags;

use sword::{update_beams, update_sword};

pub fn update(world: &mut World, input: &InputState, flags: &[u16]) {
    let pid = world.player_id;
    update_shield_and_dash_intent(world, pid, input);
    if input.buttons[BUTTON_CYCLE].pressed {
        cycle_items(world, pid, flags);
    }
    update_movement(world, pid, input);
    let (facing, center) = {
        let Some(p) = world.get(pid) else {
            return;
        };
        (p.facing, p.center())
    };
    update_sword(world, pid, input, facing, center);
    update_beams(world);
    tick_player_meters(world, pid);
    check_dash_through(world, pid);
    check_fountain(world, pid);
}

/// Cycle B-items: bombs (if bag) ↔ boomerang (if flag).
fn cycle_items(world: &mut World, pid: EntityId, flag_list: &[u16]) {
    let mut unlocked = Vec::new();
    if let Some(p) = world.get(pid) {
        if let EntityData::Player(pd) = &p.data {
            if pd.bomb_cap > 0 {
                unlocked.push(1u8);
            }
        }
    }
    if has_flag(flag_list, flags::ITEM_BOOMERANG) {
        unlocked.push(2);
    }
    if unlocked.is_empty() {
        world.push_event(WorldEvent::Sfx(SfxId::Refused));
        return;
    }
    if let Some(p) = world.get_mut(pid) {
        if let EntityData::Player(pd) = &mut p.data {
            let cur = unlocked.iter().position(|&i| i == pd.selected_item);
            let next = match cur {
                Some(i) => unlocked[(i + 1) % unlocked.len()],
                None => unlocked[0],
            };
            pd.selected_item = next;
            pd.item_cycle_flash = 12;
        }
    }
    world.push_event(WorldEvent::Sfx(SfxId::ItemCycle));
}

fn update_shield_and_dash_intent(world: &mut World, pid: EntityId, input: &InputState) {
    let item = input.buttons[BUTTON_ITEM].held;
    let dash_pressed = input.buttons[BUTTON_DASH].pressed;

    let mut denied = false;
    let mut dust = None;
    let mut sfx_dash = false;
    let mut place_bomb = false;
    let mut throw_boom = false;

    {
        let Some(p) = world.get_mut(pid) else {
            return;
        };
        let facing = p.facing;
        let center = p.center();
        let EntityData::Player(pd) = &mut p.data else {
            return;
        };

        // Shield (hold Item) + tap-release B-item (≤8 ticks).
        let can_shield = matches!(
            pd.state,
            PlayerState::Idle | PlayerState::DashRecovery { .. }
        );
        if !item
            && pd.shield_ticks >= 1
            && pd.shield_ticks <= tuning::ITEM_TAP_MAX_TICKS
            && can_shield
        {
            if pd.selected_item == 1 && pd.bombs > 0 {
                place_bomb = true;
            } else if pd.selected_item == 2 {
                throw_boom = true;
            }
        }
        if item && can_shield {
            pd.shield_held = true;
            pd.shield_ticks = pd.shield_ticks.saturating_add(1);
        } else if !item {
            pd.shield_held = false;
            pd.shield_ticks = 0;
        }
        if pd.item_cycle_flash > 0 {
            pd.item_cycle_flash -= 1;
        }

        // Dash
        if dash_pressed && matches!(pd.state, PlayerState::Idle) && !pd.shield_held {
            let cost = style::dash_energy_cost(pd);
            if pd.energy >= cost {
                pd.energy -= cost;
                pd.ticks_since_spend = 0;
                let mv = Vec2::new(input.move_vec.0, input.move_vec.1);
                let dash_dir = if mv.len_sq() > 0.01 {
                    mv.normalize_or_zero()
                } else {
                    facing.unit()
                };
                pd.dash_dir = dash_dir;
                pd.state = PlayerState::Dash { tick: 0 };
                pd.dash_through_awarded = false;
                dust = Some(center);
                sfx_dash = true;
            } else {
                denied = true;
                pd.energy_deny_flash = tuning::ENERGY_DENY_FLASH;
            }
        }
    }

    if place_bomb {
        let _ = bombs::try_place(world);
    }
    if throw_boom && !boomerang::try_throw(world) {
        boomerang::note_catch_buffer(world);
    }
    if denied {
        world.push_event(WorldEvent::EnergyDenied);
        world.push_event(WorldEvent::Sfx(SfxId::Refused));
    }
    if sfx_dash {
        world.push_event(WorldEvent::Sfx(SfxId::Dash));
    }
    if let Some(pos) = dust {
        world.push_event(WorldEvent::FxRequest(FxKind::Dust { pos }));
    }
}

fn update_movement(world: &mut World, pid: EntityId, input: &InputState) {
    let mut facing_snap = false;
    let mut dust_pos = None;

    {
        let Some(p) = world.get_mut(pid) else {
            return;
        };
        let EntityData::Player(pd) = &mut p.data else {
            return;
        };

        let move_in = Vec2::new(input.move_vec.0, input.move_vec.1);
        let want = move_in.normalize_or_zero();

        match pd.state {
            PlayerState::LedgeHop { .. } => {
                p.vel = Vec2::ZERO;
            }
            PlayerState::Dash { tick } => {
                let t = tick + 1;
                p.vel = pd.dash_dir.scale(tuning::DASH_SPEED);
                // i-frames ticks 2–8
                if let Some(h) = p.health.as_mut() {
                    if (tuning::DASH_IFRAME_START..=tuning::DASH_IFRAME_END).contains(&t) {
                        h.iframes = h.iframes.max(1);
                    }
                }
                if t >= tuning::DASH_DURATION {
                    pd.state = PlayerState::DashRecovery { tick: 0 };
                    p.vel = Vec2::ZERO;
                } else {
                    pd.state = PlayerState::Dash { tick: t };
                }
                // slash cancel
                if t >= tuning::DASH_CANCEL_SLASH_FROM && pd.buffer_attack {
                    // sword module will pick up buffer on recovery/idle
                }
            }
            PlayerState::Spin { .. } | PlayerState::Swing { .. } => {
                // Limited steering during swing
                let top = if pd.shield_held {
                    tuning::SHIELD_MOVE_SPEED
                } else {
                    tuning::RUN_SPEED * 0.35
                };
                blend_move(pd, &mut p.vel, want, top);
                // Facing locked during swing
            }
            PlayerState::Charging { .. } => {
                let top = tuning::RUN_SPEED * 0.4;
                blend_move(pd, &mut p.vel, want, top);
            }
            PlayerState::Idle | PlayerState::DashRecovery { .. } => {
                let top = if pd.shield_held {
                    tuning::SHIELD_MOVE_SPEED
                } else {
                    tuning::RUN_SPEED
                };
                blend_move(pd, &mut p.vel, want, top);
                let center = Vec2::new(p.pos.x + 8.0, p.pos.y + 8.0);
                if want.len_sq() > 0.01 {
                    let prev = p.facing;
                    p.facing = Dir4::from_vec(want, prev);
                    if p.facing != prev {
                        facing_snap = true;
                        dust_pos = Some(center);
                    }
                }
                // Walk anim
                if p.vel.len_sq() > 0.01 {
                    pd.walk_timer = pd.walk_timer.wrapping_add(1);
                    if pd.walk_timer.is_multiple_of(8) {
                        p.anim.frame = (p.anim.frame + 1) % 4;
                    }
                } else {
                    p.anim.frame = 0;
                }
            }
        }
    }

    if facing_snap {
        if let Some(pos) = dust_pos {
            world.push_event(WorldEvent::FxRequest(FxKind::Dust { pos }));
        }
    }

    let prev_hop = matches!(
        world.get(pid).map(|p| &p.data),
        Some(EntityData::Player(pd)) if matches!(pd.state, PlayerState::LedgeHop { .. })
    );

    integrate_entity(world, pid);

    if !prev_hop {
        if let Some(EntityData::Player(pd)) = world.get(pid).map(|p| &p.data) {
            if matches!(pd.state, PlayerState::LedgeHop { tick: 0, .. }) {
                if let Some(p) = world.get(pid) {
                    world.push_event(WorldEvent::FxRequest(FxKind::Dust { pos: p.center() }));
                }
                world.push_event(WorldEvent::Sfx(SfxId::LedgeHop));
            }
        }
    }
}

fn blend_move(pd: &mut crate::world::entity::PlayerData, vel: &mut Vec2, want: Vec2, top: f32) {
    let target = want.scale(top);
    if want.len_sq() > 0.01 {
        pd.move_blend = (pd.move_blend + 1.0 / tuning::ACCEL_TICKS).min(1.0);
    } else {
        pd.move_blend = (pd.move_blend - 1.0 / tuning::DECEL_TICKS).max(0.0);
    }
    // Vector lerp toward target using blend as accel feel
    let rate = if want.len_sq() > 0.01 {
        1.0 / tuning::ACCEL_TICKS
    } else {
        1.0 / tuning::DECEL_TICKS
    };
    vel.x += (target.x - vel.x) * rate.min(1.0);
    vel.y += (target.y - vel.y) * rate.min(1.0);
    if want.len_sq() < 0.01 && vel.len() < 0.05 {
        *vel = Vec2::ZERO;
    }
}

fn integrate_entity(world: &mut World, id: EntityId) {
    // Extract entity, move, write back — arena slot approach
    let slot = id.index as usize;
    if slot >= world.arena.len() || world.arena[slot].gen != id.gen {
        return;
    }
    let mut entity = match world.arena[slot].entity.take() {
        Some(e) => e,
        None => return,
    };
    physics::move_entity(world, &mut entity);
    world.arena[slot].entity = Some(entity);
}

fn tick_player_meters(world: &mut World, pid: EntityId) {
    let mut style_ev = Vec::new();
    {
        let Some(p) = world.get_mut(pid) else {
            return;
        };
        if let Some(h) = p.health.as_mut() {
            if h.iframes > 0 {
                h.iframes -= 1;
            }
            if h.flash > 0 {
                h.flash -= 1;
            }
        }
        if let EntityData::Player(pd) = &mut p.data {
            energy::tick(pd);
            style_ev = style::tick(pd);
            if let Some(h) = p.health {
                pd.hearts = h.hp;
            }
        }
    }
    for ev in style_ev {
        world.push_event(ev);
    }
}

fn check_dash_through(world: &mut World, pid: EntityId) {
    let (dashing, awarded, center, iframes) = {
        let Some(p) = world.get(pid) else {
            return;
        };
        let EntityData::Player(pd) = &p.data else {
            return;
        };
        let dashing = matches!(
            pd.state,
            PlayerState::Dash { tick }
                if (tuning::DASH_IFRAME_START..=tuning::DASH_IFRAME_END).contains(&tick)
        );
        (
            dashing,
            pd.dash_through_awarded,
            p.center(),
            p.health.map(|h| h.iframes).unwrap_or(0),
        )
    };
    if !dashing || awarded || iframes == 0 {
        return;
    }
    let hits =
        physics::query_aabb(world, center, Vec2::new(8.0, 8.0), crate::world::entity::layer::ENEMY_BODY);
    if hits.is_empty() {
        return;
    }
    {
        let Some(p) = world.get_mut(pid) else {
            return;
        };
        if let EntityData::Player(pd) = &mut p.data {
            pd.dash_through_awarded = true;
        }
    }
    world.push_event(WorldEvent::StyleAction(StyleVerb::DashThrough));
}

fn check_fountain(world: &mut World, pid: EntityId) {
    let pcenter = match world.get(pid) {
        Some(p) => p.center(),
        None => return,
    };
    let mut inside = false;
    let mut fpos = Vec2::ZERO;
    for (_id, e) in world.iter_alive() {
        if e.kind != EntityKind::FairyFountain {
            continue;
        }
        let Some(b) = e.body else {
            continue;
        };
        let c = e.center();
        if (c.x - pcenter.x).abs() <= b.half.x + 8.0 && (c.y - pcenter.y).abs() <= b.half.y + 8.0
        {
            inside = true;
            fpos = c;
            break;
        }
    }
    if !inside {
        return;
    }

    let tick = world.tick;
    let mut chime = false;
    {
        let Some(p) = world.get_mut(pid) else {
            return;
        };
        if let EntityData::Player(pd) = &mut p.data {
            let before = pd.energy;
            energy::fill(pd, tuning::FOUNTAIN_ENERGY_PER_TICK);
            if pd.energy > before {
                chime = true;
            }
            if tick.is_multiple_of(u64::from(tuning::FOUNTAIN_HEART_INTERVAL)) {
                if let Some(h) = p.health.as_mut() {
                    if h.hp < h.max {
                        h.hp += 1;
                        pd.hearts = h.hp;
                        chime = true;
                    }
                }
            }
        }
    }
    world.push_event(WorldEvent::FxRequest(FxKind::FountainSparkle { pos: fpos }));
    if chime && world.tick.is_multiple_of(20) {
        world.push_event(WorldEvent::Sfx(SfxId::FountainChime));
    }
}
