//! Game mode + map transitions + trigger handling.

use content::audio::sfx::SfxId;
use content::maps::{self, MapId, TileLayer, TriggerKind, TILE_PX};

use crate::fx::FxKind;
use crate::math::Vec2;
use crate::save_data::{has_flag, set_flag, save_flags, SaveGame};
use crate::world::entity::{EntityData, PlayerData, PlayerState};
use crate::world::{Spawner, World, WorldEvent};
use crate::Game;

pub enum GameMode {
    Play,
    Transition(Transition),
}

pub struct Transition {
    #[allow(dead_code)]
    pub kind: Fade,
    pub t: u8,
    pub target: MapId,
    pub entry: u8,
}

pub enum Fade {
    Fade,
}

pub const FADE_TICKS: u8 = 16;

#[derive(Clone, Debug)]
pub struct PlayerPersist {
    pub hearts: i32,
    pub max_hearts: i32,
    pub energy: f32,
    pub rupees: u32,
    pub style_points: f32,
    pub style_rank: u8,
    pub gems: u8,
    pub flags: Vec<u16>,
    pub checkpoint: u8,
    pub bombs: u8,
    pub bomb_cap: u8,
    pub selected_item: u8,
}

impl PlayerPersist {
    pub fn from_player(pd: &PlayerData, checkpoint: u8, gems: u8, flags: Vec<u16>) -> Self {
        Self {
            hearts: pd.hearts,
            max_hearts: pd.max_hearts,
            energy: pd.energy,
            rupees: pd.rupees,
            style_points: pd.style_points,
            style_rank: pd.style_rank,
            gems,
            flags,
            checkpoint,
            bombs: pd.bombs,
            bomb_cap: pd.bomb_cap,
            selected_item: pd.selected_item,
        }
    }

    pub fn apply(&self, pd: &mut PlayerData) {
        pd.hearts = self.hearts;
        pd.max_hearts = self.max_hearts;
        pd.energy = self.energy;
        pd.rupees = self.rupees;
        pd.style_points = self.style_points;
        pd.style_rank = self.style_rank;
        pd.bombs = self.bombs;
        pd.bomb_cap = self.bomb_cap;
        pd.selected_item = self.selected_item;
    }
}

pub fn switch_map(game: &mut Game, target: MapId, entry: u8) {
    let persist = extract_persist(game);
    let map = maps::build(target);
    let (tx, ty) = map.entry_pos(entry).unwrap_or_else(|| {
        map.entries
            .first()
            .map(|e| (e.tx, e.ty))
            .unwrap_or((2, 2))
    });
    let spawn = Vec2::new(tx as f32 * TILE_PX, ty as f32 * TILE_PX);
    let mut world = World::new(target, map, spawn);
    world.checkpoint = persist.checkpoint;
    apply_persist(&mut world, &persist);
    let mut spawner = Spawner::populate(&mut world, &persist.flags);
    spawner.apply_save(&mut world, persist.gems, &persist.flags);
    crate::puzzle::paint_closed(&mut world, target, &persist.flags);
    if target == MapId::Overworld {
        restore_shrine_door(&mut world, &persist.flags);
        crate::puzzle::restore(&mut world, &persist.flags);
        crate::puzzle::chimes::apply_courage_seal_from_flags(&mut world, &persist.flags);
        game.ui.minimap.build_class_map(&world.map);
    } else {
        crate::puzzle::restore(&mut world, &persist.flags);
    }
    world.camera.snap_to(spawn.add(Vec2::new(8.0, 8.0)));
    game.world = world;
    game.spawner = spawner;
    game.current_map = target;
    game.gems = persist.gems;
    game.flags = persist.flags;
    game.puzzle = crate::puzzle::PuzzleState::for_map(target);
    game.chunk_cache_reset = true;
    game.dungeon_puzzle = None;
    if target == MapId::Dungeon {
        // Always lock sanctum miniboss group this phase.
        if !game
            .spawner
            .locked_groups
            .contains(&content::flags::GRP_DNG_SANCTUM)
        {
            game.spawner
                .locked_groups
                .push(content::flags::GRP_DNG_SANCTUM);
        }
        crate::rooms::on_enter_dungeon(game);
    } else {
        crate::rooms::clear(game);
    }
}

