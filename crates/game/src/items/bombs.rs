//! Placed bombs: fuse countdown, blast damage, tile cracks / barricades.

use content::audio::sfx::SfxId;
use content::maps::TILE_PX;

use crate::combat::tuning;
use crate::fx::FxKind;
use crate::math::{Dir4, Vec2};
use crate::world::entity::{AnimState, BombData, Entity, EntityData, EntityKind};
use crate::world::{ActiveAttack, AttackKind, World, WorldEvent};
use crate::Game;

/// Place a bomb at the player's feet tile. Returns true if placed.
pub fn try_place(world: &mut World) -> bool {
    let (center, ok) = {
        let Some(p) = world.get(world.player_id) else {
            return false;
        };
        let EntityData::Player(pd) = &p.data else {
            return false;
        };
        if pd.selected_item != 1 || pd.bombs == 0 || pd.bomb_cap == 0 {
            return false;
        }
        (p.center(), true)
    };
    let _ = ok;
    if let Some(p) = world.get_mut(world.player_id) {
        if let EntityData::Player(pd) = &mut p.data {
            pd.bombs = pd.bombs.saturating_sub(1);
        }
    }
    let tx = (center.x / TILE_PX).floor();
    let ty = ((center.y + 6.0) / TILE_PX).floor();
    let pos = Vec2::new(tx * TILE_PX, ty * TILE_PX);
    world.spawn(Entity {
        kind: EntityKind::Bomb,
        pos,
        vel: Vec2::ZERO,
        facing: Dir4::Down,
        body: None,
        health: None,
        knockback: Vec2::ZERO,
        anim: AnimState::default(),
        data: EntityData::Bomb(BombData {
            fuse: tuning::BOMB_FUSE_TICKS,
        }),
        alive: true,
    });
    world.push_event(WorldEvent::Sfx(SfxId::BombFuse));
    true
}

pub fn update(game: &mut Game) {
    let ids = game.world.alive_ids();
    let mut boom = Vec::new();
    for id in ids {
        let Some(e) = game.world.get_mut(id) else {
            continue;
        };
        if e.kind != EntityKind::Bomb {
            continue;
        }
        let EntityData::Bomb(b) = &mut e.data else {
            continue;
        };
        if b.fuse > 0 {
            b.fuse -= 1;
        }
        let fuse = b.fuse;
        e.anim.timer = e.anim.timer.wrapping_add(1);
        if fuse <= 30 {
            e.anim.frame = (e.anim.timer / 2) % 2;
        } else {
            e.anim.frame = (e.anim.timer / 8) % 2;
        }
        if fuse == 0 {
            boom.push((id, e.center()));
        }
    }
    for (id, center) in boom {
        explode(game, id, center);
    }
}

fn explode(game: &mut Game, id: crate::world::EntityId, center: Vec2) {
    game.world.despawn(id);
    game.world
        .push_event(WorldEvent::FxRequest(FxKind::KillPoof { pos: center }));
    game.world
        .push_event(WorldEvent::Sfx(SfxId::BombBoom));
    game.world.camera.add_shake(3.0, 10);

    let swing = {
        let mut sid = 0xB0BBu32;
        if let Some(p) = game.world.get_mut(game.world.player_id) {
            if let EntityData::Player(pd) = &mut p.data {
                pd.swing_id = pd.swing_id.wrapping_add(1);
                sid = pd.swing_id;
            }
        }
        sid
    };
    game.world.active_attacks.push(ActiveAttack {
        owner: game.world.player_id,
        kind: AttackKind::Bomb,
        swing_id: swing,
        center,
        half: Vec2::ZERO,
        radius: tuning::BOMB_BLAST_RADIUS,
        use_radius: true,
        dir: Vec2::ZERO,
        damage: tuning::BOMB_DAMAGE,
        knockback: tuning::BOMB_KNOCKBACK,
    });

    if let Some(p) = game.world.get(game.world.player_id) {
        let pc = p.center();
        let d = pc.sub(center);
        if d.len() <= tuning::BOMB_TILE_RADIUS {
            let dir = if d.len_sq() > 0.01 {
                d.normalize_or_zero()
            } else {
                Dir4::Down.unit()
            };
            game.world.push_event(WorldEvent::DamagedPlayer {
                amount: tuning::BOMB_PLAYER_DAMAGE,
                dir,
            });
        }
    }

    crate::puzzle::try_open_bomb_wall(game, center, tuning::BOMB_TILE_RADIUS);
    crate::puzzle::bomb_break_barricades(game, center, tuning::BOMB_TILE_RADIUS);
}
