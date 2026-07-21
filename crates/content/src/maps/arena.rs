//! Gray-box combat arena for Phase 1A feel testing.

use super::MapDef;

pub const TILE_PX: f32 = 16.0;
pub const FLOOR: u16 = 0;
pub const WALL: u16 = 1;
pub const FOUNTAIN: u16 = 2;

/// 60×34 tiles (960×544 px) — ~2×2 screens with room for camera travel.
pub fn arena() -> MapDef {
    let mut map = MapDef::new(60, 34, FLOOR);
    map.rect_border(WALL, true);

    // Interior 2×2 pillars for circling (offset from spawn field).
    for (px, py) in [(18u32, 10), (40, 10), (18, 22), (40, 22), (28, 14)] {
        map.fill_rect(px, py, px + 1, py + 1, WALL, true);
    }

    // Fountain corner (NW open pocket).
    map.fill_rect(3, 3, 6, 6, FLOOR, false);
    map.set(4, 4, FOUNTAIN, false);
    map.set(5, 4, FOUNTAIN, false);
    map.set(4, 5, FOUNTAIN, false);
    map.set(5, 5, FOUNTAIN, false);

    map
}
