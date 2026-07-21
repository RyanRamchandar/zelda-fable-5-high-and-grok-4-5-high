//! Enemy AI: slime / bat / octorok + arena wave director.

mod ai;
pub(crate) mod bat;
pub(crate) mod octorok;
pub(crate) mod slime;
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
        if matches!(kind, EntityKind::Slime | EntityKind::Bat | EntityKind::Octorok)
            && crate::world::spawner::enemy_should_sleep(world, pos)
        {
            continue;
        }
        match kind {
            EntityKind::Slime => slime::update_one(world, id),
            EntityKind::Bat => bat::update_one(world, id),
            EntityKind::Octorok => octorok::update_one(world, id),
            EntityKind::OctorokRock
            | EntityKind::Player
            | EntityKind::Dummy
            | EntityKind::Pickup
            | EntityKind::SwordBeam
            | EntityKind::FairyFountain
            | EntityKind::DebugShot
            | EntityKind::Sign
            | EntityKind::Npc
            | EntityKind::Chest
            | EntityKind::Gem => {}
        }
    }
    octorok::update_rocks(world);
}
