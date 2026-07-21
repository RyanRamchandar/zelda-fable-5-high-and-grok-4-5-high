//! Game facade: fixed 60 Hz update + render. Phase 2A overworld foundation.

mod assets;
mod boss;
mod combat;
mod debug;
mod draw_enemies;
mod draw_player;
mod draw_world;
mod enemies;
mod events;
mod fx;
mod interact;
mod items;
mod math;
mod music_director;
mod player;
mod puzzle;
mod rooms;
mod save_data;
mod state;
mod ui;
mod world;

pub use assets::{bake as bake_assets, BakedAssets, SpriteMap};
pub use content::audio::music::TrackId;
pub use content::audio::sfx::SfxId;
pub use save_data::{SaveGame, SAVE_KEY};

use content::maps::{self, MapId, TILE_PX};
use engine::chunks::ChunkCache;
use engine::input::{InputState, DEBUG_ACTION, DEBUG_MAP, DEBUG_OVERLAY, DEBUG_TELEPORT};
use engine::render::Draw;

use crate::draw_world::{MapRenderStats, TileSprites};
use crate::enemies::WaveDirector;
use crate::fx::FxState;
use crate::math::{Dir4, Vec2};
use crate::puzzle::PuzzleState;
use crate::state::{fade_alpha, GameMode};
use crate::ui::UiState;
use crate::world::entity::{EntityData, EntityKind};
use crate::world::physics;
use crate::world::{Spawner, World};

#[derive(Clone, Debug)]
pub enum GameEvent {
    Sfx(SfxId),
    Save(String),
    SetMuted(bool),
    SetMusic(TrackId),
}

#[derive(Clone, Debug, Default)]
pub struct Settings {
    pub muted: bool,
}

pub struct Game {
    pub(crate) world: World,
    pub(crate) spawner: Spawner,
    pub(crate) current_map: MapId,
    pub(crate) mode: GameMode,
    pub(crate) gems: u8,
    pub(crate) flags: Vec<u16>,
    pub(crate) chunk_cache_reset: bool,
    pub(crate) fx: FxState,
    pub(crate) ui: UiState,
    pub(crate) sprites: SpriteMap,
    tile_sprites: TileSprites,
    chunk_cache: Option<ChunkCache>,
    waves: WaveDirector,
    pub(crate) ticks: u32,
    touch_active: bool,
    touch_overlay: engine::input::TouchOverlay,
    map_stats: MapRenderStats,
    teleport_idx: usize,
    pub(crate) door_cooldown: u8,
    pub(crate) pending_save: Option<String>,
    pub(crate) puzzle: PuzzleState,
    pub(crate) rooms: Option<rooms::RoomsState>,
    pub(crate) dungeon_puzzle: Option<puzzle::dungeon::DungeonPuzzleState>,
    pub(crate) boss: Option<boss::BossState>,
    pub(crate) settings: Settings,
    pub(crate) had_save: bool,
    pub(crate) pending_muted: Option<bool>,
    pub(crate) open_title_after_transition: bool,
    muted_boot_sent: bool,
    pub(crate) last_input: InputState,
    /// Overworld region index for Village vs Overworld music (interiors inherit).
    pub(crate) music_region: u8,
    pub(crate) music_playing: Option<TrackId>,
}

