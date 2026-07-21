//! Entity arena types. Phase 1B/2B extend `EntityKind` / `EntityData` — keep matches exhaustive.

use content::maps::Loot;
use content::text::{NpcId, TextId};

use crate::math::{Dir4, Vec2};

use super::entity_data;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EntityId {
    pub index: u32,
    pub gen: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EntityKind {
    Player,
    Dummy,
    Pickup,
    SwordBeam,
    FairyFountain,
    /// Slow test projectile (H key).
    DebugShot,
    Slime,
    Bat,
    Octorok,
    OctorokRock,
    RaiderSpear,
    RaiderTorch,
    Wisp,
    Skeleton,
    TorchProj,
    TorchFlame,
    Sign,
    Npc,
    Chest,
    Gem,
    Bomb,
    Boomerang,
}

/// Collision layer bits (bitmask).
pub mod layer {
    pub const PLAYER_BODY: u8 = 1 << 0;
    pub const ENEMY_BODY: u8 = 1 << 1;
    pub const PLAYER_HIT: u8 = 1 << 2;
    pub const ENEMY_HIT: u8 = 1 << 3;
    pub const PICKUP: u8 = 1 << 4;
}

#[derive(Clone, Copy, Debug)]
pub struct Body {
    pub half: Vec2,
    pub solid: bool,
    pub layer: u8,
    pub mask: u8,
}

#[derive(Clone, Copy, Debug)]
pub struct Health {
    pub hp: i32,
    pub max: i32,
    pub iframes: u16,
    pub flash: u8,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct AnimState {
    pub sheet: u16,
    pub frame: u16,
    pub timer: u16,
}

#[derive(Clone, Debug)]
pub enum EntityData {
    #[allow(dead_code)]
    None,
    Player(PlayerData),
    Dummy(DummyData),
    Pickup(PickupData),
    Beam(BeamData),
    Fountain,
    Slime(SlimeData),
    Bat(BatData),
    Octorok(OctorokData),
    Rock(RockData),
    RaiderSpear(RaiderSpearData),
    RaiderTorch(RaiderTorchData),
    Wisp(WispData),
    Skeleton(SkeletonData),
    TorchProj(TorchProjData),
    TorchFlame(TorchFlameData),
    Sign(SignData),
    Npc(NpcData),
    Chest(ChestData),
    Gem(GemData),
    Bomb(BombData),
    Boomerang(BoomerangData),
}

#[derive(Clone, Debug)]
pub struct SignData {
    pub text: TextId,
}

#[derive(Clone, Debug)]
pub struct NpcData {
    pub npc: NpcId,
}

#[derive(Clone, Debug)]
pub struct ChestData {
    pub flag: u16,
    pub loot: Loot,
    pub open: bool,
    pub open_anim: u8,
}

#[derive(Clone, Debug)]
pub struct GemData {
    pub id: u8,
    pub taken: bool,
    /// Courage gem ward until chime finale (`PUZZLE_CHIMES_DONE`).
    pub sealed: bool,
}

#[derive(Clone, Debug)]
pub struct BombData {
    pub fuse: u16,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlayerState {
    Idle,
    Swing { stage: u8, tick: u16 },
    Charging { tick: u16 },
    Spin { tick: u16 },
    Dash { tick: u16 },
    DashRecovery { tick: u16 },
    LedgeHop { tick: u16, from: Vec2, to: Vec2 },
}

#[derive(Clone, Debug)]
pub struct PlayerData {
    pub state: PlayerState,
    pub energy: f32,
    pub energy_deny_flash: u8,
    pub ticks_since_spend: u16,
    pub hearts: i32,
    pub max_hearts: i32,
    pub rupees: u32,
    pub move_blend: f32,
    pub swing_id: u32,
    pub hit_mask: u64,
    pub combo_drop: u16,
    pub buffer_attack: bool,
    pub charge_ready_sfx: bool,
    pub shield_ticks: u16,
    pub shield_held: bool,
    pub dash_dir: Vec2,
    pub dash_through_awarded: bool,
    pub lunge_ticks: u8,
    pub style_points: f32,
    pub style_rank: u8,
    pub style_pulse: u8,
    pub verb_cooldowns: [u16; 7],
    pub combat_timer: u16,
    pub no_dmg_streak: u16,
    pub out_of_combat: u16,
    pub walk_timer: u16,
    pub bombs: u8,
    pub bomb_cap: u8,
    /// 0 = none, 1 = bombs (2 = boomerang in Phase 3).
    pub selected_item: u8,
    pub item_cycle_flash: u8,
}

impl PlayerData {
    pub fn new() -> Self {
        Self {
            state: PlayerState::Idle,
            energy: 100.0,
            energy_deny_flash: 0,
            ticks_since_spend: 999,
            hearts: 6,
            max_hearts: 6,
            rupees: 0,
            move_blend: 0.0,
            swing_id: 0,
            hit_mask: 0,
            combo_drop: 0,
            buffer_attack: false,
            charge_ready_sfx: false,
            shield_ticks: 0,
            shield_held: false,
            dash_dir: Vec2::ZERO,
            dash_through_awarded: false,
            lunge_ticks: 0,
            style_points: 0.0,
            style_rank: 0,
            style_pulse: 0,
            verb_cooldowns: [0; 7],
            combat_timer: 0,
            no_dmg_streak: 0,
            out_of_combat: 0,
            walk_timer: 0,
            bombs: 0,
            bomb_cap: 0,
            selected_item: 0,
            item_cycle_flash: 0,
        }
    }

    pub fn at_full_hearts(&self) -> bool {
        self.hearts >= self.max_hearts
    }
}

impl Default for PlayerData {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct DummyData {
    pub home: Vec2,
    pub dead_ticks: Option<u16>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PickupKind {
    Rupee,
    Heart,
    Energy,
}

#[derive(Clone, Debug)]
pub struct PickupData {
    pub kind: PickupKind,
    pub life: u16,
}

#[derive(Clone, Debug)]
pub struct BeamData {
    pub dir: Vec2,
    pub traveled: f32,
    pub damage: f32,
    pub knockback: f32,
    /// Player-owned beams use PLAYER_HIT; enemy/debug use ENEMY_HIT.
    pub from_player: bool,
    pub swing_id: u32,
    pub hit: bool,
}

pub use entity_data::{
    BatData, BatState, BoomerangData, BoomerangPhase, OctorokData, OctorokState, RaiderSpearData,
    RaiderSpearState, RaiderTorchData, RaiderTorchState, RockData, SkeletonData, SkeletonState,
    SlimeData, SlimeState, TorchFlameData, TorchProjData, WispData, WispState,
};

#[derive(Clone, Debug)]
pub struct Entity {
    pub kind: EntityKind,
    pub pos: Vec2,
    pub vel: Vec2,
    pub facing: Dir4,
    pub body: Option<Body>,
    pub health: Option<Health>,
    pub knockback: Vec2,
    pub anim: AnimState,
    pub data: EntityData,
    pub alive: bool,
}

impl Entity {
    pub fn player(pos: Vec2) -> Self {
        Self {
            kind: EntityKind::Player,
            pos,
            vel: Vec2::ZERO,
            facing: Dir4::Down,
            body: Some(Body {
                half: Vec2::new(8.0, 8.0),
                solid: true,
                layer: layer::PLAYER_BODY,
                mask: layer::ENEMY_BODY | layer::ENEMY_HIT | layer::PICKUP,
            }),
            health: Some(Health {
                hp: 6,
                max: 6,
                iframes: 0,
                flash: 0,
            }),
            knockback: Vec2::ZERO,
            anim: AnimState::default(),
            data: EntityData::Player(PlayerData::new()),
            alive: true,
        }
    }

    pub fn dummy(pos: Vec2) -> Self {
        Self {
            kind: EntityKind::Dummy,
            pos,
            vel: Vec2::ZERO,
            facing: Dir4::Down,
            body: Some(Body {
                half: Vec2::new(8.0, 8.0),
                solid: false,
                layer: layer::ENEMY_BODY,
                mask: layer::PLAYER_HIT,
            }),
            health: Some(Health {
                hp: 20,
                max: 20,
                iframes: 0,
                flash: 0,
            }),
            knockback: Vec2::ZERO,
            anim: AnimState::default(),
            data: EntityData::Dummy(DummyData {
                home: pos,
                dead_ticks: None,
            }),
            alive: true,
        }
    }

    pub fn fountain(pos: Vec2) -> Self {
        Self {
            kind: EntityKind::FairyFountain,
            pos,
            vel: Vec2::ZERO,
            facing: Dir4::Down,
            body: Some(Body {
                half: Vec2::new(20.0, 20.0),
                solid: false,
                layer: 0,
                mask: 0,
            }),
            health: None,
            knockback: Vec2::ZERO,
            anim: AnimState::default(),
            data: EntityData::Fountain,
            alive: true,
        }
    }

    pub fn sign(pos: Vec2, text: TextId) -> Self {
        Self {
            kind: EntityKind::Sign,
            pos,
            vel: Vec2::ZERO,
            facing: Dir4::Down,
            body: Some(Body {
                half: Vec2::new(8.0, 8.0),
                solid: true,
                layer: layer::ENEMY_BODY,
                mask: 0,
            }),
            health: None,
            knockback: Vec2::ZERO,
            anim: AnimState::default(),
            data: EntityData::Sign(SignData { text }),
            alive: true,
        }
    }

    pub fn npc(pos: Vec2, npc: NpcId) -> Self {
        Self {
            kind: EntityKind::Npc,
            pos,
            vel: Vec2::ZERO,
            facing: Dir4::Down,
            body: Some(Body {
                half: Vec2::new(8.0, 8.0),
                solid: true,
                layer: layer::ENEMY_BODY,
                mask: 0,
            }),
            health: None,
            knockback: Vec2::ZERO,
            anim: AnimState::default(),
            data: EntityData::Npc(NpcData { npc }),
            alive: true,
        }
    }

    pub fn chest(pos: Vec2, flag: u16, loot: Loot, open: bool) -> Self {
        Self {
            kind: EntityKind::Chest,
            pos,
            vel: Vec2::ZERO,
            facing: Dir4::Down,
            body: Some(Body {
                half: Vec2::new(8.0, 8.0),
                solid: true,
                layer: layer::ENEMY_BODY,
                mask: 0,
            }),
            health: None,
            knockback: Vec2::ZERO,
            anim: AnimState {
                frame: if open { 1 } else { 0 },
                ..AnimState::default()
            },
            data: EntityData::Chest(ChestData {
                flag,
                loot,
                open,
                open_anim: 0,
            }),
            alive: true,
        }
    }

    pub fn gem(pos: Vec2, id: u8, taken: bool) -> Self {
        Self {
            kind: EntityKind::Gem,
            pos,
            vel: Vec2::ZERO,
            facing: Dir4::Down,
            body: Some(Body {
                half: Vec2::new(8.0, 8.0),
                solid: true,
                layer: layer::ENEMY_BODY,
                mask: 0,
            }),
            health: None,
            knockback: Vec2::ZERO,
            anim: AnimState::default(),
            data: EntityData::Gem(GemData {
                id,
                taken,
                sealed: false,
            }),
            alive: true,
        }
    }

    pub fn center(&self) -> Vec2 {
        // Body is bottom-aligned for player (16×16 collision under 16×24 sprite).
        match self.kind {
            EntityKind::Player | EntityKind::Npc => Vec2::new(self.pos.x + 8.0, self.pos.y + 8.0),
            _ => {
                if let Some(b) = self.body {
                    Vec2::new(self.pos.x + b.half.x, self.pos.y + b.half.y)
                } else {
                    self.pos
                }
            }
        }
    }

    pub fn is_enemy(&self) -> bool {
        matches!(
            self.kind,
            EntityKind::Slime
                | EntityKind::Bat
                | EntityKind::Octorok
                | EntityKind::RaiderSpear
                | EntityKind::RaiderTorch
                | EntityKind::Wisp
                | EntityKind::Skeleton
                | EntityKind::Dummy
        )
    }
}
