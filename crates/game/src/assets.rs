//! Decode `content::art` grids → RGBA → `engine::atlas`. App sets atlas on `Draw`.

use std::collections::HashMap;

use content::art::palette::{self, PaletteSwap};
use content::art::{self, SpriteBake, SpriteDef};
use engine::atlas::{Atlas, AtlasBuilder, SpriteHandle};

#[derive(Clone, Debug)]
pub struct SpriteMap {
    by_key: HashMap<&'static str, SpriteHandle>,
    /// Ordered keys for F2 viewer (unique bake keys).
    pub viewer_keys: Vec<&'static str>,
}

impl SpriteMap {
    pub fn get(&self, key: &str) -> Option<SpriteHandle> {
        self.by_key.get(key).copied()
    }

    pub fn must(&self, key: &str) -> SpriteHandle {
        self.get(key).unwrap_or_else(|| {
            // Fallback: first available (should never hit if bake complete).
            *self.by_key.values().next().expect("empty sprite map")
        })
    }
}

pub struct BakedAssets {
    pub atlas: Atlas,
    pub sprites: SpriteMap,
}

pub fn bake() -> Result<BakedAssets, String> {
    let mut builder = AtlasBuilder::new(1024, 1024)?;
    let mut by_key = HashMap::new();
    let mut viewer_keys = Vec::new();

    for bake in art::all_bakes() {
        let pixels = decode_strip(bake.def, bake.swap)?;
        let handle = builder.add_strip(bake.def.w, bake.def.h, bake.def.frames, &pixels)?;
        by_key.insert(bake.key, handle);
        viewer_keys.push(bake.key);
    }

    let mut missing = Vec::new();
    for &id in content::maps::catalog::all_tile_ids() {
        let info = content::maps::catalog::tile_info(id);
        if info.sprite.is_empty() {
            continue;
        }
        if !by_key.contains_key(info.sprite) {
            missing.push(info.sprite);
        }
    }
    if !missing.is_empty() {
        return Err(format!("tile sprites missing from atlas: {missing:?}"));
    }

    Ok(BakedAssets {
        atlas: builder.finish(),
        sprites: SpriteMap { by_key, viewer_keys },
    })
}

fn decode_strip(def: &SpriteDef, swap: Option<&PaletteSwap>) -> Result<Vec<u8>, String> {
    let expect_w = def.w * def.frames;
    let mut rows: Vec<&str> = Vec::new();
    for line in def.grid.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        if line.chars().count() != expect_w as usize {
            return Err(format!(
                "{}: row width {} != {}",
                def.name,
                line.chars().count(),
                expect_w
            ));
        }
        rows.push(line);
    }
    if rows.len() != def.h as usize {
        return Err(format!(
            "{}: row count {} != {}",
            def.name,
            rows.len(),
            def.h
        ));
    }

    let mut pixels = Vec::with_capacity((expect_w * def.h * 4) as usize);
    for row in rows {
        for ch in row.chars() {
            let idx = palette::char_to_index(ch).ok_or_else(|| {
                format!("{}: bad char {:?}", def.name, ch)
            })?;
            let idx = palette::apply_swap(idx, swap);
            let rgba = palette::rgba(idx);
            pixels.extend_from_slice(&rgba);
        }
    }
    Ok(pixels)
}

/// Viewer metadata: which bake keys play as animation strips.
pub fn viewer_entries() -> Vec<SpriteBake> {
    art::all_bakes()
}
