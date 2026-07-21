use content::maps::MapId;
use serde::{Deserialize, Serialize};

pub const SAVE_KEY: &str = "shard_save_v1";
pub const SAVE_VERSION: u32 = 2;

/// Overworld fog grid: 60×60 cells (4×4 tiles), packed into u32 words.
pub const FOG_CELLS: usize = 60 * 60;
pub const FOG_WORDS: usize = FOG_CELLS.div_ceil(32);

/// Re-export content flag registry (single source of ids).
pub mod save_flags {
    pub use content::flags::*;
}

pub fn has_flag(flags: &[u16], id: u16) -> bool {
    flags.contains(&id)
}

pub fn set_flag(flags: &mut Vec<u16>, id: u16) -> bool {
    if has_flag(flags, id) {
        false
    } else {
        flags.push(id);
        true
    }
}

pub fn heart_piece_count(flags: &[u16]) -> u32 {
    [
        save_flags::HEART_PIECE_1,
        save_flags::HEART_PIECE_2,
        save_flags::HEART_PIECE_3,
        save_flags::HEART_PIECE_4,
    ]
    .iter()
    .filter(|f| has_flag(flags, **f))
    .count() as u32
}

/// Apply +1 max heart (2 half-units) once four pieces are collected.
pub fn maybe_apply_heart_container(flags: &mut Vec<u16>, max_hearts: &mut i32) -> bool {
    if heart_piece_count(flags) >= 4 && !has_flag(flags, save_flags::HEART_REWARD_APPLIED) {
        set_flag(flags, save_flags::HEART_REWARD_APPLIED);
        *max_hearts = (*max_hearts).saturating_add(2);
        true
    } else {
        false
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveGame {
    pub version: u32,
    pub map: u8,
    pub entry: u8,
    pub checkpoint: u8,
    pub hearts: i32,
    pub max_hearts: i32,
    pub rupees: u32,
    pub gems: u8,
    pub flags: Vec<u16>,
    /// Compact fog bitset (113 words). Missing/short → treated as empty fog.
    #[serde(default)]
    pub fog: Vec<u32>,
    #[serde(default)]
    pub bombs: u8,
    #[serde(default)]
    pub bomb_cap: u8,
    #[serde(default)]
    pub selected_item: u8,
    #[serde(default)]
    pub muted: bool,
}

impl SaveGame {
    pub fn default_spawn() -> Self {
        Self {
            version: SAVE_VERSION,
            map: MapId::Overworld.to_u8(),
            entry: 0,
            checkpoint: 0,
            hearts: 6,
            max_hearts: 6,
            rupees: 0,
            gems: 0,
            flags: Vec::new(),
            fog: vec![0; FOG_WORDS],
            bombs: 0,
            bomb_cap: 0,
            selected_item: 0,
            muted: false,
        }
    }

    /// Strict parse — `None` if missing/invalid/wrong version (no silent New Game).
    pub fn try_from_json(json: &str) -> Option<Self> {
        match serde_json::from_str::<SaveGame>(json) {
            Ok(mut s) if s.version == SAVE_VERSION => {
                if s.fog.len() < FOG_WORDS {
                    s.fog.resize(FOG_WORDS, 0);
                }
                Some(s)
            }
            _ => None,
        }
    }

    pub fn from_json(json: &str) -> Self {
        Self::try_from_json(json).unwrap_or_else(Self::default_spawn)
    }

    pub fn to_json(&self) -> Option<String> {
        serde_json::to_string(self).ok()
    }

    pub fn map_id(&self) -> MapId {
        MapId::from_u8(self.map)
    }
}
