//! Emit `GameEvent::SetMusic` when the active track should change.

use content::audio::music::TrackId;
use content::maps::MapId;

use crate::boss::BossPhase;
use crate::state::GameMode;
use crate::Game;

pub fn desired_track(game: &Game) -> TrackId {
    if matches!(game.mode, GameMode::Title) {
        return TrackId::Title;
    }
    if game.ui.credits.active {
        return TrackId::Victory;
    }
    if let Some(boss) = &game.boss {
        match boss.phase {
            BossPhase::Intro | BossPhase::Fight => return TrackId::Boss,
            BossPhase::Victory => return TrackId::Victory,
        }
    }
    match game.current_map {
        MapId::Dungeon => TrackId::Dungeon,
        MapId::Arena => TrackId::Overworld,
        MapId::Overworld => {
            // Mosslight Village = region index 1 (see overworld region paint order).
            if game.music_region == 1 {
                TrackId::Village
            } else {
                TrackId::Overworld
            }
        }
        // Interiors / shrine lobby keep last overworld choice via music_region.
        MapId::House(_) | MapId::Shop | MapId::Cave(_) | MapId::ShrineLobby => {
            if game.music_region == 1 {
                TrackId::Village
            } else {
                TrackId::Overworld
            }
        }
    }
}

pub fn sync(game: &mut Game, outbound: &mut Vec<crate::GameEvent>) {
    let want = desired_track(game);
    if game.music_playing != Some(want) {
        game.music_playing = Some(want);
        outbound.push(crate::GameEvent::SetMusic(want));
    }
}