fn extract_persist(game: &Game) -> PlayerPersist {
    let checkpoint = game.world.checkpoint;
    let gems = game.gems;
    let flags = game.flags.clone();
    if let Some(p) = game.world.get(game.world.player_id) {
        if let EntityData::Player(pd) = &p.data {
            return PlayerPersist::from_player(pd, checkpoint, gems, flags);
        }
    }
    PlayerPersist {
        hearts: 6,
        max_hearts: 6,
        energy: 100.0,
        rupees: 0,
        style_points: 0.0,
        style_rank: 0,
        gems,
        flags,
        checkpoint,
        bombs: 0,
        bomb_cap: 0,
        selected_item: 0,
    }
}

fn apply_persist(world: &mut World, persist: &PlayerPersist) {
    let pid = world.player_id;
    if let Some(p) = world.get_mut(pid) {
        if let EntityData::Player(pd) = &mut p.data {
            persist.apply(pd);
        }
        if let Some(h) = p.health.as_mut() {
            h.hp = persist.hearts;
            h.max = persist.max_hearts;
        }
    }
}

pub fn begin_transition(game: &mut Game, target: MapId, entry: u8) {
    game.mode = GameMode::Transition(Transition {
        kind: Fade::Fade,
        t: 0,
        target,
        entry,
    });
}

pub fn tick_transition(game: &mut Game) {
    let GameMode::Transition(tr) = &mut game.mode else {
        return;
    };
    tr.t = tr.t.saturating_add(1);
    let t = tr.t;
    let target = tr.target;
    let entry = tr.entry;
    if t == FADE_TICKS {
        switch_map(game, target, entry);
        // Re-borrow after switch.
        if let GameMode::Transition(tr) = &mut game.mode {
            tr.t = FADE_TICKS;
        }
    }
    if t >= FADE_TICKS * 2 {
        game.mode = GameMode::Play;
    }
}

pub fn fade_alpha(mode: &GameMode) -> f32 {
    let GameMode::Transition(tr) = mode else {
        return 0.0;
    };
    let t = tr.t as f32;
    let half = FADE_TICKS as f32;
    if t <= half {
        t / half
    } else {
        1.0 - (t - half) / half
    }
}

pub fn save_from_game(game: &Game) -> SaveGame {
    let (hearts, max_hearts, rupees, bombs, bomb_cap, selected_item) = game
        .world
        .get(game.world.player_id)
        .and_then(|p| match &p.data {
            EntityData::Player(pd) => Some((
                pd.hearts,
                pd.max_hearts,
                pd.rupees,
                pd.bombs,
                pd.bomb_cap,
                pd.selected_item,
            )),
            _ => None,
        })
        .unwrap_or((6, 6, 0, 0, 0, 0));
    SaveGame {
        version: crate::save_data::SAVE_VERSION,
        map: game.current_map.to_u8(),
        entry: game.world.checkpoint,
        checkpoint: game.world.checkpoint,
        hearts,
        max_hearts,
        rupees,
        gems: game.gems,
        flags: game.flags.clone(),
        fog: game.ui.minimap.fog_bits(),
        bombs,
        bomb_cap,
        selected_item,
    }
}

