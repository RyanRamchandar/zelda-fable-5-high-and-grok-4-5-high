//! World: entity arena, map, camera, hitstop, event queue.

pub mod camera;
pub mod entity;
pub mod physics;
pub mod spawner;

use content::maps::{catalog, MapDef, MapId, TileLayer};
use engine::chunks::ChunkKey;
use fastrand::Rng;

use crate::combat::style::StyleVerb;
use crate::fx::FxKind;
use crate::math::Vec2;
use content::audio::sfx::SfxId;
use content::maps::TILE_PX;

pub use camera::Camera;
pub use entity::{Entity, EntityId, EntityKind};
pub use spawner::Spawner;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AttackKind {
    Slash,
    Backslash,
    Finisher,
    Spin,
    Beam,
    #[allow(dead_code)]
    DebugShot,
}

#[derive(Clone, Debug)]
pub enum WorldEvent {
    AttackHit {
        target: EntityId,
        attack: AttackKind,
        dir: Vec2,
        pos: Vec2,
        damage: f32,
        knockback: f32,
        #[allow(dead_code)]
        source: EntityId,
    },
    DamagedPlayer {
        amount: i32,
        dir: Vec2,
    },
    Killed {
        #[allow(dead_code)]
        kind: EntityKind,
        pos: Vec2,
    },
    FxRequest(FxKind),
    Sfx(SfxId),
    StyleAction(StyleVerb),
    EnergyDenied,
    RegionEntered(u8),
    GroupCleared(u16),
}

#[derive(Clone, Copy, Debug)]
pub struct ActiveAttack {
    pub owner: EntityId,
    pub kind: AttackKind,
    pub swing_id: u32,
    pub center: Vec2,
    pub half: Vec2,
    pub radius: f32,
    pub use_radius: bool,
    pub dir: Vec2,
    pub damage: f32,
    pub knockback: f32,
}

pub struct Slot {
    pub gen: u32,
    pub entity: Option<Entity>,
}

pub struct World {
    pub arena: Vec<Slot>,
    pub free: Vec<u32>,
    pub map: MapDef,
    #[allow(dead_code)] // seam: map identity for 2B systems
    pub map_id: MapId,
    pub camera: Camera,
    pub hitstop: u8,
    pub events: Vec<WorldEvent>,
    pub tick: u64,
    pub rng: Rng,
    pub player_id: EntityId,
    pub active_attacks: Vec<ActiveAttack>,
    pub hit_pairs: Vec<(u32, u32)>,
    pub dirty_chunks: Vec<ChunkKey>,
    pub animated_tiles: Vec<(u32, u32, u16)>,
    pub checkpoint: u8,
}

impl World {
    pub fn new(map_id: MapId, map: MapDef, player_pos: Vec2) -> Self {
        let animated_tiles = collect_animated(&map);
        let mut world = Self {
            arena: Vec::new(),
            free: Vec::new(),
            map,
            map_id,
            camera: Camera::new(player_pos.add(Vec2::new(8.0, 8.0))),
            hitstop: 0,
            events: Vec::new(),
            tick: 0,
            rng: Rng::new(),
            player_id: EntityId { index: 0, gen: 0 },
            active_attacks: Vec::new(),
            hit_pairs: Vec::new(),
            dirty_chunks: Vec::new(),
            animated_tiles,
            checkpoint: 0,
        };
        let pid = world.spawn(Entity::player(player_pos));
        world.player_id = pid;
        world.camera.snap_to(player_pos.add(Vec2::new(8.0, 8.0)));
        world
    }

    /// Mutate a tile and mark the owning chunk dirty (2B secrets / Phase 3 bombs).
    #[allow(dead_code)]
    pub fn set_tile(&mut self, layer: TileLayer, tx: u32, ty: u32, id: u16) {
        self.map.set(tx, ty, layer, id);
        let cx = tx / 16;
        let cy = ty / 16;
        let key = ChunkKey {
            layer: if matches!(layer, TileLayer::Overhang) {
                1
            } else {
                0
            },
            cx,
            cy,
        };
        if !self.dirty_chunks.contains(&key) {
            self.dirty_chunks.push(key);
        }
        // Ground change also dirties the composite layer-0 chunk.
        if matches!(layer, TileLayer::Detail) {
            let g = ChunkKey { layer: 0, cx, cy };
            if !self.dirty_chunks.contains(&g) {
                self.dirty_chunks.push(g);
            }
        }
        self.animated_tiles.retain(|&(x, y, _)| !(x == tx && y == ty));
        if catalog::tile_info(id).frames > 1 {
            self.animated_tiles.push((tx, ty, id));
        }
    }

    pub fn spawn(&mut self, entity: Entity) -> EntityId {
        if let Some(index) = self.free.pop() {
            let slot = &mut self.arena[index as usize];
            let gen = slot.gen;
            slot.entity = Some(entity);
            EntityId { index, gen }
        } else {
            let index = self.arena.len() as u32;
            self.arena.push(Slot {
                gen: 0,
                entity: Some(entity),
            });
            EntityId { index, gen: 0 }
        }
    }

    pub fn despawn(&mut self, id: EntityId) {
        let Some(slot) = self.arena.get_mut(id.index as usize) else {
            return;
        };
        if slot.gen != id.gen {
            return;
        }
        slot.entity = None;
        slot.gen = slot.gen.wrapping_add(1);
        self.free.push(id.index);
    }

    pub fn get(&self, id: EntityId) -> Option<&Entity> {
        let slot = self.arena.get(id.index as usize)?;
        if slot.gen != id.gen {
            return None;
        }
        slot.entity.as_ref().filter(|e| e.alive)
    }

    pub fn get_mut(&mut self, id: EntityId) -> Option<&mut Entity> {
        let slot = self.arena.get_mut(id.index as usize)?;
        if slot.gen != id.gen {
            return None;
        }
        slot.entity.as_mut().filter(|e| e.alive)
    }

    pub fn iter_alive(&self) -> impl Iterator<Item = (EntityId, &Entity)> {
        self.arena.iter().enumerate().filter_map(|(i, slot)| {
            let e = slot.entity.as_ref()?;
            if !e.alive {
                return None;
            }
            Some((
                EntityId {
                    index: i as u32,
                    gen: slot.gen,
                },
                e,
            ))
        })
    }

    /// Stable id list for pairwise queries (avoids borrow issues).
    pub fn alive_ids(&self) -> Vec<EntityId> {
        self.iter_alive().map(|(id, _)| id).collect()
    }

    pub fn push_event(&mut self, ev: WorldEvent) {
        self.events.push(ev);
    }

    pub fn entity_count(&self) -> usize {
        self.iter_alive().count()
    }

    pub fn already_hit(&self, swing_id: u32, target_index: u32) -> bool {
        self.hit_pairs
            .iter()
            .any(|&(s, t)| s == swing_id && t == target_index)
    }

    pub fn mark_hit(&mut self, swing_id: u32, target_index: u32) {
        self.hit_pairs.push((swing_id, target_index));
    }

    pub fn clear_swing_hits(&mut self, swing_id: u32) {
        self.hit_pairs.retain(|&(s, _)| s != swing_id);
    }
}

fn collect_animated(map: &MapDef) -> Vec<(u32, u32, u16)> {
    let mut out = Vec::new();
    for ty in 0..map.height {
        for tx in 0..map.width {
            let i = map.idx(tx, ty);
            for &id in &[map.ground[i], map.detail[i]] {
                if catalog::tile_info(id).frames > 1 {
                    out.push((tx, ty, id));
                }
            }
        }
    }
    let _ = TILE_PX;
    out
}
