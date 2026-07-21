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

use crate::world::entity::EntityKind;
use crate::world::World;

pub use waves::WaveDirector;

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
            | EntityKind::Gem
            | EntityKind::Bomb => {}
        }
    }
    octorok::update_rocks(world);
    raider::update_projectiles(world);
}
