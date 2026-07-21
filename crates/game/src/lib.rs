//! Game facade: fixed 60 Hz update + render. Phase 1A arena combat feel.

mod combat;
mod enemies;
mod fx;
mod items;
mod math;
mod player;
mod save_data;
mod ui;
mod world;

pub use content::audio::sfx::SfxId;
pub use save_data::{SaveGame, SAVE_KEY};

use content::maps::{self, FOUNTAIN, TILE_PX, WALL};
use engine::input::{InputState, DEBUG_ACTION, DEBUG_OVERLAY};
use engine::render::Draw;

use crate::combat::style;
use crate::fx::{FxKind, FxState};
use crate::math::{Dir4, Vec2};
use crate::ui::UiState;
use crate::world::entity::{
    layer, AnimState, BeamData, Body, Entity, EntityData, EntityKind, PlayerState,
};
use crate::world::physics;
use crate::world::{World, WorldEvent};

const SAVE_INTERVAL_TICKS: u32 = 60;

#[derive(Clone, Debug)]
pub enum GameEvent {
    Sfx(SfxId),
    Save(String),
}

pub struct Game {
    world: World,
    fx: FxState,
    ui: UiState,
    ticks: u32,
    touch_active: bool,
    touch_overlay: engine::input::TouchOverlay,
}

impl Game {
    pub fn new(save: SaveGame) -> Self {
        let map = maps::arena();
        let spawn = Vec2::new(save.x, save.y);
        let mut world = World::new(map, spawn);

        // Three target dummies
        let dummies = [
            Vec2::new(400.0, 240.0),
            Vec2::new(520.0, 260.0),
            Vec2::new(460.0, 320.0),
        ];
        for pos in dummies {
            world.spawn(Entity::dummy(pos));
        }

        // Fountain zone (NW corner tiles 4–5 → ~72,72)
        world.spawn(Entity::fountain(Vec2::new(72.0, 72.0)));

        Self {
            world,
            fx: FxState::new(),
            ui: UiState::new(),
            ticks: 0,
            touch_active: false,
            touch_overlay: engine::input::TouchOverlay::default(),
        }
    }

    pub fn from_storage_json(json: Option<String>) -> Self {
        let save = match json {
            Some(s) => SaveGame::from_json(&s),
            None => SaveGame::default_spawn(),
        };
        Self::new(save)
    }

    pub fn update(&mut self, input: &InputState) -> Vec<GameEvent> {
        self.touch_active = input.touch_active;
        self.touch_overlay = input.touch_overlay.clone();

        if input.debug[DEBUG_OVERLAY].pressed {
            self.ui.debug_overlay = !self.ui.debug_overlay;
        }

        // 1. tick timers
        self.world.tick = self.world.tick.wrapping_add(1);
        tick_entity_timers(&mut self.world);
        combat::tick_dummies(&mut self.world);

        // 2. hitstop freeze
        if self.world.hitstop > 0 {
            self.world.hitstop -= 1;
            fx::update(&mut self.world, &mut self.fx);
            return self.drain_events(input);
        }

        // Debug projectile (H while F1 overlay on)
        if self.ui.debug_overlay && input.debug[DEBUG_ACTION].pressed {
            spawn_debug_shot(&mut self.world);
        }

        // 3–9 systems
        player::update(&mut self.world, input);
        enemies::update(&mut self.world, input);
        integrate_non_player(&mut self.world);
        combat::resolve_hits(&mut self.world);
        items::update(&mut self.world);
        fx::update(&mut self.world, &mut self.fx);

        // Camera follow player
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

    fn drain_events(&mut self, _input: &InputState) -> Vec<GameEvent> {
        let raw = std::mem::take(&mut self.world.events);
        let rest = combat::route_combat_events(&mut self.world, raw);
        // apply_* may have pushed follow-up events
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
                WorldEvent::EnergyDenied => {
                    // flash already set on player; sfx may accompany separately
                }
                WorldEvent::Killed { kind: _kind, pos } => {
                    self.fx.handle(FxKind::KillPoof { pos }, &mut self.world.rng);
                    if sfx_count < 8 {
                        outbound.push(GameEvent::Sfx(SfxId::Kill));
                        sfx_count += 1;
                    }
                    items::pickups::spawn_drops(&mut self.world, pos);
                }
                WorldEvent::AttackHit { .. } | WorldEvent::DamagedPlayer { .. } => {
                    // already routed
                }
            }
        }

