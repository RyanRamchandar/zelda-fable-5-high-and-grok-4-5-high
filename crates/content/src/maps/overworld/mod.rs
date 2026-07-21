//! Act 1 contiguous 240×240 overworld — terrain shells for Phase 2A.

mod camp;
mod cliffs;
mod connective;
mod grove;
mod ruins;
mod shrine;
mod village;

use super::catalog::{T_CLIFF_FACE, T_GRASS_A, T_TREE_TRUNK};
use super::{MapDef, TileLayer};

pub const OW_W: u32 = 240;
pub const OW_H: u32 = 240;

pub fn build() -> MapDef {
    let mut map = MapDef::new(OW_W, OW_H, T_GRASS_A);
    // Outer 2-tile solid border.
    for y in 0..OW_H {
        for x in 0..OW_W {
            if x < 2 || y < 2 || x >= OW_W - 2 || y >= OW_H - 2 {
                let tile = if y < 40 {
                    T_CLIFF_FACE
                } else {
                    T_TREE_TRUNK
                };
                map.set(x, y, TileLayer::Ground, tile);
            }
        }
    }

    connective::paint(&mut map);
    village::paint(&mut map);
    grove::paint(&mut map);
    camp::paint(&mut map);
    ruins::paint(&mut map);
    cliffs::paint(&mut map);
    shrine::paint(&mut map);

    // Re-seal outer border after region paints.
    for y in 0..OW_H {
        for x in 0..OW_W {
            if x < 2 || y < 2 || x >= OW_W - 2 || y >= OW_H - 2 {
                let tile = if y < 40 {
                    T_CLIFF_FACE
                } else {
                    T_TREE_TRUNK
                };
                map.set(x, y, TileLayer::Ground, tile);
            }
        }
    }

    map
}
