//! Live combat / player feel constants. Start from GAME_DESIGN; ±30% ok if logged.

// --- Movement ---
pub const RUN_SPEED: f32 = 1.5;
pub const ACCEL_TICKS: f32 = 10.0;
pub const DECEL_TICKS: f32 = 10.0;
pub const SHIELD_MOVE_SPEED: f32 = 0.6;

// --- Dash ---
pub const DASH_SPEED: f32 = 4.5;
pub const DASH_DURATION: u16 = 10;
pub const DASH_RECOVERY: u16 = 6;
pub const DASH_IFRAME_START: u16 = 2;
pub const DASH_IFRAME_END: u16 = 8;
pub const DASH_ENERGY: f32 = 25.0;
pub const DASH_ENERGY_A_PLUS: f32 = 20.0;
pub const DASH_CANCEL_SLASH_FROM: u16 = 6;

// --- Sword ---
pub const SLASH_TICKS: u16 = 12;
pub const SLASH_ACTIVE_START: u16 = 3;
pub const SLASH_ACTIVE_END: u16 = 9;
pub const SLASH_BUFFER_TICKS: u16 = 6;
pub const SLASH_HIT_HALF: f32 = 10.0;
pub const SLASH_REACH: f32 = 14.0;
pub const SLASH_DAMAGE: f32 = 1.0;
pub const COMBO_DROP_WINDOW: u16 = 20;
pub const FINISHER_DAMAGE_MULT: f32 = 1.5;
pub const FINISHER_KB_MULT: f32 = 1.6;
pub const FINISHER_HOP_PX: f32 = 8.0;
pub const FINISHER_HOP_TICKS: u8 = 4;

pub const CHARGE_TICKS: u16 = 30;
pub const SPIN_ENERGY: f32 = 20.0;
pub const SPIN_TICKS: u16 = 14;
pub const SPIN_RADIUS: f32 = 28.0;
pub const SPIN_DAMAGE_MULT: f32 = 2.0;

pub const BEAM_SPEED: f32 = 3.0;
pub const BEAM_RANGE: f32 = 128.0;
pub const BEAM_DAMAGE_MULT: f32 = 0.75;

// --- Shield ---
pub const PERFECT_BLOCK_WINDOW: u16 = 6;
pub const BLOCK_ENERGY: f32 = 5.0;
pub const PERFECT_BLOCK_REFUND: f32 = 10.0;
pub const BLOCK_PUSHBACK: f32 = 1.2;

// --- Knockback / hitstop ---
pub const KB_QUICK: f32 = 2.0;
pub const KB_FINISHER: f32 = 3.2;
pub const KB_SPIN: f32 = 4.0;
pub const HITSTOP_NORMAL: u8 = 3;
pub const HITSTOP_HEAVY: u8 = 5;
#[allow(dead_code)] // reserved for boss phase breaks
pub const HITSTOP_BOSS_BREAK: u8 = 12;
pub const FLASH_TICKS: u8 = 4;
pub const PLAYER_IFRAMES: u16 = 60;

// --- Energy ---
pub const ENERGY_MAX: f32 = 100.0;
pub const ENERGY_REGEN_PER_TICK: f32 = 0.2; // 12/s
pub const ENERGY_REGEN_DELAY: u16 = 30;
pub const ENERGY_DENY_FLASH: u8 = 10;
pub const FOUNTAIN_ENERGY_PER_TICK: f32 = 2.0;
pub const FOUNTAIN_HEART_INTERVAL: u16 = 30;

// --- Style thresholds (points) ---
pub const STYLE_C: f32 = 20.0;
pub const STYLE_B: f32 = 50.0;
pub const STYLE_A: f32 = 90.0;
pub const STYLE_S: f32 = 140.0;
pub const STYLE_VERB_WINDOW: u16 = 180;
pub const STYLE_OUT_OF_COMBAT: u16 = 300;
pub const STYLE_DECAY_INTERVAL: u16 = 90;
pub const STYLE_STREAK_INTERVAL: u16 = 120;

// --- Dummy / pickups ---
pub const DUMMY_HP: i32 = 20;
pub const DUMMY_RESPAWN: u16 = 120;
pub const PICKUP_MAGNET: f32 = 24.0;
pub const PICKUP_LIFE: u16 = 600;
pub const PICKUP_BLINK: u16 = 120;
pub const ENERGY_ORB: f32 = 15.0;

// --- Debug shot ---
pub const DEBUG_SHOT_SPEED: f32 = 1.2;
pub const DEBUG_SHOT_DAMAGE: f32 = 1.0;

// --- Enemies (Phase 1B) ---
pub const SLIME_HP: i32 = 3;
pub const SLIME_WANDER: f32 = 0.3;
pub const SLIME_CHASE: f32 = 0.7;
pub const SLIME_LUNGE_SPEED: f32 = 4.0;
pub const SLIME_CHASE_RANGE: f32 = 96.0;
pub const SLIME_LUNGE_RANGE: f32 = 28.0;
pub const SLIME_HOP_MOVE: u16 = 12;
pub const SLIME_HOP_REST: u16 = 8;
pub const SLIME_WINDUP: u16 = 20;
pub const SLIME_LUNGE_TICKS: u16 = 8;
pub const SLIME_RECOVER: u16 = 20;
pub const SLIME_CONTACT: i32 = 1; // half-heart

pub const BAT_HP: i32 = 2;
pub const BAT_HOVER_DRIFT: f32 = 0.35;
pub const BAT_SWOOP_SPEED: f32 = 2.2;
pub const BAT_SWOOP_PERIOD: u16 = 180;
pub const BAT_TELEGRAPH: u16 = 20;
pub const BAT_SWOOP_TICKS: u16 = 40;
pub const BAT_CLIMB_TICKS: u16 = 30;
pub const BAT_CONTACT: i32 = 1;

pub const OCTOROK_HP: i32 = 3;
pub const OCTOROK_CYCLE: u16 = 150;
pub const OCTOROK_SPIT_TELEGRAPH: u16 = 20;
pub const OCTOROK_HIDE: u16 = 60;
pub const OCTOROK_ROCK_SPEED: f32 = 2.5;
pub const OCTOROK_ROCK_DAMAGE: f32 = 1.0;
pub const OCTOROK_ROCK_REFLECT_DAMAGE: f32 = 1.0;

pub const SPAWN_TELEGRAPH: u16 = 45;
pub const WAVE_LULL: u16 = 120;
pub const WAVE_ALIVE_CAP: usize = 10;
pub const ENEMY_IFRAMES: u16 = 10;
