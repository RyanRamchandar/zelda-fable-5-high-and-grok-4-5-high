//! Map definitions and builders. Pure data — no engine/game imports.

mod arena;

pub use arena::{arena, FLOOR, FOUNTAIN, TILE_PX, WALL};

/// Tile map used by `game::world`. Tile ids are gray-box constants until 1B skins them.
#[derive(Clone, Debug)]
pub struct MapDef {
    pub width: u32,
    pub height: u32,
    pub ground: Vec<u16>,
    pub collision: Vec<bool>,
}

impl MapDef {
    pub fn new(width: u32, height: u32, fill_tile: u16) -> Self {
        let n = (width * height) as usize;
        Self {
            width,
            height,
            ground: vec![fill_tile; n],
            collision: vec![false; n],
        }
    }

    pub fn idx(&self, tx: u32, ty: u32) -> usize {
        (ty * self.width + tx) as usize
    }

    pub fn in_bounds(&self, tx: i32, ty: i32) -> bool {
        tx >= 0 && ty >= 0 && (tx as u32) < self.width && (ty as u32) < self.height
    }

    pub fn solid_at(&self, tx: i32, ty: i32) -> bool {
        if !self.in_bounds(tx, ty) {
            return true;
        }
        self.collision[self.idx(tx as u32, ty as u32)]
    }

    pub fn fill(&mut self, tile: u16) {
        for t in &mut self.ground {
            *t = tile;
        }
    }

    pub fn rect_border(&mut self, tile: u16, solid: bool) {
        let w = self.width;
        let h = self.height;
        for x in 0..w {
            self.set(x, 0, tile, solid);
            self.set(x, h - 1, tile, solid);
        }
        for y in 0..h {
            self.set(0, y, tile, solid);
            self.set(w - 1, y, tile, solid);
        }
    }

    pub fn fill_rect(&mut self, x0: u32, y0: u32, x1: u32, y1: u32, tile: u16, solid: bool) {
        for y in y0..=y1.min(self.height.saturating_sub(1)) {
            for x in x0..=x1.min(self.width.saturating_sub(1)) {
                self.set(x, y, tile, solid);
            }
        }
    }

    pub fn set(&mut self, x: u32, y: u32, tile: u16, solid: bool) {
        if x >= self.width || y >= self.height {
            return;
        }
        let i = self.idx(x, y);
        self.ground[i] = tile;
        self.collision[i] = solid;
    }
}
