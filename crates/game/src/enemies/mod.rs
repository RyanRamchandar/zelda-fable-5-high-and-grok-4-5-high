//! Enemy AI: slime / bat / octorok + arena wave director.

mod ai;
mod bat;
mod octorok;
mod slime;
mod waves;

use engine::input::InputState;

use crate::world::entity::EntityKind;
use crate::world::World;

pub use waves::WaveDirector;

pub fn update(world: &mut World, _input: &InputState, waves: &mut WaveDirector) {
    waves.update(world);

    let ids = world.alive_ids();
    for id in ids {
        let kind = match world.get(id).map(|e| e.kind) {
            Some(k) => k,
            None => continue,
        };
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
            | EntityKind::DebugShot => {}
        }
    }
    octorok::update_rocks(world);
}