impl Game {
    pub fn new(save: SaveGame, sprites: SpriteMap, boot_to_title: bool) -> Self {
        let tile_sprites = TileSprites::build(&sprites);
        let map_id = save.map_id();
        // Boot overworld at checkpoint entry (New Game = entry 0).
        let map_id = match map_id {
            MapId::Arena => MapId::Arena,
            MapId::Dungeon => MapId::Dungeon,
            _ => MapId::Overworld,
        };
        let entry = save.checkpoint;
        let map = maps::build(map_id);
        let (tx, ty) = map
            .entry_pos(entry)
            .or_else(|| map.entry_pos(0))
            .unwrap_or((118, 206));
        let spawn = Vec2::new(tx as f32 * TILE_PX, ty as f32 * TILE_PX);
        let mut world = World::new(map_id, map, spawn);
        world.checkpoint = save.checkpoint;
        if let Some(p) = world.get_mut(world.player_id) {
            if let EntityData::Player(pd) = &mut p.data {
                pd.hearts = save.hearts;
                pd.max_hearts = save.max_hearts;
                pd.rupees = save.rupees;
                pd.bombs = save.bombs;
                pd.bomb_cap = save.bomb_cap;
                pd.selected_item = save.selected_item;
            }
            if let Some(h) = p.health.as_mut() {
                h.hp = save.hearts;
                h.max = save.max_hearts;
            }
        }
        let mut spawner = Spawner::populate(&mut world, &save.flags);
        spawner.apply_save(&mut world, save.gems, &save.flags);
        puzzle::paint_closed(&mut world, map_id, &save.flags);
        state::restore_shrine_door(&mut world, &save.flags);
        puzzle::restore(&mut world, &save.flags);
        puzzle::chimes::apply_courage_seal_from_flags(&mut world, &save.flags);
        debug::debug_assert_door_entries(&world);

        let chunk_cache = ChunkCache::new(48).ok();
        let mut ui = UiState::new();
        ui.minimap.load_fog(&save.fog);
        ui.minimap.build_class_map(&world.map);
        ui.minimap.refresh_objective(save.gems, &save.flags);

        let mode = if boot_to_title {
            GameMode::Title
        } else {
            GameMode::Play
        };
        let mut game = Self {
            world,
            spawner,
            current_map: map_id,
            mode,
            gems: save.gems,
            flags: save.flags,
            chunk_cache_reset: true,
            fx: FxState::new(),
            ui,
            sprites,
            tile_sprites,
            chunk_cache,
            waves: WaveDirector::new(),
            ticks: 0,
            touch_active: false,
            touch_overlay: engine::input::TouchOverlay::default(),
            map_stats: MapRenderStats {
                chunks_ready: 0,
                chunks_budget: 0,
                bakes: 0,
                direct: false,
            },
            teleport_idx: 0,
            door_cooldown: 0,
            pending_save: None,
            puzzle: PuzzleState::for_map(map_id),
            rooms: None,
            dungeon_puzzle: None,
            boss: None,
            settings: Settings {
                muted: save.muted,
            },
            had_save: false,
            pending_muted: Some(save.muted),
            open_title_after_transition: false,
            muted_boot_sent: false,
            last_input: InputState::default(),
            music_region: 1, // boot near village
            music_playing: None,
        };
        if map_id == MapId::Dungeon {
            rooms::on_enter_dungeon(&mut game);
        }
        if boot_to_title && ui::title::has_progress(&game) {
            game.ui.title.cursor = 0; // CONTINUE first when present
        }
        game
    }

    pub fn from_storage_json(json: Option<String>, sprites: SpriteMap) -> Self {
        let (save, had_save) = match json {
            Some(s) => match SaveGame::try_from_json(&s) {
                Some(sg) => (sg, true),
                None => (SaveGame::default_spawn(), false),
            },
            None => (SaveGame::default_spawn(), false),
        };
        let mut game = Self::new(save, sprites, true);
        game.had_save = had_save;
        if had_save && ui::title::has_progress(&game) {
            game.ui.title.cursor = 0;
        }
        game
    }

