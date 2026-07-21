use serde::{Deserialize, Serialize};

pub const SAVE_KEY: &str = "shard_save_v1";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveGame {
    pub x: f32,
    pub y: f32,
}

impl SaveGame {
    pub fn default_spawn() -> Self {
        // Open combat field south of the center pillar (60×34 tiles @ 16px).
        Self { x: 480.0, y: 300.0 }
    }

    pub fn from_json(json: &str) -> Self {
        serde_json::from_str(json).unwrap_or_else(|_| Self::default_spawn())
    }

    pub fn to_json(&self) -> Option<String> {
        serde_json::to_string(self).ok()
    }
}