        // Periodic save
        self.ticks = self.ticks.wrapping_add(1);
        self.ui.fps_accum = self.ui.fps_accum.wrapping_add(1);
        if self.ticks.is_multiple_of(SAVE_INTERVAL_TICKS) {
            if let Some(p) = self.world.get(self.world.player_id) {
                let save = SaveGame {
                    x: p.pos.x,
                    y: p.pos.y,
                };
                if let Some(json) = save.to_json() {
                    outbound.push(GameEvent::Save(json));
                }
            }
            // fps estimate: renders counted in render(); here use tick window
            self.ui.fps_est = self.ui.renders as f32;
            self.ui.renders = 0;
        }

        outbound
    }

    pub fn render(&mut self, d: &mut Draw) {
        self.ui.renders = self.ui.renders.wrapping_add(1);

        d.clear("#12141a");

        let cam = self.world.camera.offset();
        d.set_offset(-cam.x, -cam.y);

        // Floor tiles
        render_map(d, &self.world);

        // Entities y-sorted
        let mut ids = self.world.alive_ids();
        ids.sort_by(|a, b| {
            let ya = self.world.get(*a).map(|e| e.pos.y).unwrap_or(0.0);
            let yb = self.world.get(*b).map(|e| e.pos.y).unwrap_or(0.0);
            ya.total_cmp(&yb)
        });
        for id in ids {
            if let Some(e) = self.world.get(id) {
                render_entity(d, e);
            }
        }

        self.fx.render_world(d);

        // Screen-space UI
        d.set_offset(0.0, 0.0);
        ui::render_hud(d, &self.world);
        self.fx.render_screen(d);

        let state_str = player_state_label(&self.world);
        ui::render_debug(d, &self.world, &self.ui, &self.fx, &state_str);

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
                // player iframes ticked in player module; others here
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
        // Pickups / beams move themselves; dummies get knockback via physics
        match entity.kind {
            EntityKind::Dummy => physics::move_entity(world, &mut entity),
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

fn spawn_debug_shot(world: &mut World) {
    let (center, facing) = match world.get(world.player_id) {
        Some(p) => (p.center(), p.facing),
        None => return,
    };
    let dir = facing.unit();
    let spawn = center.add(dir.scale(60.0)).sub(Vec2::new(3.0, 3.0));
    let vel = dir.scale(-crate::combat::tuning::DEBUG_SHOT_SPEED);
    world.spawn(Entity {
        kind: EntityKind::DebugShot,
        pos: spawn,
        vel,
        facing: Dir4::from_vec(vel, facing),
        body: Some(Body {
            half: Vec2::new(3.0, 3.0),
            solid: false,
            layer: layer::ENEMY_HIT,
            mask: layer::PLAYER_BODY,
        }),
        health: None,
        knockback: Vec2::ZERO,
        anim: AnimState::default(),
        data: EntityData::Beam(BeamData {
            dir: vel.normalize_or_zero(),
            traveled: 0.0,
            damage: crate::combat::tuning::DEBUG_SHOT_DAMAGE,
            knockback: 2.0,
            from_player: false,
            swing_id: 0xDEAD,
            hit: false,
        }),
        alive: true,
    });
}

fn render_map(d: &mut Draw, world: &World) {
    let map = &world.map;
    let cam = world.camera.offset();
    let x0 = ((cam.x / TILE_PX).floor() as i32).max(0) as u32;
    let y0 = ((cam.y / TILE_PX).floor() as i32).max(0) as u32;
    let x1 = ((cam.x + 480.0) / TILE_PX).ceil() as u32 + 1;
    let y1 = ((cam.y + 270.0) / TILE_PX).ceil() as u32 + 1;
    let x1 = x1.min(map.width);
    let y1 = y1.min(map.height);

    for ty in y0..y1 {
        for tx in x0..x1 {
            let tile = map.ground[map.idx(tx, ty)];
            let x = tx as f32 * TILE_PX;
            let y = ty as f32 * TILE_PX;
            match tile {
                WALL => d.rect(x, y, TILE_PX, TILE_PX, "#3a3f4a"),
                FOUNTAIN => d.rect(x, y, TILE_PX, TILE_PX, "#1a3040"),
                _ => d.rect(x, y, TILE_PX, TILE_PX, "#1e2430"),
            }
        }
    }
    // Fountain visual
    for (_id, e) in world.iter_alive() {
        if e.kind == EntityKind::FairyFountain {
            d.circle(e.pos.x + 20.0, e.pos.y + 20.0, 14.0, "#2a6a6a");
            d.circle(e.pos.x + 20.0, e.pos.y + 18.0, 6.0, "#40c0a0");
        }
    }
}

fn render_entity(d: &mut Draw, e: &Entity) {
    match e.kind {
        EntityKind::Player => {
            let flash = e.health.map(|h| h.flash > 0).unwrap_or(false);
            let iframes = e.health.map(|h| h.iframes > 0).unwrap_or(false);
            if iframes && (e.health.unwrap().iframes / 2).is_multiple_of(2) {
                return;
            }
            let color = if flash { "#ffffff" } else { "#e8e8f0" };
            // 16×24 visual, 16×16 collision at feet
            d.rect(e.pos.x, e.pos.y - 8.0, 16.0, 24.0, color);
            // Facing notch
            let c = e.center();
            let tip = c.add(e.facing.unit().scale(6.0));
            d.rect(tip.x - 2.0, tip.y - 2.0, 4.0, 4.0, "#8080ff");
            if let EntityData::Player(pd) = &e.data {
                if pd.shield_held {
                    let s = c.add(e.facing.unit().scale(10.0));
                    d.rect(s.x - 5.0, s.y - 5.0, 10.0, 10.0, "#6080c0");
                }
            }
        }
        EntityKind::Dummy => {
            if let EntityData::Dummy(d) = &e.data {
                if d.dead_ticks.is_some() {
                    return;
                }
            }
            let flash = e.health.map(|h| h.flash > 0).unwrap_or(false);
            let color = if flash { "#ffffff" } else { "#c06040" };
            d.rect(e.pos.x, e.pos.y, 16.0, 16.0, color);
        }
        EntityKind::Pickup => {
            if let EntityData::Pickup(pd) = &e.data {
                if pd.life < crate::combat::tuning::PICKUP_BLINK && (pd.life / 4) % 2 == 0 {
                    return;
                }
                match pd.kind {
                    crate::world::entity::PickupKind::Rupee => {
                        d.rect(e.pos.x, e.pos.y, 6.0, 8.0, "#40e080");
                    }
                    crate::world::entity::PickupKind::Heart => {
                        d.rect(e.pos.x, e.pos.y, 7.0, 6.0, "#e04040");
                    }
                    crate::world::entity::PickupKind::Energy => {
                        d.circle(e.pos.x + 3.0, e.pos.y + 3.0, 3.0, "#40e0ff");
                    }
                }
            }
        }
        EntityKind::SwordBeam => {
            d.rect(e.pos.x, e.pos.y, 6.0, 6.0, "#c0e0ff");
        }
        EntityKind::DebugShot => {
            d.rect(e.pos.x, e.pos.y, 6.0, 6.0, "#ff6060");
        }
        EntityKind::FairyFountain => {}
    }
}

fn player_state_label(world: &World) -> String {
    let Some(p) = world.get(world.player_id) else {
        return "?".into();
    };
    match &p.data {
        EntityData::Player(pd) => match pd.state {
            PlayerState::Idle => "idle".into(),
            PlayerState::Swing { stage, tick } => format!("swing{stage}:{tick}"),
            PlayerState::Charging { tick } => format!("charge:{tick}"),
            PlayerState::Spin { tick } => format!("spin:{tick}"),
            PlayerState::Dash { tick } => format!("dash:{tick}"),
            PlayerState::DashRecovery { tick } => format!("drec:{tick}"),
        },
        _ => "?".into(),
    }
}