    pub fn update(&mut self, input: &InputState) -> Vec<GameEvent> {
        self.touch_active = input.touch_active;
        self.touch_overlay = input.touch_overlay.clone();
        self.last_input = input.clone();

        if input.debug[DEBUG_OVERLAY].pressed {
            self.ui.debug_overlay = !self.ui.debug_overlay;
        }

        self.ui.viewer.update(input);
        if self.ui.viewer.active {
            return Vec::new();
        }

        // Debug map cycle / teleport (overlay required for teleport).
        if self.ui.debug_overlay && input.debug[DEBUG_MAP].pressed {
            let target = match self.current_map {
                MapId::Overworld => MapId::Arena,
                _ => MapId::Overworld,
            };
            state::begin_transition(self, target, 0);
        }
        if self.ui.debug_overlay && input.debug[DEBUG_TELEPORT].pressed {
            self.cycle_teleport();
        }

        if matches!(self.mode, GameMode::Transition(_)) {
            state::tick_transition(self);
            self.ui.banner.update();
            return events::drain(self, input);
        }

        if matches!(self.mode, GameMode::Title) {
            ui::title::update(self, input);
            return events::drain(self, input);
        }

        // Portrait + touch: freeze play until rotated (title still reachable via quit).
        if input.touch_active && input.viewport_portrait {
            return events::drain(self, input);
        }

        // Corner minimap toggle via touch tap (KeyM path stays).
        if let Some(tap) = input.menu_tap {
            if ui::touch::hit_corner_minimap(tap) {
                self.ui.minimap.show_corner = !self.ui.minimap.show_corner;
            }
        }

        // Pause routing before dialog/shop early-out.
        if ui::pause::update(self, input) {
            return events::drain(self, input);
        }

        // Dialog / shop pause world sim (viewer pattern).
        self.ui.minimap.update(
            input,
            &self.world,
            self.current_map,
            self.gems,
            &self.flags,
        );
        if self.ui.dialog.open || self.ui.shop.open {
            if self.ui.dialog.open {
                self.ui.dialog.update(input, &mut self.world);
            }
            if self.ui.shop.open {
                if let Some(json) = ui::shop::update(self, input) {
                    self.pending_save = Some(json);
                }
            }
            // Victory may be waiting on dialog close.
            let _ = boss::update(self, input);
            return events::drain(self, input);
        }

        // Boss intro / victory / credits pause world sim.
        if boss::update(self, input) {
            self.ui.banner.update();
            return events::drain(self, input);
        }

        // Room slide pauses world sim (dialog pattern).
        if rooms::update(self) {
            self.ui.banner.update();
            return events::drain(self, input);
        }

        self.world.tick = self.world.tick.wrapping_add(1);
        tick_entity_timers(&mut self.world);
        combat::tick_dummies(&mut self.world);
        if self.door_cooldown > 0 {
            self.door_cooldown -= 1;
        }

        if self.world.hitstop > 0 {
            self.world.hitstop -= 1;
            fx::update(
                &mut self.world,
                &mut self.fx,
                self.current_map,
                self.map_stats.direct,
            );
            return events::drain(self, input);
        }

        if self.ui.debug_overlay && input.debug[DEBUG_ACTION].pressed {
            debug::spawn_debug_shot(&mut self.world);
            // Temporary bomb / boomerang grant (behind F1).
            if let Some(p) = self.world.get_mut(self.world.player_id) {
                if let EntityData::Player(pd) = &mut p.data {
                    if pd.bomb_cap == 0 {
                        pd.bomb_cap = 10;
                        pd.bombs = 10;
                        pd.selected_item = 1;
                    } else {
                        pd.bombs = pd.bombs.saturating_add(5).min(pd.bomb_cap);
                    }
                }
            }
            crate::save_data::set_flag(&mut self.flags, content::flags::ITEM_BOOMERANG);
            crate::save_data::set_flag(&mut self.flags, content::flags::DKEY_SMALL_1);
            crate::save_data::set_flag(&mut self.flags, content::flags::DKEY_SMALL_2);
            crate::save_data::set_flag(&mut self.flags, content::flags::DKEY_BOSS);
            if let Some(p) = self.world.get_mut(self.world.player_id) {
                if let EntityData::Player(pd) = &mut p.data {
                    pd.selected_item = 2;
                }
            }
        }

        player::update(&mut self.world, input, &self.flags);
        if let Some(json) = interact::update(self, input) {
            self.pending_save = Some(json);
        }
        if self.ui.dialog.open || self.ui.shop.open {
            return events::drain(self, input);
        }
        self.spawner.update(&mut self.world);
        if self.current_map == MapId::Arena {
            enemies::update(&mut self.world, input, &mut self.waves);
        } else {
            enemies::update_no_waves(&mut self.world, input);
        }
        integrate_non_player(&mut self.world);
        puzzle::update(self);
        combat::resolve_hits(&mut self.world);
        items::update(self);
        fx::update(
            &mut self.world,
            &mut self.fx,
            self.current_map,
            self.map_stats.direct,
        );
        self.ui.banner.update();
        if let Some(json) = state::check_triggers(self) {
            self.pending_save = Some(json);
        }
        state::check_player_death(self);

        let (target, facing) = self
            .world
            .get(self.world.player_id)
            .map(|p| (p.center(), p.facing))
            .unwrap_or((Vec2::ZERO, Dir4::Down));
        let map_w = self.world.map.width as f32 * TILE_PX;
        let map_h = self.world.map.height as f32 * TILE_PX;
        {
            let World {
                camera, rng, ..
            } = &mut self.world;
            camera.update(map_w, map_h, rng, target, facing);
        }

        events::drain(self, input)
    }

    fn cycle_teleport(&mut self) {
        let entries = self.world.map.entries.clone();
        if entries.is_empty() {
            return;
        }
        self.teleport_idx = (self.teleport_idx + 1) % entries.len();
        let e = entries[self.teleport_idx];
        let pid = self.world.player_id;
        if let Some(p) = self.world.get_mut(pid) {
            p.pos = Vec2::new(e.tx as f32 * TILE_PX, e.ty as f32 * TILE_PX);
            p.vel = Vec2::ZERO;
        }
        self.world.camera.snap_to(Vec2::new(
            e.tx as f32 * TILE_PX + 8.0,
            e.ty as f32 * TILE_PX + 8.0,
        ));
        self.door_cooldown = 30;
        self.chunk_cache_reset = true;
    }

