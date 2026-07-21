//! Debug helpers extracted from `lib.rs` for file-cap headroom.

use content::maps::{self, TriggerKind};

use crate::math::{Dir4, Vec2};
use crate::world::entity::{
    layer, AnimState, BeamData, Body, Entity, EntityData, EntityKind, PlayerState,
};
use crate::world::World;

pub(crate) fn spawn_debug_shot(world: &mut World) {
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

pub(crate) fn player_state_label(world: &World) -> String {
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
            PlayerState::LedgeHop { tick, .. } => format!("ledge:{tick}"),
        },
        _ => "?".into(),
    }
}

pub(crate) fn debug_assert_door_entries(world: &World) {
    for tr in &world.map.triggers {
        let TriggerKind::Door { target, entry } = tr.kind else {
            continue;
        };
        let dest = maps::build(target);
        let Some((tx, ty)) = dest.entry_pos(entry) else {
            continue;
        };
        for dtr in &dest.triggers {
            if let TriggerKind::Door { .. } = dtr.kind {
                let inside = tx >= dtr.tx
                    && ty >= dtr.ty
                    && tx < dtr.tx + dtr.w
                    && ty < dtr.ty + dtr.h;
                debug_assert!(!inside, "door re-entry: {target:?} entry {entry}");
            }
        }
    }
}
