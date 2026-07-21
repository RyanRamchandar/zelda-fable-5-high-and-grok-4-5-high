//! Enemy AI: slime / bat / octorok / raiders / wisp / skeleton + arena waves.

mod ai;
pub(crate) mod bat;
pub(crate) mod octorok;
pub(crate) mod raider;
pub(crate) mod skeleton;
pub(crate) mod slime;
pub(crate) mod wisp;
mod waves;

use engine::input::InputState;

use crate::fx::FxKind;
use crate::world::entity::{EntityData, EntityId, EntityKind, WispState};
use crate::world::{World, WorldEvent};
use content::audio::sfx::SfxId;

pub use waves::WaveDirector;

/// Generic stun: zero velocity and freeze AI for `ticks` (boomerang).
pub fn stun(world: &mut World, id: EntityId, ticks: u16) {
    let Some(e) = world.get_mut(id) else {
        return;
    };
    // Phased wisp immune.
    if let EntityData::Wisp(d) = &e.data {
        if matches!(d.state, WispState::Phased | WispState::FadeOut) {
            return;
        }
    }
    e.vel = crate::math::Vec2::ZERO;
    let pos = e.center();
    match &mut e.data {
        EntityData::Slime(d) => d.stun_ticks = ticks,
        EntityData::Bat(d) => d.stun_ticks = ticks,
        EntityData::Octorok(d) => d.stun_ticks = ticks,
        EntityData::RaiderSpear(d) => d.stun_ticks = ticks,
        EntityData::RaiderTorch(d) => d.stun_ticks = ticks,
        EntityData::Wisp(d) => d.stun_ticks = ticks,
        EntityData::Skeleton(_) => {
            // Prefer stagger hook.
        }
        _ => {}
    }
    world.push_event(WorldEvent::FxRequest(FxKind::Impact { pos }));
    world.push_event(WorldEvent::Sfx(SfxId::SkeletonRattle));
}

pub fn update(world: &mut World, input: &InputState, waves: &mut WaveDirector) {
    waves.update(world);
    update_enemies(world, input);
}

pub fn update_no_waves(world: &mut World, input: &InputState) {
    update_enemies(world, input);
}

fn update_enemies(world: &mut World, _input: &InputState) {
    let ids = world.alive_ids();
    for id in ids {
        let (kind, pos) = match world.get(id).map(|e| (e.kind, e.pos)) {
            Some(v) => v,
            None => continue,
        };
        if matches!(
            kind,
            EntityKind::Slime
                | EntityKind::Bat
                | EntityKind::Octorok
                | EntityKind::RaiderSpear
                | EntityKind::RaiderTorch
                | EntityKind::Wisp
                | EntityKind::Skeleton
        ) && crate::world::spawner::enemy_should_sleep(world, pos)
        {
            continue;
        }
        match kind {
            EntityKind::Slime => slime::update_one(world, id),
            EntityKind::Bat => bat::update_one(world, id),
            EntityKind::Octorok => octorok::update_one(world, id),
            EntityKind::RaiderSpear => raider::update_spear(world, id),
            EntityKind::RaiderTorch => raider::update_torch(world, id),
            EntityKind::Wisp => wisp::update_one(world, id),
            EntityKind::Skeleton => skeleton::update_one(world, id),
            EntityKind::OctorokRock
            | EntityKind::TorchProj
            | EntityKind::TorchFlame
            | EntityKind::Player
            | EntityKind::Dummy
            | EntityKind::Pickup
            | EntityKind::SwordBeam
            | EntityKind::FairyFountain
            | EntityKind::DebugShot
            | EntityKind::Sign
            | EntityKind::Npc
            | EntityKind::Chest
            |             EntityKind::Gem
            | EntityKind::Bomb
            | EntityKind::Boomerang => {}
        }
    }
    octorok::update_rocks(world);
    raider::update_projectiles(world);
}
