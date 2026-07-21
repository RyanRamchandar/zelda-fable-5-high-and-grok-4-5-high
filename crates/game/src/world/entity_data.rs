//! Per-family enemy / projectile data structs (extracted for file-cap headroom).

use crate::math::Vec2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlimeState {
    Idle,
    Chase,
    LungeWindup,
    Lunge,
    Recover,
}

#[derive(Clone, Debug)]
pub struct SlimeData {
    pub spawn_telegraph: u16,
    pub state: SlimeState,
    pub timer: u16,
    pub hop_phase: u16,
    pub stun_ticks: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BatState {
    Hover,
    SwoopTelegraph,
    Swoop,
    Climb,
}

#[derive(Clone, Debug)]
pub struct BatData {
    pub spawn_telegraph: u16,
    pub state: BatState,
    pub timer: u16,
    pub hover_phase: f32,
    pub swoop_origin: Vec2,
    pub swoop_target: Vec2,
    pub stun_ticks: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OctorokState {
    Idle,
    SpitTelegraph,
    Spit,
    Hide,
}

#[derive(Clone, Debug)]
pub struct OctorokData {
    pub spawn_telegraph: u16,
    pub state: OctorokState,
    pub timer: u16,
    pub cycle: u16,
    pub stun_ticks: u16,
}

#[derive(Clone, Debug)]
pub struct RockData {
    pub dir: Vec2,
    pub damage: f32,
    pub from_player: bool,
    pub hit: bool,
    pub swing_id: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RaiderSpearState {
    Idle,
    Approach,
    PokeTelegraph,
    Thrust,
    Guard,
}

#[derive(Clone, Debug)]
pub struct RaiderSpearData {
    pub spawn_telegraph: u16,
    pub state: RaiderSpearState,
    pub timer: u16,
    pub patrol_phase: f32,
    pub stun_ticks: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RaiderTorchState {
    Idle,
    ThrowTelegraph,
    Cooldown,
}

#[derive(Clone, Debug)]
pub struct RaiderTorchData {
    pub spawn_telegraph: u16,
    pub state: RaiderTorchState,
    pub timer: u16,
    pub stun_ticks: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WispState {
    Visible,
    FadeOut,
    Phased,
    FadeIn,
}

#[derive(Clone, Debug)]
pub struct WispData {
    pub spawn_telegraph: u16,
    pub state: WispState,
    pub timer: u16,
    pub stun_ticks: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkeletonState {
    Walk,
    PokeTelegraph,
    Lunge,
    Stagger,
}

#[derive(Clone, Debug)]
pub struct SkeletonData {
    pub spawn_telegraph: u16,
    pub state: SkeletonState,
    pub timer: u16,
    pub shield_up: bool,
    /// Remaining stagger duration when entering `Stagger`.
    pub stagger_len: u16,
}

#[derive(Clone, Debug)]
pub struct TorchProjData {
    pub dir: Vec2,
    pub life: u16,
    pub age: u16,
    pub hit: bool,
}

#[derive(Clone, Debug)]
pub struct TorchFlameData {
    pub life: u16,
    pub tick: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoomerangPhase {
    Out,
    Return,
}

#[derive(Clone, Debug)]
pub struct BoomerangData {
    pub dir: Vec2,
    pub phase: BoomerangPhase,
    pub traveled: f32,
    pub throw_id: u32,
    pub flame: bool,
    pub catch_buffer: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IronshellState {
    Advance,
    BashTelegraph,
    Bash,
    Recover,
    Stagger,
}

#[derive(Clone, Debug)]
pub struct IronshellData {
    pub spawn_telegraph: u16,
    pub state: IronshellState,
    pub timer: u16,
    pub stun_ticks: u16,
    pub stagger_len: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WardenPhase {
    One,
    Two,
    Three,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WardenAttack {
    Idle,
    SlamTele,
    Slam,
    RockFanTele,
    RockFan,
    SweepTele,
    Sweep,
    FakeFlash,
    Collapse,
}

#[derive(Clone, Debug)]
pub struct WardenData {
    pub phase: WardenPhase,
    pub attack: WardenAttack,
    pub timer: u16,
    pub core_exposed: u16,
    pub hp: f32,
    pub max_hp: f32,
    pub fake_armed: bool,
}

#[derive(Clone, Debug)]
pub struct WindCrystalData {
    pub perch: u8,
    pub primed: u16,
    pub throw_id_seen: u32,
    pub orbit_angle: f32,
}

#[derive(Clone, Debug)]
pub struct PebbleData {
    pub spawn_telegraph: u16,
    pub state: SlimeState,
    pub timer: u16,
}