    pub fn render(&mut self, d: &mut Draw) {
        self.ui.renders = self.ui.renders.wrapping_add(1);

        if self.ui.viewer.active {
            self.ui.viewer.render(d, &self.sprites);
            return;
        }

        d.clear("#12141a");

        let cam = self.world.camera.offset();
        d.set_offset(-cam.x, -cam.y);

        let prebake = self.chunk_cache_reset;
        self.chunk_cache_reset = false;
        draw_world::render_map(
            d,
            &mut self.world,
            &self.tile_sprites,
            &mut self.chunk_cache,
            prebake,
            &mut self.map_stats,
        );

        let mut ids = self.world.alive_ids();
        ids.sort_by(|a, b| {
            let ya = self.world.get(*a).map(|e| e.pos.y).unwrap_or(0.0);
            let yb = self.world.get(*b).map(|e| e.pos.y).unwrap_or(0.0);
            ya.total_cmp(&yb)
        });
        let tunic = crate::save_data::has_flag(&self.flags, content::flags::TUNIC_BOUGHT);
        for id in ids {
            if let Some(e) = self.world.get(id) {
                draw_world::render_entity(d, e, &self.sprites, tunic);
            }
        }

        draw_world::render_overhang(d, &self.world, &self.tile_sprites, &self.chunk_cache);

        self.fx.render_world(d);

        if let Some(mark) = interact::prompt_target(&self.world) {
            d.text("!", mark.x, mark.y, "#ffe040");
        }

        d.set_offset(0.0, 0.0);
        ui::render_hud(d, &self.world, &self.sprites, self.touch_active);
        boss::render_overlay(d, self);
        self.ui.credits.draw(d);
        if self.current_map == MapId::Dungeon {
            ui::hud::draw_dungeon_keys(
                d,
                rooms::small_keys_held(&self.flags),
                crate::save_data::has_flag(&self.flags, content::flags::DKEY_BOSS),
            );
            ui::dungeon_map::render_corner(d, self);
        }
        self.fx.render_screen(d, &self.sprites);
        self.ui.banner.render(d);
        if self.current_map != MapId::Dungeon {
            self.ui.minimap.render_corner(
                d,
                &self.world,
                &self.sprites,
                self.current_map,
                self.gems,
                &self.flags,
            );
        }
        self.ui.dialog.render(d, &self.sprites);
        ui::shop::render(d, &self.ui.shop, &self.flags);
        ui::pause::render(d, self);

        if matches!(self.mode, GameMode::Title) {
            ui::title::render(d, self);
        }

        let alpha = fade_alpha(&self.mode);
        if alpha > 0.01 {
            let a = (alpha * 255.0) as u8;
            d.rect(0.0, 0.0, 480.0, 270.0, &format!("rgba(0,0,0,{:.3})", alpha));
            let _ = a;
        }

        let state_str = debug::player_state_label(&self.world);
        ui::render_debug(
            d,
            &self.world,
            &self.ui,
            &self.fx,
            &state_str,
            &self.map_stats,
            self.current_map,
        );

        if self.touch_active {
            ui::touch::render(d, &self.touch_overlay, &self.world, &self.sprites);
        }
        if self.touch_active && self.last_input.viewport_portrait {
            ui::touch::render_portrait_hint(d);
        }
    }
}

fn tick_entity_timers(world: &mut World) {
    let ids = world.alive_ids();
    for id in ids {
        if let Some(e) = world.get_mut(id) {
            if let Some(h) = e.health.as_mut() {
                if h.flash > 0 {
                    h.flash -= 1;
                }
                if e.kind != EntityKind::Player && h.iframes > 0 {
                    h.iframes -= 1;
                }
            }
            e.anim.timer = e.anim.timer.wrapping_add(1);
        }
    }
}

fn integrate_non_player(world: &mut World) {
    let ids: Vec<_> = world
        .alive_ids()
        .into_iter()
        .filter(|id| *id != world.player_id)
        .collect();
    for id in ids {
        let slot = id.index as usize;
        if slot >= world.arena.len() || world.arena[slot].gen != id.gen {
            continue;
        }
        let mut entity = match world.arena[slot].entity.take() {
            Some(e) => e,
            None => continue,
        };
        match entity.kind {
            EntityKind::Dummy => physics::move_entity(world, &mut entity),
            EntityKind::Slime
            | EntityKind::Octorok
            | EntityKind::Bat
            | EntityKind::OctorokRock
            | EntityKind::RaiderSpear
            | EntityKind::RaiderTorch
            | EntityKind::Wisp
            | EntityKind::Skeleton
            | EntityKind::Ironshell
            | EntityKind::GraniteWarden
            | EntityKind::WindCrystal
            | EntityKind::PebbleCrawler
            | EntityKind::TorchProj
            | EntityKind::TorchFlame
            | EntityKind::Sign
            | EntityKind::Npc
            | EntityKind::Chest
            | EntityKind::Gem
            | EntityKind::Bomb
            | EntityKind::Boomerang => {}
            EntityKind::Pickup
            | EntityKind::SwordBeam
            | EntityKind::DebugShot
            | EntityKind::FairyFountain
            | EntityKind::Player => {
                physics::decay_knockback(&mut entity);
            }
        }
        world.arena[slot].entity = Some(entity);
    }
}
