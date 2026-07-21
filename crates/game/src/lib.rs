//! Game facade: fixed 60 Hz update + render. Phase 2A overworld foundation.

mod assets;
mod combat;
mod debug;
mod draw_world;
mod enemies;
mod fx;
mod interact;
mod items;
mod math;
mod player;
mod puzzle;
mod save_data;
mod state;
mod ui;
mod world;

pub use assets::{bake as bake_assets, BakedAssets, SpriteMap};
pub use content::audio::sfx::SfxId;
pub use save_data::{SaveGame, SAVE_KEY};

use content::maps::{self, MapId, TILE_PX};
use engine::chunks::ChunkCache;
use engine::input::{InputState, DEBUG_ACTION, DEBUG_MAP, DEBUG_OVERLAY, DEBUG_TELEPORT};
use engine::render::Draw;

use crate::combat::style;
use crate::draw_world::{MapRenderStats, TileSprites};
use crate::enemies::WaveDirector;
use crate::fx::{FxKind, FxState};
use crate::math::{Dir4, Vec2};
use crate::puzzle::PuzzleState;
use crate::state::{fade_alpha, save_from_game, GameMode};
use crate::ui::UiState;
use crate::world::entity::{EntityData, EntityKind};
use crate::world::physics;
use crate::world::{Spawner, World, WorldEvent};

const SAVE_INTERVAL_TICKS: u32 = 60;

#[derive(Clone, Debug)]
pub enum GameEvent {
    Sfx(SfxId),
    Save(String),
}

pub struct Game {
    pub(crate) world: World,
    pub(crate) spawner: Spawner,
    pub(crate) current_map: MapId,
    pub(crate) mode: GameMode,
    pub(crate) gems: u8,
    pub(crate) flags: Vec<u16>,
    pub(crate) chunk_cache_reset: bool,
    fx: FxState,
    pub(crate) ui: UiState,
    sprites: SpriteMap,
    tile_sprites: TileSprites,
    chunk_cache: Option<ChunkCache>,
    waves: WaveDirector,
    ticks: u32,
    touch_active: bool,
    touch_overlay: engine::input::TouchOverlay,
    map_stats: MapRenderStats,
    teleport_idx: usize,
    pub(crate) door_cooldown: u8,
    pub(crate) pending_save: Option<String>,
    pub(crate) puzzle: PuzzleState,
}

