//! Wisp: visible drift → phase out (untargetable) → reappear near player.

use content::audio::sfx::SfxId;

use crate::combat::tuning;
use crate::fx::FxKind;
use crate::math::{Dir4, Vec2};
use crate::world::entity::{
    layer, AnimState, Body, Entity, EntityData, EntityId, EntityKind, Health, WispData, WispState,
};
use crate::world::{World, WorldEvent};

use super::ai;

pub fn spawn(world: &mut World, pos: Vec2) -> EntityId {
    world.spawn(Entity {
        kind: EntityKind::Wisp,
        pos,
        vel: Vec2::ZERO,
        facing: Dir4::Down,
        body: None,
        health: Some(Health {
            hp: tuning::WISP_HP,
            max: tuning::WISP_HP,
            iframes: 0,
            flash: 0,
        }),
        knockback: Vec2::ZERO,
        anim: AnimState::default(),
        data: EntityData::Wisp(WispData {
            spawn_telegraph: tuning::SPAWN_TELEGRAPH,
            state: WispState::Visible,
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
            EntityData::Wisp(d) => d.spawn_telegraph,
            _ => return,
        }
    };
    if telegraph > 0 {
        tick_spawn(world, id);
        return;
    }

    let Some(ppos) = ai::player_pos(world) else {
        return;
    };
    let (state, timer, epos) = {
        let Some(e) = world.get(id) else {
            return;
        };
        let EntityData::Wisp(d) = &e.data else {
            return;
        };
        (d.state, d.timer, e.center())
    };

    let mut next_state = state;
    let mut next_timer = timer.saturating_add(1);
    let mut vel = Vec2::ZERO;
    let anim;
    let mut sfx = None;
    let mut contact = false;
    let mut teleport = None;
    let mut set_iframes = None;

    match state {
        WispState::Visible => {
            anim = (next_timer / 8) % 2;
            vel = ai::steer_toward(epos, ppos, tuning::WISP_DRIFT);
            contact = true;
            set_iframes = Some(0u16);
            if next_timer >= tuning::WISP_VISIBLE {
                next_state = WispState::FadeOut;
                next_timer = 0;
                sfx = Some(SfxId::WispPhase);
            }
        }
        WispState::FadeOut => {
            anim = 2;
            if next_timer >= tuning::WISP_FADE_TELE {
                next_state = WispState::Phased;
                next_timer = 0;
                set_iframes = Some(999);
            }
        }
        WispState::Phased => {
            anim = 2;
            set_iframes = Some(999);
            if next_timer >= tuning::WISP_PHASED {
                next_state = WispState::FadeIn;
                next_timer = 0;
                sfx = Some(SfxId::WispPhase);
                teleport = Some(reappear_near(ppos, world.tick, id.index));
            }
        }
        WispState::FadeIn => {
            anim = 1;
            if next_timer >= tuning::WISP_FADE_TELE {
                next_state = WispState::Visible;
                next_timer = 0;
                set_iframes = Some(0);
            }
        }
    }

    {
        let Some(e) = world.get_mut(id) else {
            return;
        };
        e.vel = vel;
        e.anim.frame = anim;
        e.anim.sheet = match next_state {
            WispState::Phased | WispState::FadeOut => 1,
            _ => 0,
        };
        if let Some(pos) = teleport {
            e.pos = pos;
        }
        if let Some(frames) = set_iframes {
            if let Some(h) = e.health.as_mut() {
                h.iframes = frames;
            }
        }
        if let EntityData::Wisp(d) = &mut e.data {
            d.state = next_state;
            d.timer = next_timer;
        }
        // Phased: strip body so attacks/contact miss.
        match next_state {
            WispState::Phased => e.body = None,
            WispState::Visible | WispState::FadeIn if e.body.is_none() => {
                e.body = Some(Body {
                    half: Vec2::new(8.0, 8.0),
                    solid: false,
                    layer: layer::ENEMY_BODY,
                    mask: layer::PLAYER_HIT,
                });
            }
            _ => {}
        }
    }
    ai::fly_enemy(world, id);

    if let Some(id_sfx) = sfx {
        world.push_event(WorldEvent::Sfx(id_sfx));
    }

    if contact {
        if let Some(e) = world.get(id) {
            if e.body.is_some() && ai::overlaps_player(world, e) {
                let dir = ai::player_pos(world)
                    .map(|p| p.sub(e.center()).normalize_or_zero())
                    .unwrap_or(Vec2::ZERO);
                world.push_event(WorldEvent::DamagedPlayer {
                    amount: tuning::WISP_CONTACT,
                    dir,
                    source: Some(id),
                });
                // Brief orange burn flash on player.
                if let Some(p) = world.get_mut(world.player_id) {
                    if let Some(h) = p.health.as_mut() {
                        h.flash = tuning::FLASH_TICKS.saturating_add(2);
                    }
                }
            }
        }
    }
}

fn reappear_near(ppos: Vec2, tick: u64, index: u32) -> Vec2 {
    let ang = (tick as f32) * 0.17 + index as f32 * 1.7;
    let radius = tuning::WISP_REAPPEAR_MIN
        + (tuning::WISP_REAPPEAR_MAX - tuning::WISP_REAPPEAR_MIN) * 0.55;
    Vec2::new(
        ppos.x + ang.cos() * radius - 8.0,
        ppos.y + ang.sin() * radius - 8.0,
    )
}

fn tick_spawn(world: &mut World, id: EntityId) {
    let left = {
        let Some(e) = world.get_mut(id) else {
            return;
        };
        let EntityData::Wisp(d) = &mut e.data else {
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
