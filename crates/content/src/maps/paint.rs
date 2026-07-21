//! Deterministic map painting helpers (no fastrand — tiny LCG).

use super::catalog;
use super::{MapDef, TileLayer};

/// Tiny LCG for deterministic organic fills. Same seed → same map.
pub struct Lcg {
    state: u32,
}

impl Lcg {
    pub fn new(seed: u32) -> Self {
        Self {
            state: seed.wrapping_mul(1664525).wrapping_add(1013904223) | 1,
        }
    }

    pub fn next_u32(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(1664525)
            .wrapping_add(1013904223);
        self.state
    }

    pub fn chance(&mut self, density: f32) -> bool {
        let t = (self.next_u32() as f32) / (u32::MAX as f32);
        t < density
    }

    pub fn range_i32(&mut self, lo: i32, hi: i32) -> i32 {
        if hi <= lo {
            return lo;
        }
        lo + (self.next_u32() as i32).rem_euclid(hi - lo + 1)
    }
}

pub fn hline(map: &mut MapDef, x0: u32, x1: u32, y: u32, tile: u16) {
    let (a, b) = if x0 <= x1 { (x0, x1) } else { (x1, x0) };
    for x in a..=b {
        map.set(x, y, TileLayer::Ground, tile);
    }
}

pub fn vline(map: &mut MapDef, x: u32, y0: u32, y1: u32, tile: u16) {
    let (a, b) = if y0 <= y1 { (y0, y1) } else { (y1, y0) };
    for y in a..=b {
        map.set(x, y, TileLayer::Ground, tile);
    }
}

/// Walkable road of given half-width along polyline points (tile coords).
pub fn path(map: &mut MapDef, points: &[(u32, u32)], width: u32, tile: u16) {
    if points.is_empty() {
        return;
    }
    let half = width.max(1) / 2;
    for w in points.windows(2) {
        let (x0, y0) = (w[0].0 as i32, w[0].1 as i32);
        let (x1, y1) = (w[1].0 as i32, w[1].1 as i32);
        let steps = (x1 - x0).abs().max((y1 - y0).abs()).max(1);
        for s in 0..=steps {
            let t = s as f32 / steps as f32;
            let cx = x0 as f32 + (x1 - x0) as f32 * t;
            let cy = y0 as f32 + (y1 - y0) as f32 * t;
            let r = half as i32;
            for dy in -r..=r {
                for dx in -r..=r {
                    if dx * dx + dy * dy <= r * r + r {
                        let x = (cx as i32 + dx).max(0) as u32;
                        let y = (cy as i32 + dy).max(0) as u32;
                        map.set(x, y, TileLayer::Ground, tile);
                    }
                }
            }
        }
    }
}

/// Organic circular-ish patch via LCG jitter.
pub fn blob(map: &mut MapDef, cx: u32, cy: u32, r: u32, tile: u16, seed: u32) {
    let mut rng = Lcg::new(seed ^ (cx * 374761393) ^ (cy * 668265263));
    let r = r as i32;
    for dy in -r..=r {
        for dx in -r..=r {
            let dist2 = dx * dx + dy * dy;
            let limit = r * r - rng.range_i32(0, r.max(1));
            if dist2 <= limit {
                let x = cx as i32 + dx;
                let y = cy as i32 + dy;
                if map.in_bounds(x, y) {
                    map.set(x as u32, y as u32, TileLayer::Ground, tile);
                }
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn scatter(
    map: &mut MapDef,
    x0: u32,
    y0: u32,
    x1: u32,
    y1: u32,
    tile: u16,
    density: f32,
    seed: u32,
) {
    let mut rng = Lcg::new(seed);
    for y in y0..=y1.min(map.height.saturating_sub(1)) {
        for x in x0..=x1.min(map.width.saturating_sub(1)) {
            if rng.chance(density) {
                map.set(x, y, TileLayer::Ground, tile);
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn scatter_detail(
    map: &mut MapDef,
    x0: u32,
    y0: u32,
    x1: u32,
    y1: u32,
    tile: u16,
    density: f32,
    seed: u32,
) {
    let mut rng = Lcg::new(seed);
    for y in y0..=y1.min(map.height.saturating_sub(1)) {
        for x in x0..=x1.min(map.width.saturating_sub(1)) {
            if rng.chance(density) && catalog::tile_info(map.ground[map.idx(x, y)]).flags == 0 {
                map.set(x, y, TileLayer::Detail, tile);
            }
        }
    }
}

/// Stamp a char-grid prefab. Legend maps char → (layer, tile). `'.'` skips.
pub fn stamp(
    map: &mut MapDef,
    ox: u32,
    oy: u32,
    rows: &[&str],
    legend: &[(char, TileLayer, u16)],
) {
    for (row_i, row) in rows.iter().enumerate() {
        for (col_i, ch) in row.chars().enumerate() {
            if ch == '.' || ch == ' ' {
                continue;
            }
            if let Some(&(_, layer, tile)) = legend.iter().find(|(c, _, _)| *c == ch) {
                map.set(ox + col_i as u32, oy + row_i as u32, layer, tile);
            }
        }
    }
}

/// Wide water channel along polyline.
pub fn river(map: &mut MapDef, points: &[(u32, u32)], width: u32, water: u16, shore: Option<u16>) {
    path(map, points, width, water);
    if let Some(shore_tile) = shore {
        let outer = width + 2;
        // Paint shore ring by checking neighbors of water — cheap second pass on bbox.
        let mut min_x = u32::MAX;
        let mut min_y = u32::MAX;
        let mut max_x = 0u32;
        let mut max_y = 0u32;
        for &(x, y) in points {
            min_x = min_x.min(x.saturating_sub(outer));
            min_y = min_y.min(y.saturating_sub(outer));
            max_x = max_x.max(x + outer);
            max_y = max_y.max(y + outer);
        }
        max_x = max_x.min(map.width.saturating_sub(1));
        max_y = max_y.min(map.height.saturating_sub(1));
        for y in min_y..=max_y {
            for x in min_x..=max_x {
                let i = map.idx(x, y);
                if map.ground[i] == water {
                    continue;
                }
                let mut near = false;
                for (dx, dy) in [(-1i32, 0), (1, 0), (0, -1), (0, 1)] {
                    let nx = x as i32 + dx;
                    let ny = y as i32 + dy;
                    if map.in_bounds(nx, ny)
                        && map.ground[map.idx(nx as u32, ny as u32)] == water
                    {
                        near = true;
                        break;
                    }
                }
                if near && catalog::tile_info(map.ground[i]).flags == 0 {
                    map.set(x, y, TileLayer::Ground, shore_tile);
                }
            }
        }
    }
}
