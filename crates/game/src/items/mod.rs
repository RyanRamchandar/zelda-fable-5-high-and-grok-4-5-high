//! Items and pickups.

pub mod bombs;
pub mod boomerang;
pub mod pickups;

use crate::Game;

pub fn update(game: &mut Game) {
    pickups::update(&mut game.world);
    bombs::update(game);
    boomerang::update(game);
}
