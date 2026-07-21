//! World-event drain (extracted from `lib.rs` for file-cap headroom).

use content::audio::sfx::SfxId;
use engine::input::InputState;

use crate::combat::style;
use crate::fx::FxKind;
use crate::items;
use crate::state::save_from_game;
use crate::world::entity::EntityData;
use crate::world::WorldEvent;
use crate::{Game, GameEvent};

const SAVE_INTERVAL_TICKS: u32 = 60;

pub(crate) fn drain(game: &mut Game, _input: &InputState) -> Vec<GameEvent> {
    let raw = std::mem::take(&mut game.world.events);
    let rest = crate::combat::route_combat_events(&mut game.world, raw);
    let mut rest = rest;
    rest.extend(std::mem::take(&mut game.world.events));

    let mut outbound = Vec::new();
    let mut sfx_count = 0u8;

    for ev in rest {
        match ev {
            WorldEvent::FxRequest(kind) => {
                game.fx.handle(kind, &mut game.world.rng);
            }
            WorldEvent::Sfx(id) => {
                if sfx_count < 8 {
                    outbound.push(GameEvent::Sfx(id));
                    sfx_count += 1;
                }
            }
            WorldEvent::StyleAction(verb) => {
                let pid = game.world.player_id;
                let mut follow = Vec::new();
                if let Some(p) = game.world.get_mut(pid) {
                    if let EntityData::Player(pd) = &mut p.data {
                        follow = style::apply_verb(pd, verb);
                    }
                }
                for fev in follow {
                    match fev {
                        WorldEvent::FxRequest(k) => game.fx.handle(k, &mut game.world.rng),
                        WorldEvent::Sfx(id) if sfx_count < 8 => {
                            outbound.push(GameEvent::Sfx(id));
                            sfx_count += 1;
                        }
                        other => game.world.events.push(other),
                    }
                }
            }
            WorldEvent::EnergyDenied => {}
            WorldEvent::Killed { kind: _kind, pos } => {
                game.fx
                    .handle(FxKind::KillPoof { pos }, &mut game.world.rng);
                if sfx_count < 8 {
                    outbound.push(GameEvent::Sfx(SfxId::Kill));
                    sfx_count += 1;
                }
                items::pickups::spawn_drops(&mut game.world, pos);
            }
            WorldEvent::AttackHit { .. } | WorldEvent::DamagedPlayer { .. } => {}
            WorldEvent::RegionEntered(region) => {
                if let Some(r) = game.world.map.regions.get(region as usize) {
                    game.ui.banner.on_region(region, r.name, game.world.tick);
                }
            }
            WorldEvent::GroupCleared(group) => {
                handle_group_cleared(game, group, &mut outbound, &mut sfx_count);
            }
        }
    }

    if let Some(json) = game.pending_save.take() {
        outbound.push(GameEvent::Save(json));
    }
    if let Some(muted) = game.pending_muted.take() {
        outbound.push(GameEvent::SetMuted(muted));
        game.muted_boot_sent = true;
    } else if !game.muted_boot_sent {
        outbound.push(GameEvent::SetMuted(game.settings.muted));
        game.muted_boot_sent = true;
    }

    game.ticks = game.ticks.wrapping_add(1);
    game.ui.fps_accum = game.ui.fps_accum.wrapping_add(1);
    if game.ticks.is_multiple_of(SAVE_INTERVAL_TICKS) {
        let save = save_from_game(game);
        if let Some(json) = save.to_json() {
            outbound.push(GameEvent::Save(json));
        }
        game.ui.fps_est = game.ui.renders as f32;
        game.ui.renders = 0;
    }

    outbound
}

fn handle_group_cleared(
    game: &mut Game,
    group: u16,
    outbound: &mut Vec<GameEvent>,
    sfx_count: &mut u8,
) {
    if let Some(&(_, next)) = content::flags::CAMP_WAVE_CHAIN
        .iter()
        .find(|(from, _)| *from == group)
    {
        game.spawner.unlock_and_activate(&mut game.world, next);
        let toast = if next == content::flags::GRP_CAMP_W2 {
            "WAVE 2!"
        } else {
            "WAVE 3!"
        };
        game.fx
            .handle(FxKind::Toast { text: toast }, &mut game.world.rng);
        if *sfx_count < 8 {
            outbound.push(GameEvent::Sfx(SfxId::WaveCue));
            *sfx_count += 1;
        }
    }
    if group == content::flags::GRP_CAMP_W3 {
        crate::save_data::set_flag(&mut game.flags, content::flags::GROUP_CAMP_GUARD);
        game.spawner.camp_war_won = true;
        game.spawner.locked_groups = vec![
            content::flags::GRP_CAMP_W2,
            content::flags::GRP_CAMP_W3,
        ];
        game.fx.handle(
            FxKind::Toast {
                text: "GUARDS CLEARED",
            },
            &mut game.world.rng,
        );
        let save = save_from_game(game);
        if let Some(json) = save.to_json() {
            game.pending_save = Some(json);
        }
    }
    // Dungeon combat shutters (Trials / Currents).
    crate::puzzle::dungeon::on_group_cleared(game, group);
}
