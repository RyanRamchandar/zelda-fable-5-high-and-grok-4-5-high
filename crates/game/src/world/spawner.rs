//! Distance-activated spawn slots from MapDef.spawns.

use content::maps::{SpawnDef, SpawnKind, TILE_PX};

use crate::enemies::{bat, octorok, slime};
use crate::math::Vec2;
use crate::world::entity::Entity;
use crate::world::{EntityId, World};

const ACTIVATE_PX: f32 = 480.0;
const SLEEP_PX: f32 = 420.0;
const RESPAWN_PX: f32 = 720.0;
const RESPAWN_TICKS: u32 = 600;
const SCAN_INTERVAL: u64 = 16;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotState {
    Dormant,
    Alive,
    Dead,
}

pub struct SpawnSlot {
    pub def: SpawnDef,
    pub state: SlotState,
    pub entity: Option<EntityId>,
    pub far_ticks: u32,
}

pub struct Spawner {
    pub slots: Vec<SpawnSlot>,
}

impl Spawner {
    pub fn populate(world: &mut World) -> Self {
        let mut slots = Vec::new();
        for def in world.map.spawns.clone() {
            match def.kind {
                SpawnKind::FairyFountain | SpawnKind::Dummy => {
                    let id = spawn_kind(world, def.kind, tile_pos(def.tx, def.ty));
                    slots.push(SpawnSlot {
                        def,
                        state: SlotState::Alive,
                        entity: Some(id),
                        far_ticks: 0,
                    });
                }
                _ => {
                    slots.push(SpawnSlot {
                        def,
                        state: SlotState::Dormant,
                        entity: None,
                        far_ticks: 0,
                    });
                }
            }
        }
        Self { slots }
    }

    pub fn update(&mut self, world: &mut World) {
        let player_pos = world
            .get(world.player_id)
            .map(|p| p.center())
            .unwrap_or(Vec2::ZERO);

        // Death → Dead
        for slot in &mut self.slots {
            if slot.state == SlotState::Alive {
                if let Some(id) = slot.entity {
                    if world.get(id).is_none() {
                        slot.state = SlotState::Dead;
                        slot.entity = None;
                        slot.far_ticks = 0;
                    }
                }
            }
        }

        if !world.tick.is_multiple_of(SCAN_INTERVAL) {
            return;
        }

        for slot in &mut self.slots {
            let pos = tile_pos(slot.def.tx, slot.def.ty);
            let dist = pos.sub(player_pos).len();

            match slot.state {
                SlotState::Dormant if dist < ACTIVATE_PX => {
                    let id = spawn_kind(world, slot.def.kind, pos);
                    slot.entity = Some(id);
                    slot.state = SlotState::Alive;
                    slot.far_ticks = 0;
                }
                SlotState::Dead => {
                    if dist > RESPAWN_PX {
                        slot.far_ticks = slot.far_ticks.saturating_add(SCAN_INTERVAL as u32);
                        if slot.far_ticks >= RESPAWN_TICKS {
                            slot.state = SlotState::Dormant;
                            slot.far_ticks = 0;
                        }
                    } else {
                        slot.far_ticks = 0;
                    }
                }
                _ => {}
            }
        }
    }
}

pub fn enemy_should_sleep(world: &World, pos: Vec2) -> bool {
    let Some(p) = world.get(world.player_id) else {
        return false;
    };
    pos.sub(p.center()).len() > SLEEP_PX
}

fn tile_pos(tx: u32, ty: u32) -> Vec2 {
    Vec2::new(tx as f32 * TILE_PX, ty as f32 * TILE_PX)
}

fn spawn_kind(world: &mut World, kind: SpawnKind, pos: Vec2) -> EntityId {
    match kind {
        SpawnKind::Slime => slime::spawn(world, pos),
        SpawnKind::Bat => bat::spawn(world, pos),
        SpawnKind::Octorok => octorok::spawn(world, pos),
        SpawnKind::FairyFountain => world.spawn(Entity::fountain(pos)),
        SpawnKind::Dummy => world.spawn(Entity::dummy(pos)),
    }
}