pub fn restore_shrine_door(world: &mut World, flags: &[u16]) {
    use crate::save_data::{has_flag, save_flags};
    use content::maps::catalog;
    if !has_flag(flags, save_flags::DOOR_SHRINE_OPEN) {
        return;
    }
    world.set_tile(TileLayer::Ground, 120, 10, catalog::T_CAVE_MOUTH);
    world.set_tile(TileLayer::Ground, 119, 10, catalog::T_PATH);
    world.set_tile(TileLayer::Ground, 121, 10, catalog::T_PATH);
    if !world.map.triggers.iter().any(|t| {
        matches!(
            t.kind,
            TriggerKind::Door {
                target: MapId::ShrineLobby,
                ..
            }
        )
    }) {
        world.map.triggers.push(content::maps::TriggerDef {
            tx: 120,
            ty: 10,
            w: 1,
            h: 1,
            kind: TriggerKind::Door {
                target: MapId::ShrineLobby,
                entry: 0,
            },
        });
    }
}

pub fn check_triggers(game: &mut Game) -> Option<String> {
    let feet = {
        let p = game.world.get(game.world.player_id)?;
        let c = p.center();
        (
            (c.x / TILE_PX).floor() as i32,
            ((c.y + 6.0) / TILE_PX).floor() as i32,
        )
    };
    let triggers = game.world.map.triggers.clone();
    let mut immediate_save = None;
    for tr in triggers {
        let inside = feet.0 >= tr.tx as i32
            && feet.1 >= tr.ty as i32
            && feet.0 < (tr.tx + tr.w) as i32
            && feet.1 < (tr.ty + tr.h) as i32;
        if !inside {
            continue;
        }
        match tr.kind {
            TriggerKind::Door { target, entry } => {
                if game.door_cooldown == 0 {
                    game.door_cooldown = 40;
                    begin_transition(game, target, entry);
                    return None;
                }
            }
            TriggerKind::Banner { region } => {
                game.world
                    .push_event(WorldEvent::RegionEntered(region));
            }
            TriggerKind::Checkpoint { id } => {
                if game.world.checkpoint != id {
                    game.world.checkpoint = id;
                    game.world.push_event(WorldEvent::FxRequest(FxKind::Toast {
                        text: "CHECKPOINT",
                    }));
                    game.world
                        .push_event(WorldEvent::Sfx(SfxId::CheckpointChime));
                    immediate_save = save_from_game(game).to_json();
                }
            }
            TriggerKind::Secret { flag } => {
                if set_flag(&mut game.flags, flag) {
                    game.world
                        .push_event(WorldEvent::Sfx(SfxId::SecretChime));
                    game.world.push_event(WorldEvent::FxRequest(FxKind::Toast {
                        text: "SECRET!",
                    }));
                    if flag == save_flags::SECRET_MEADOW_FLOWERS {
                        // Fairy + energy refill.
                        if let Some(p) = game.world.get_mut(game.world.player_id) {
                            if let EntityData::Player(pd) = &mut p.data {
                                pd.energy = 100.0;
                            }
                        }
                        game.world.push_event(WorldEvent::FxRequest(FxKind::Toast {
                            text: "FAIRY!",
                        }));
                    }
                    game.ui.minimap.mark_discovered_secret();
                    immediate_save = save_from_game(game).to_json();
                }
            }
        }
    }
    let _ = has_flag;
    immediate_save
}

pub fn check_player_death(game: &mut Game) {
    let dead = game
        .world
        .get(game.world.player_id)
        .and_then(|p| p.health)
        .map(|h| h.hp <= 0)
        .unwrap_or(false);
    if !dead {
        return;
    }
    let cp = game.world.checkpoint;
    let map = game.current_map;
    if let Some(p) = game.world.get_mut(game.world.player_id) {
        if let Some(h) = p.health.as_mut() {
            h.hp = 6;
            h.max = h.max.max(6);
            h.iframes = 90;
            h.flash = 0;
        }
        if let EntityData::Player(pd) = &mut p.data {
            pd.hearts = 6;
            pd.energy = 100.0;
            pd.state = PlayerState::Idle;
        }
    }
    switch_map(game, map, cp);
    game.world.push_event(WorldEvent::FxRequest(FxKind::Toast {
        text: "FAIRY RESCUE",
    }));
}
