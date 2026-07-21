//! Warden attack payloads: slam, rock fan, sweep, rim crumble.

use content::audio::sfx::SfxId;
use content::maps::{catalog, TileLayer};

use crate::combat::tuning;
use crate::fx::FxKind;
use crate::math::{Dir4, Vec2};
use crate::world::entity::{layer, AnimState, Body, Entity, EntityData, EntityId, EntityKind};
use crate::world::{World, WorldEvent};
use crate::Game;

use super::granite_warden::{arena_cx, arena_cy};

pub(crate) fn do_slam(game: &mut Game, warden: EntityId) {
    let center = game
        .world
        .get(warden)
        .map(|e| e.center())
        .unwrap_or(Vec2::ZERO);
    game.world.camera.add_shake(2.5, 10);
    game.world
        .push_event(WorldEvent::FxRequest(FxKind::Impact { pos: center }));
    // Expanding ring pulses (3 steps).
    for r in [24.0f32, 40.0, 56.0] {
        if let Some(p) = game.world.get(game.world.player_id) {
            let d = p.center().sub(center).len();
            if (d - r).abs() < 12.0 {
                let dir = p.center().sub(center).normalize_or_zero();
                game.world.push_event(WorldEvent::DamagedPlayer {
                    amount: tuning::WARDEN_SLAM_DAMAGE,
                    dir,
                    source: Some(warden),
                });
            }
        }
        let _ = r;
    }
}

pub(crate) fn do_rock_fan(game: &mut Game, warden: EntityId, count: u8) {
    let origin = game
        .world
        .get(warden)
        .map(|e| e.center())
        .unwrap_or(Vec2::ZERO);
    let ppos = game
        .world
        .get(game.world.player_id)
        .map(|p| p.center())
        .unwrap_or(origin.add(Vec2::new(0.0, 40.0)));
    let base = ppos.sub(origin).normalize_or_zero();
    let spread = if count >= 5 { 0.45 } else { 0.35 };
    for i in 0..count {
        let t = i as f32 - (count as f32 - 1.0) * 0.5;
        let angle = t * spread;
        let (s, c) = (angle.sin(), angle.cos());
        let dir = Vec2::new(base.x * c - base.y * s, base.x * s + base.y * c);
        spawn_rock(&mut game.world, origin, dir);
    }
}

pub(crate) fn spawn_rock(world: &mut World, pos: Vec2, dir: Vec2) {
    use crate::world::entity::RockData;
    world.spawn(Entity {
        kind: EntityKind::OctorokRock,
        pos: Vec2::new(pos.x - 4.0, pos.y - 4.0),
        vel: dir.scale(2.4),
        facing: Dir4::Down,
        body: Some(Body {
            half: Vec2::new(4.0, 4.0),
            solid: false,
            layer: layer::ENEMY_HIT,
            mask: layer::PLAYER_BODY,
        }),
        health: None,
        knockback: Vec2::ZERO,
        anim: AnimState::default(),
        data: EntityData::Rock(RockData {
            dir,
            damage: tuning::WARDEN_ROCK_DAMAGE,
            from_player: false,
            hit: false,
            swing_id: world.tick as u32,
        }),
        alive: true,
    });
}

pub(crate) fn do_sweep(game: &mut Game, warden: EntityId) {
    let cy = arena_cy();
    game.world.camera.add_shake(2.0, 8);
    if let Some(p) = game.world.get(game.world.player_id) {
        // Half-arena horizontal band — dash i-frames dodge.
        if (p.center().y - cy).abs() < 28.0 {
            let iframes = p.health.map(|h| h.iframes).unwrap_or(0);
            if iframes == 0 {
                let dir = Vec2::new(1.0, 0.0);
                game.world.push_event(WorldEvent::DamagedPlayer {
                    amount: tuning::WARDEN_SWEEP_DAMAGE,
                    dir,
                    source: Some(warden),
                });
            }
        }
    }
    for x in 0..8 {
        let pos = Vec2::new(arena_cx() - 70.0 + x as f32 * 20.0, cy);
        game.world
            .push_event(WorldEvent::FxRequest(FxKind::Impact { pos }));
    }
}

pub(crate) fn spawn_two_pebbles(game: &mut Game) {
    let a = Vec2::new(arena_cx() - 40.0, arena_cy() + 20.0);
    let b = Vec2::new(arena_cx() + 40.0, arena_cy() + 20.0);
    super::pebble::spawn(&mut game.world, a);
    super::pebble::spawn(&mut game.world, b);
}

pub(crate) fn crumble_rim(game: &mut Game) {
    // Shrink floor ~1 tile: paint pit/crumble on outer ring of arena room.
    let (x0, y0, x1, y1) = (61u32, 1u32, 78u32, 14u32);
    for tx in x0..=x1 {
        for ty in [y0, y1] {
            game.world
                .set_tile(TileLayer::Ground, tx, ty, catalog::D_PIT);
        }
    }
    for ty in y0..=y1 {
        for tx in [x0, x1] {
            game.world
                .set_tile(TileLayer::Ground, tx, ty, catalog::D_PIT);
        }
    }
    game.world.camera.add_shake(2.0, 20);
    game.world
        .push_event(WorldEvent::Sfx(SfxId::PhaseBreak));
}

