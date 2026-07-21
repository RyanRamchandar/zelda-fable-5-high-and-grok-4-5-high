use content::maps::MapId;
use serde::{Deserialize, Serialize};

pub const SAVE_KEY: &str = "shard_save_v1";
pub const SAVE_VERSION: u32 = 2;

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
        }
    }

    pub fn from_json(json: &str) -> Self {
        match serde_json::from_str::<SaveGame>(json) {
            Ok(s) if s.version == SAVE_VERSION => s,
            _ => Self::default_spawn(),
        }
    }

    pub fn to_json(&self) -> Option<String> {
        serde_json::to_string(self).ok()
    }

    pub fn map_id(&self) -> MapId {
        MapId::from_u8(self.map)
    }
}