impl Game {
    pub fn new(save: SaveGame, sprites: SpriteMap) -> Self {
        let tile_sprites = TileSprites::build(&sprites);
        let map_id = save.map_id();
        // Boot overworld at checkpoint entry (New Game = entry 0).
        let map_id = if map_id == MapId::Arena {
            MapId::Arena
        } else {
            MapId::Overworld
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
        let mut spawner = Spawner::populate(&mut world);
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

        Self {
            world,
            spawner,
            current_map: map_id,
            mode: GameMode::Play,
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
        }
    }

    pub fn from_storage_json(json: Option<String>, sprites: SpriteMap) -> Self {
        let save = match json {
            Some(s) => SaveGame::from_json(&s),
            None => SaveGame::default_spawn(),
        };
        Self::new(save, sprites)
    }

    pub fn update(&mut self, input: &InputState) -> Vec<GameEvent> {
        self.touch_active = input.touch_active;
        self.touch_overlay = input.touch_overlay.clone();

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
            return self.drain_events(input);
        }

        // Dialog / pause-map / shop pause world sim (viewer pattern).
        self.ui.minimap.update(
            input,
            &self.world,
            self.current_map,
            self.gems,
            &self.flags,
        );
        if self.ui.dialog.open || self.ui.minimap.pause_open || self.ui.shop.open {
            if self.ui.dialog.open {
                self.ui.dialog.update(input, &mut self.world);
            }
            if self.ui.shop.open {
                if let Some(json) = ui::shop::update(self, input) {
                    self.pending_save = Some(json);
                }
            }
            return self.drain_events(input);
        }

        self.world.tick = self.world.tick.wrapping_add(1);
        tick_entity_timers(&mut self.world);
        combat::tick_dummies(&mut self.world);
        if self.door_cooldown > 0 {
            self.door_cooldown -= 1;
        }

        if self.world.hitstop > 0 {
            self.world.hitstop -= 1;
            fx::update(&mut self.world, &mut self.fx);
            return self.drain_events(input);
        }

        if self.ui.debug_overlay && input.debug[DEBUG_ACTION].pressed {
            debug::spawn_debug_shot(&mut self.world);
            // Temporary bomb grant for M3 testing (behind F1).
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
        }

        player::update(&mut self.world, input);
        if let Some(json) = interact::update(self, input) {
            self.pending_save = Some(json);
        }
        if self.ui.dialog.open || self.ui.shop.open {
            return self.drain_events(input);
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
        fx::update(&mut self.world, &mut self.fx);
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

        self.drain_events(input)
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

    fn drain_events(&mut self, _input: &InputState) -> Vec<GameEvent> {
        let raw = std::mem::take(&mut self.world.events);
        let rest = combat::route_combat_events(&mut self.world, raw);
        let mut rest = rest;
        rest.extend(std::mem::take(&mut self.world.events));

        let mut outbound = Vec::new();
        let mut sfx_count = 0u8;

        for ev in rest {
            match ev {
                WorldEvent::FxRequest(kind) => {
                    self.fx.handle(kind, &mut self.world.rng);
                }
                WorldEvent::Sfx(id) => {
                    if sfx_count < 8 {
                        outbound.push(GameEvent::Sfx(id));
                        sfx_count += 1;
                    }
                }
                WorldEvent::StyleAction(verb) => {
                    let pid = self.world.player_id;
                    let mut follow = Vec::new();
                    if let Some(p) = self.world.get_mut(pid) {
                        if let EntityData::Player(pd) = &mut p.data {
                            follow = style::apply_verb(pd, verb);
                        }
                    }
                    for fev in follow {
                        match fev {
                            WorldEvent::FxRequest(k) => self.fx.handle(k, &mut self.world.rng),
                            WorldEvent::Sfx(id) if sfx_count < 8 => {
                                outbound.push(GameEvent::Sfx(id));
                                sfx_count += 1;
                            }
                            other => self.world.events.push(other),
                        }
                    }
                }
                WorldEvent::EnergyDenied => {}
                WorldEvent::Killed { kind: _kind, pos } => {
                    self.fx.handle(FxKind::KillPoof { pos }, &mut self.world.rng);
                    if sfx_count < 8 {
                        outbound.push(GameEvent::Sfx(SfxId::Kill));
                        sfx_count += 1;
                    }
                    items::pickups::spawn_drops(&mut self.world, pos);
                }
                WorldEvent::AttackHit { .. } | WorldEvent::DamagedPlayer { .. } => {}
                WorldEvent::RegionEntered(region) => {
                    if let Some(r) = self.world.map.regions.get(region as usize) {
                        self.ui
                            .banner
                            .on_region(region, r.name, self.world.tick);
                    }
                }
                WorldEvent::GroupCleared(group) => {
                    if group == content::flags::GRP_CAMP_GUARD {
                        crate::save_data::set_flag(
                            &mut self.flags,
                            content::flags::GROUP_CAMP_GUARD,
                        );
                        self.fx.handle(
                            FxKind::Toast {
                                text: "GUARDS CLEARED",
                            },
                            &mut self.world.rng,
                        );
                    }
                }
            }
        }

        if let Some(json) = self.pending_save.take() {
            outbound.push(GameEvent::Save(json));
        }

        self.ticks = self.ticks.wrapping_add(1);
        self.ui.fps_accum = self.ui.fps_accum.wrapping_add(1);
        if self.ticks.is_multiple_of(SAVE_INTERVAL_TICKS) {
            let save = save_from_game(self);
            if let Some(json) = save.to_json() {
                outbound.push(GameEvent::Save(json));
            }
            self.ui.fps_est = self.ui.renders as f32;
            self.ui.renders = 0;
        }

        outbound
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
        for id in ids {
            if let Some(e) = self.world.get(id) {
                draw_world::render_entity(d, e, &self.sprites);
            }
        }

        draw_world::render_overhang(d, &self.world, &self.tile_sprites, &self.chunk_cache);

        self.fx.render_world(d);

        if let Some(mark) = interact::prompt_target(&self.world) {
            d.text("!", mark.x, mark.y, "#ffe040");
        }

        d.set_offset(0.0, 0.0);
        ui::render_hud(d, &self.world, &self.sprites);
        self.fx.render_screen(d, &self.sprites);
        self.ui.banner.render(d);
        self.ui.minimap.render_corner(
            d,
            &self.world,
            &self.sprites,
            self.current_map,
            self.gems,
            &self.flags,
        );
        self.ui.dialog.render(d, &self.sprites);
        ui::shop::render(d, &self.ui.shop, &self.flags);
        self.ui.minimap.render_pause(
            d,
            &self.world,
            &self.sprites,
            self.current_map,
            self.gems,
            &self.flags,
        );

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
            let o = &self.touch_overlay;
            if let Some((ox, oy)) = o.joystick_origin {
                d.circle(
                    ox,
                    oy,
                    engine::input::JOYSTICK_MAX_RADIUS,
                    "rgba(255,255,255,0.25)",
                );
            }
            if let Some((kx, ky)) = o.joystick_knob {
                d.circle(kx, ky, 8.0, "rgba(255,255,255,0.45)");
            }
            d.circle(
                o.attack_pos.0,
                o.attack_pos.1,
                o.button_radius,
                "rgba(255,80,80,0.35)",
            );
            d.circle(
                o.dash_pos.0,
                o.dash_pos.1,
                o.button_radius,
                "rgba(80,160,255,0.35)",
            );
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
            | EntityKind::Sign
            | EntityKind::Npc
            | EntityKind::Chest
            | EntityKind::Gem
            | EntityKind::Bomb => {}
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
