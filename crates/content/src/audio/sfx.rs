//! SFX ids and plain-data specs. Engine playback goes through the app adapter.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SfxId {
    Slash1,
    Slash2,
    Finisher,
    ChargeReady,
    SpinRelease,
    Beam,
    Dash,
    Refused,
    ShieldBlock,
    PerfectBlock,
    HitEnemy,
    HurtPlayer,
    Kill,
    PickupRupee,
    PickupHeart,
    PickupEnergy,
    RankUp,
    RankDown,
    FountainChime,
    // Phase 1B enemies / waves
    SlimeSquish,
    SlimeLunge,
    BatSqueak,
    BatSwoop,
    OctorokSpit,
    OctorokDuck,
    EnemyHurt,
    SpawnShimmer,
    WaveCue,
    ReflectZing,
    LedgeHop,
    CheckpointChime,
    TextBlip,
    ChestOpen,
    GemGet,
    SealOpen,
    SecretChime,
}

#[derive(Clone, Copy, Debug)]
pub enum OscKind {
    Square,
    Triangle,
    Saw,
    Sine,
    Noise,
}

#[derive(Clone, Copy, Debug)]
pub struct SfxSpec {
    pub osc: OscKind,
    pub freq_start: f32,
    pub freq_end: f32,
    pub attack_s: f32,
    pub decay_s: f32,
    pub gain: f32,
    pub noise_mix: f32,
}

pub fn spec(id: SfxId) -> SfxSpec {
    match id {
        SfxId::Slash1 => saw(420.0, 280.0, 0.01, 0.07, 0.18),
        SfxId::Slash2 => saw(380.0, 240.0, 0.01, 0.08, 0.18),
        SfxId::Finisher => saw(520.0, 180.0, 0.01, 0.12, 0.22),
        SfxId::ChargeReady => sine(660.0, 880.0, 0.02, 0.1, 0.14),
        SfxId::SpinRelease => saw(300.0, 140.0, 0.01, 0.16, 0.24),
        SfxId::Beam => sine(880.0, 440.0, 0.005, 0.1, 0.12),
        SfxId::Dash => noise(200.0, 120.0, 0.005, 0.08, 0.12, 0.7),
        SfxId::Refused => square(90.0, 70.0, 0.01, 0.12, 0.1),
        SfxId::ShieldBlock => square(220.0, 160.0, 0.005, 0.06, 0.14),
        SfxId::PerfectBlock => sine(520.0, 780.0, 0.01, 0.1, 0.16),
        SfxId::HitEnemy => saw(180.0, 90.0, 0.005, 0.07, 0.16),
        SfxId::HurtPlayer => square(140.0, 60.0, 0.01, 0.14, 0.18),
        SfxId::Kill => noise(160.0, 40.0, 0.01, 0.12, 0.14, 0.85),
        SfxId::PickupRupee => sine(880.0, 1320.0, 0.005, 0.08, 0.1),
        SfxId::PickupHeart => sine(660.0, 990.0, 0.01, 0.1, 0.12),
        SfxId::PickupEnergy => sine(440.0, 660.0, 0.01, 0.09, 0.11),
        SfxId::RankUp => sine(440.0, 880.0, 0.02, 0.14, 0.14),
        SfxId::RankDown => square(330.0, 160.0, 0.02, 0.14, 0.12),
        SfxId::FountainChime => sine(740.0, 980.0, 0.02, 0.16, 0.08),
        SfxId::SlimeSquish => noise(140.0, 80.0, 0.01, 0.09, 0.12, 0.55),
        SfxId::SlimeLunge => saw(220.0, 360.0, 0.01, 0.1, 0.16),
        SfxId::BatSqueak => sine(980.0, 1240.0, 0.005, 0.07, 0.1),
        SfxId::BatSwoop => saw(640.0, 280.0, 0.01, 0.12, 0.14),
        SfxId::OctorokSpit => square(180.0, 90.0, 0.01, 0.1, 0.14),
        SfxId::OctorokDuck => noise(100.0, 60.0, 0.01, 0.08, 0.1, 0.4),
        SfxId::EnemyHurt => saw(200.0, 110.0, 0.005, 0.08, 0.14),
        SfxId::SpawnShimmer => sine(520.0, 780.0, 0.02, 0.14, 0.1),
        SfxId::WaveCue => square(330.0, 520.0, 0.02, 0.16, 0.14),
        SfxId::ReflectZing => sine(880.0, 1320.0, 0.005, 0.09, 0.14),
        SfxId::LedgeHop => noise(180.0, 90.0, 0.005, 0.1, 0.12, 0.55),
        SfxId::CheckpointChime => sine(660.0, 990.0, 0.02, 0.18, 0.1),
        SfxId::TextBlip => square(880.0, 920.0, 0.001, 0.03, 0.06),
        SfxId::ChestOpen => saw(240.0, 480.0, 0.01, 0.14, 0.16),
        SfxId::GemGet => sine(440.0, 1320.0, 0.02, 0.28, 0.18),
        SfxId::SealOpen => saw(160.0, 520.0, 0.02, 0.32, 0.2),
        SfxId::SecretChime => sine(740.0, 1180.0, 0.015, 0.22, 0.14),
    }
}

fn saw(fs: f32, fe: f32, a: f32, d: f32, g: f32) -> SfxSpec {
    SfxSpec {
        osc: OscKind::Saw,
        freq_start: fs,
        freq_end: fe,
        attack_s: a,
        decay_s: d,
        gain: g,
        noise_mix: 0.0,
    }
}

fn sine(fs: f32, fe: f32, a: f32, d: f32, g: f32) -> SfxSpec {
    SfxSpec {
        osc: OscKind::Sine,
        freq_start: fs,
        freq_end: fe,
        attack_s: a,
        decay_s: d,
        gain: g,
        noise_mix: 0.0,
    }
}

fn square(fs: f32, fe: f32, a: f32, d: f32, g: f32) -> SfxSpec {
    SfxSpec {
        osc: OscKind::Square,
        freq_start: fs,
        freq_end: fe,
        attack_s: a,
        decay_s: d,
        gain: g,
        noise_mix: 0.0,
    }
}

fn noise(fs: f32, fe: f32, a: f32, d: f32, g: f32, mix: f32) -> SfxSpec {
    SfxSpec {
        osc: OscKind::Noise,
        freq_start: fs,
        freq_end: fe,
        attack_s: a,
        decay_s: d,
        gain: g,
        noise_mix: mix,
    }
}
