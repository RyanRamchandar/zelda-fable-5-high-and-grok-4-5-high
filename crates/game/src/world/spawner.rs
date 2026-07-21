//! Distance-activated spawn slots from MapDef.spawns.

use content::flags::{self, CAMP_WAVE_CHAIN};
use content::maps::{SpawnDef, SpawnKind, TILE_PX};

use crate::enemies::{bat, octorok, raider, skeleton, slime, wisp};
use crate::math::Vec2;
use crate::save_data::has_flag;
use crate::world::entity::Entity;
use crate::world::{EntityId, World, WorldEvent};

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
    /// Groups that must not distance-activate (camp wave chain).
    pub locked_groups: Vec<u16>,
    /// `GROUP_CAMP_GUARD` was set at map load — war already won.
    pub camp_war_won: bool,
}

impl Spawner {
    pub fn populate(world: &mut World, save_flags: &[u16]) -> Self {
        let camp_war_won = has_flag(save_flags, flags::GROUP_CAMP_GUARD);
        let mut locked_groups = Vec::new();
        if camp_war_won {
            locked_groups.push(flags::GRP_CAMP_W2);
            locked_groups.push(flags::GRP_CAMP_W3);
        } else {
            for &(_, next) in &CAMP_WAVE_CHAIN {
                locked_groups.push(next);
            }
        }
        // Phase 3A: sanctum miniboss reserved for 3B.
        locked_groups.push(flags::GRP_DNG_SANCTUM);

        let mut slots = Vec::new();
        for def in world.map.spawns.clone() {
            match def.kind {
                SpawnKind::FairyFountain
                | SpawnKind::Dummy
                | SpawnKind::Sign { .. }
                | SpawnKind::Npc { .. }
                | SpawnKind::Chest { .. }
                | SpawnKind::Gem { .. } => {
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
        Self {
            slots,
            locked_groups,
            camp_war_won,
        }
    }

    /// Re-apply save flags to already-spawned chests/gems after populate.
    pub fn apply_save(&mut self, world: &mut World, gems: u8, flags: &[u16]) {
        for slot in &self.slots {
            let Some(id) = slot.entity else {
                continue;
            };
            let Some(e) = world.get_mut(id) else {
                continue;
            };
            match (&slot.def.kind, &mut e.data) {
                (SpawnKind::Chest { flag, .. }, crate::world::entity::EntityData::Chest(cd)) => {
                    if has_flag(flags, *flag) {
                        cd.open = true;
                        e.anim.frame = 1;
                    }
                }
                (SpawnKind::Gem { id: gid }, crate::world::entity::EntityData::Gem(g))
                    if gems & (1 << (*gid).min(2)) != 0 =>
                {
                    g.taken = true;
                }
                _ => {}
            }
        }
    }

    pub fn unlock_and_activate(&mut self, world: &mut World, group: u16) {
        self.locked_groups.retain(|&g| g != group);
        for i in 0..self.slots.len() {
            if self.slots[i].def.group != group {
                continue;
            }
            if self.slots[i].state != SlotState::Dormant {
                continue;
            }
            if !is_hostile_kind(self.slots[i].def.kind) {
                continue;
            }
            let pos = tile_pos(self.slots[i].def.tx, self.slots[i].def.ty);
            let id = spawn_kind(world, self.slots[i].def.kind, pos);
            self.slots[i].entity = Some(id);
            self.slots[i].state = SlotState::Alive;
            self.slots[i].far_ticks = 0;
        }
    }

    pub fn is_locked(&self, group: u16) -> bool {
        group != 0 && self.locked_groups.contains(&group)
    }

    pub fn update(&mut self, world: &mut World) {
        let player_pos = world
            .get(world.player_id)
            .map(|p| p.center())
            .unwrap_or(Vec2::ZERO);

        let mut cleared_groups = Vec::new();

        // Death → Dead
        for slot in &mut self.slots {
            if slot.state == SlotState::Alive {
                if let Some(id) = slot.entity {
                    if world.get(id).is_none() {
                        slot.state = SlotState::Dead;
                        slot.entity = None;
                        slot.far_ticks = 0;
                        if slot.def.group != 0 && is_hostile_kind(slot.def.kind) {
                            cleared_groups.push(slot.def.group);
                        }
                    }
                }
            }
        }

        for g in cleared_groups {
            if group_cleared(self, g) {
                world.push_event(WorldEvent::GroupCleared(g));
            }
        }

        if !world.tick.is_multiple_of(SCAN_INTERVAL) {
            return;
        }

        for i in 0..self.slots.len() {
            let pos = tile_pos(self.slots[i].def.tx, self.slots[i].def.ty);
            let dist = pos.sub(player_pos).len();
            let group = self.slots[i].def.group;
            let kind = self.slots[i].def.kind;

            match self.slots[i].state {
                SlotState::Dormant if dist < ACTIVATE_PX => {
                    if self.is_locked(group) {
                        continue;
                    }
                    if is_hostile_kind(kind) && group != 0 && !group_dormant_eligible(self, group)
                    {
                        continue;
                    }
                    let id = spawn_kind(world, kind, pos);
                    self.slots[i].entity = Some(id);
                    self.slots[i].state = SlotState::Alive;
                    self.slots[i].far_ticks = 0;
                }
                SlotState::Dead => {
                    if !is_hostile_kind(kind) {
                        continue;
                    }
                    if self.is_locked(group) {
                        continue;
                    }
                    // Do not rewind wave-1 while later waves are unlocked mid-chain.
                    if group == flags::GRP_CAMP_GUARD
                        && !self.camp_war_won
                        && camp_chain_in_progress(self)
                    {
                        self.slots[i].far_ticks = 0;
                        continue;
                    }
                    // Grouped: only respawn when whole group is dormant-eligible.
                    if group != 0 && !group_all_dead_far(self, group, player_pos) {
                        self.slots[i].far_ticks = 0;
                        continue;
                    }
                    if dist > RESPAWN_PX {
                        self.slots[i].far_ticks = self.slots[i]
                            .far_ticks
                            .saturating_add(SCAN_INTERVAL as u32);
                        if self.slots[i].far_ticks >= RESPAWN_TICKS {
                            if group != 0 {
                                for s in &mut self.slots {
                                    if s.def.group == group && s.state == SlotState::Dead {
                                        s.state = SlotState::Dormant;
                                        s.far_ticks = 0;
                                    }
                                }
                                // Restart camp chain if wave-1 rewinds before war won.
                                if group == flags::GRP_CAMP_GUARD && !self.camp_war_won {
                                    reset_camp_chain(self, world);
                                }
                            } else {
                                self.slots[i].state = SlotState::Dormant;
                                self.slots[i].far_ticks = 0;
                            }
                        }
                    } else {
                        self.slots[i].far_ticks = 0;
                    }
                }
                _ => {}
            }
        }
    }
}

fn camp_chain_in_progress(spawner: &Spawner) -> bool {
    for &(_, next) in &CAMP_WAVE_CHAIN {
        if !spawner.locked_groups.contains(&next) {
            return true;
        }
    }
    false
}

fn reset_camp_chain(spawner: &mut Spawner, world: &mut World) {
    for &g in &[flags::GRP_CAMP_W2, flags::GRP_CAMP_W3] {
        if !spawner.locked_groups.contains(&g) {
            spawner.locked_groups.push(g);
        }
        for slot in &mut spawner.slots {
            if slot.def.group != g {
                continue;
            }
            if let Some(id) = slot.entity.take() {
                world.despawn(id);
            }
            slot.state = SlotState::Dormant;
            slot.far_ticks = 0;
        }
    }
}

pub fn group_cleared(spawner: &Spawner, group: u16) -> bool {
    if group == 0 {
        return false;
    }
    let mut any = false;
    for s in &spawner.slots {
        if s.def.group != group || !is_hostile_kind(s.def.kind) {
            continue;
        }
        any = true;
        if s.state != SlotState::Dead {
            return false;
        }
    }
    any
}

fn group_dormant_eligible(spawner: &Spawner, group: u16) -> bool {
    for s in &spawner.slots {
        if s.def.group == group && is_hostile_kind(s.def.kind) && s.state == SlotState::Alive {
            return false;
        }
    }
    true
}

fn group_all_dead_far(spawner: &Spawner, group: u16, player_pos: Vec2) -> bool {
    for s in &spawner.slots {
        if s.def.group != group || !is_hostile_kind(s.def.kind) {
            continue;
        }
        if s.state != SlotState::Dead {
            return false;
        }
        let pos = tile_pos(s.def.tx, s.def.ty);
        if pos.sub(player_pos).len() <= RESPAWN_PX {
            return false;
        }
    }
    true
}

fn is_hostile_kind(kind: SpawnKind) -> bool {
    matches!(
        kind,
        SpawnKind::Slime
            | SpawnKind::Bat
            | SpawnKind::Octorok
            | SpawnKind::RaiderSpear
            | SpawnKind::RaiderTorch
            | SpawnKind::Wisp
            | SpawnKind::Skeleton
            | SpawnKind::Dummy
    )
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
        SpawnKind::RaiderSpear => raider::spawn_spear(world, pos),
        SpawnKind::RaiderTorch => raider::spawn_torch(world, pos),
        SpawnKind::Wisp => wisp::spawn(world, pos),
        SpawnKind::Skeleton => skeleton::spawn(world, pos),
        SpawnKind::FairyFountain => world.spawn(Entity::fountain(pos)),
        SpawnKind::Dummy => world.spawn(Entity::dummy(pos)),
        SpawnKind::Sign { text } => world.spawn(Entity::sign(pos, text)),
        SpawnKind::Npc { npc } => world.spawn(Entity::npc(pos, npc)),
        SpawnKind::Chest { flag, loot } => world.spawn(Entity::chest(pos, flag, loot, false)),
        SpawnKind::Gem { id } => world.spawn(Entity::gem(pos, id, false)),
    }
}
