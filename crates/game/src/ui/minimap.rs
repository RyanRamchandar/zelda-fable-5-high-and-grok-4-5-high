//! Overworld fog minimap (corner + pause) with POI / objective markers.

use content::maps::catalog::{self, TileInfo};
use content::maps::{MapDef, MapId, TILE_PX};
use content::text::TextId;
use engine::input::{InputState, BUTTON_CONFIRM, BUTTON_PAUSE};
use engine::render::Draw;

use crate::assets::SpriteMap;
use crate::math::Vec2;
use crate::save_data::{has_flag, save_flags, FOG_CELLS, FOG_WORDS};
use crate::world::World;

const CELL: u32 = 4; // tiles per fog cell
const GRID: usize = 60;
const REVEAL_R: i32 = 10;
const REVEAL_EVERY: u64 = 8;

/// Terrain class for minimap coloring.
const C_GRASS: u8 = 0;
const C_PATH: u8 = 1;
const C_WATER: u8 = 2;
const C_FOREST: u8 = 3;
const C_CLIFF: u8 = 4;
const C_STRUCT: u8 = 5;
const C_SAND: u8 = 6;

pub struct MinimapState {
    pub show_corner: bool,
    pub pause_open: bool,
    fog: Vec<u32>,
    class_map: Vec<u8>,
    pub objective: Option<(u32, u32, TextId)>,
    brazier_a: u64,
    brazier_b: u64,
    secrets_discovered: u8,
}

impl MinimapState {
    pub fn new() -> Self {
        Self {
            show_corner: true,
            pause_open: false,
            fog: vec![0; FOG_WORDS],
            class_map: vec![C_GRASS; GRID * GRID],
            objective: None,
            brazier_a: 0,
            brazier_b: 0,
            secrets_discovered: 0,
        }
    }

    pub fn load_fog(&mut self, fog: &[u32]) {
        self.fog = vec![0; FOG_WORDS];
        for (i, w) in fog.iter().take(FOG_WORDS).enumerate() {
            self.fog[i] = *w;
        }
    }

    pub fn fog_bits(&self) -> Vec<u32> {
        self.fog.clone()
    }

    pub fn build_class_map(&mut self, map: &MapDef) {
        self.class_map = vec![C_GRASS; GRID * GRID];
        if map.width < 240 || map.height < 240 {
            return;
        }
        for cy in 0..GRID {
            for cx in 0..GRID {
                let tx = (cx as u32) * CELL + 2;
                let ty = (cy as u32) * CELL + 2;
                let id = map.ground[map.idx(tx.min(map.width - 1), ty.min(map.height - 1))];
                self.class_map[cy * GRID + cx] = class_of(id);
            }
        }
    }

    pub fn update(
        &mut self,
        input: &InputState,
        world: &World,
        map_id: MapId,
        gems: u8,
        flags: &[u16],
    ) {
        if input.minimap_toggle {
            self.show_corner = !self.show_corner;
        }
        if input.buttons[BUTTON_PAUSE].pressed || input.buttons[BUTTON_CONFIRM].pressed {
            self.pause_open = !self.pause_open;
        }

        if map_id != MapId::Overworld {
            return;
        }
        if !world.tick.is_multiple_of(REVEAL_EVERY) {
            self.refresh_objective(gems, flags);
            return;
        }
        if let Some(p) = world.get(world.player_id) {
            let cx = (p.pos.x / (TILE_PX * CELL as f32)).floor() as i32;
            let cy = (p.pos.y / (TILE_PX * CELL as f32)).floor() as i32;
            for dy in -REVEAL_R..=REVEAL_R {
                for dx in -REVEAL_R..=REVEAL_R {
                    if dx * dx + dy * dy > REVEAL_R * REVEAL_R {
                        continue;
                    }
                    let x = cx + dx;
                    let y = cy + dy;
                    if x >= 0 && y >= 0 && x < GRID as i32 && y < GRID as i32 {
                        set_fog(&mut self.fog, (y as usize) * GRID + x as usize);
                    }
                }
            }
        }
        self.refresh_objective(gems, flags);
    }

    pub fn refresh_objective(&mut self, gems: u8, flags: &[u16]) {
        if !has_flag(flags, save_flags::QUEST_STARTED) {
            self.objective = None;
            return;
        }
        if gems.count_ones() >= 3 {
            self.objective = Some((120, 10, TextId::WaypostShrine));
            return;
        }
        // Nearest unclaimed gem: Courage(74,114), Power(203,60), Wisdom(198,150)
        let sites = [(0u8, 74u32, 114u32), (1, 203, 60), (2, 198, 150)];
        for (id, tx, ty) in sites {
            if gems & (1 << id) == 0 {
                self.objective = Some((tx, ty, TextId::ElderIntro));
                return;
            }
        }
        self.objective = None;
    }

    pub fn note_brazier(&mut self, tick: u64) {
        // Alternate slots by proximity in time — first press A, second within 5s links.
        if self.brazier_a == 0 || tick.saturating_sub(self.brazier_a) > 300 {
            self.brazier_a = tick;
            self.brazier_b = 0;
        } else {
            self.brazier_b = tick;
        }
    }

    pub fn braziers_linked(&self, tick: u64) -> bool {
        self.brazier_a > 0
            && self.brazier_b > 0
            && self.brazier_b.saturating_sub(self.brazier_a) <= 300
            && tick.saturating_sub(self.brazier_b) < 30
    }

    pub fn mark_secret(&mut self, _flag: u16) {
        self.secrets_discovered = self.secrets_discovered.saturating_add(1);
    }

    pub fn mark_discovered_secret(&mut self) {
        self.secrets_discovered = self.secrets_discovered.saturating_add(1);
    }

    pub fn render_corner(
        &self,
        d: &mut Draw,
        world: &World,
        sprites: &SpriteMap,
        map_id: MapId,
        gems: u8,
        flags: &[u16],
    ) {
        if !self.show_corner || self.pause_open || map_id != MapId::Overworld {
            return;
        }
        let ox = 408.0;
        let oy = 8.0;
        d.rect(ox - 2.0, oy - 2.0, 68.0, 68.0, "#101820");
        draw_fog_panel(d, &self.fog, &self.class_map, ox, oy, 1.0);
        draw_pois(d, sprites, &self.fog, ox, oy, 1.0, gems, flags, self.objective);
        if let Some(p) = world.get(world.player_id) {
            let (px, py) = world_to_cell(p.pos);
            let blink = (world.tick / 16).is_multiple_of(2);
            if blink {
                d.rect(ox + px as f32, oy + py as f32, 2.0, 2.0, "#ffffff");
            }
            // Camera view outline (~30×17 cells of 480×270 / 64).
            let vx = (px as i32 - 7).clamp(0, 52) as f32;
            let vy = (py as i32 - 4).clamp(0, 52) as f32;
            d.rect(ox + vx, oy + vy, 1.0, 15.0, "#ffffff");
            d.rect(ox + vx + 14.0, oy + vy, 1.0, 15.0, "#ffffff");
            d.rect(ox + vx, oy + vy, 15.0, 1.0, "#ffffff");
            d.rect(ox + vx, oy + vy + 14.0, 15.0, 1.0, "#ffffff");
        }
        draw_objective_arrow(d, self.objective, world, ox, oy, 64.0);
    }

    pub fn render_pause(
        &self,
        d: &mut Draw,
        world: &World,
        sprites: &SpriteMap,
        map_id: MapId,
        gems: u8,
        flags: &[u16],
    ) {
        if !self.pause_open || map_id != MapId::Overworld {
            return;
        }
        d.rect(0.0, 0.0, 480.0, 270.0, "rgba(0,0,0,0.55)");
        let ox = 150.0;
        let oy = 40.0;
        d.rect(ox - 4.0, oy - 16.0, 188.0, 200.0, "#182028");
        d.text("MAP", ox + 70.0, oy - 12.0, "#e0e0c0");
        draw_fog_panel(d, &self.fog, &self.class_map, ox, oy, 3.0);
        draw_pois(d, sprites, &self.fog, ox, oy, 3.0, gems, flags, self.objective);
        // Region labels on revealed cells.
        for (name, cx, cy) in [
            ("Village", 30usize, 45usize),
            ("Grove", 12, 35),
            ("Camp", 50, 18),
            ("Ruins", 48, 36),
            ("Cliffs", 25, 12),
            ("Shrine", 30, 4),
        ] {
            if fog_get(&self.fog, cy * GRID + cx) {
                d.text(
                    name,
                    ox + cx as f32 * 3.0,
                    oy + cy as f32 * 3.0,
                    "#c0d0e0",
                );
            }
        }
        if let Some(p) = world.get(world.player_id) {
            let (px, py) = world_to_cell(p.pos);
            d.rect(
                ox + px as f32 * 3.0,
                oy + py as f32 * 3.0,
                3.0,
                3.0,
                "#ffffff",
            );
        }
        d.text("Esc/Enter: resume", ox + 20.0, oy + 185.0, "#a0a090");
        d.text("shop  fountain  shrine  gem", ox - 10.0, oy + 198.0, "#808070");
        let _ = sprites;
        let _ = FOG_CELLS;
    }
}

impl Default for MinimapState {
    fn default() -> Self {
        Self::new()
    }
}

fn class_of(id: u16) -> u8 {
    let info: &TileInfo = catalog::tile_info(id);
    if info.flags & content::maps::flags::WATER != 0 {
        return C_WATER;
    }
    match id {
        catalog::T_PATH
        | catalog::T_PATH_N
        | catalog::T_PATH_S
        | catalog::T_PATH_E
        | catalog::T_PATH_W
        | catalog::T_BRIDGE_H
        | catalog::T_BRIDGE_V => C_PATH,
        catalog::T_TREE_TRUNK
        | catalog::T_CANOPY_NW
        | catalog::T_CANOPY_NE
        | catalog::T_CANOPY_SW
        | catalog::T_CANOPY_SE => C_FOREST,
        catalog::T_CLIFF_TOP
        | catalog::T_CLIFF_FACE
        | catalog::T_CLIFF_EDGE_N
        | catalog::T_CLIFF_EDGE_S
        | catalog::T_CLIFF_EDGE_E
        | catalog::T_CLIFF_EDGE_W
        | catalog::T_CLIFF_STAIRS => C_CLIFF,
        catalog::T_HOUSE_WALL
        | catalog::T_HOUSE_DOOR
        | catalog::T_FENCE
        | catalog::T_SHRINE_STONE
        | catalog::T_DOOR_SEALED
        | catalog::T_COLUMN => C_STRUCT,
        catalog::T_SAND | catalog::T_DIRT | catalog::T_DIRT_ASH => C_SAND,
        _ => C_GRASS,
    }
}

fn color(c: u8) -> &'static str {
    match c {
        C_PATH => "#c8b080",
        C_WATER => "#3060a0",
        C_FOREST => "#206030",
        C_CLIFF => "#706050",
        C_STRUCT => "#a09070",
        C_SAND => "#b0a070",
        _ => "#3a6a3a",
    }
}

fn set_fog(fog: &mut [u32], idx: usize) {
    let w = idx / 32;
    let b = idx % 32;
    if w < fog.len() {
        fog[w] |= 1 << b;
    }
}

fn fog_get(fog: &[u32], idx: usize) -> bool {
    let w = idx / 32;
    let b = idx % 32;
    w < fog.len() && (fog[w] & (1 << b)) != 0
}

fn world_to_cell(pos: Vec2) -> (usize, usize) {
    let cx = (pos.x / (TILE_PX * CELL as f32)).floor().clamp(0.0, 59.0) as usize;
    let cy = (pos.y / (TILE_PX * CELL as f32)).floor().clamp(0.0, 59.0) as usize;
    (cx, cy)
}

fn draw_fog_panel(d: &mut Draw, fog: &[u32], class_map: &[u8], ox: f32, oy: f32, scale: f32) {
    for y in 0..GRID {
        let mut x = 0usize;
        while x < GRID {
            let idx = y * GRID + x;
            if !fog_get(fog, idx) {
                // dark run
                let mut x2 = x + 1;
                while x2 < GRID && !fog_get(fog, y * GRID + x2) {
                    x2 += 1;
                }
                d.rect(
                    ox + x as f32 * scale,
                    oy + y as f32 * scale,
                    (x2 - x) as f32 * scale,
                    scale,
                    "#0a1018",
                );
                x = x2;
                continue;
            }
            let c = class_map.get(idx).copied().unwrap_or(C_GRASS);
            let mut x2 = x + 1;
            while x2 < GRID
                && fog_get(fog, y * GRID + x2)
                && class_map.get(y * GRID + x2).copied() == Some(c)
            {
                x2 += 1;
            }
            d.rect(
                ox + x as f32 * scale,
                oy + y as f32 * scale,
                (x2 - x) as f32 * scale,
                scale,
                color(c),
            );
            x = x2;
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn draw_pois(
    d: &mut Draw,
    sprites: &SpriteMap,
    fog: &[u32],
    ox: f32,
    oy: f32,
    scale: f32,
    gems: u8,
    flags: &[u16],
    objective: Option<(u32, u32, TextId)>,
) {
    let pois: &[(u32, u32, &str, bool)] = &[
        (120, 188, "poi_shop", true),
        (119, 175, "poi_fountain", true),
        (120, 10, "poi_shrine", true),
        (74, 114, "poi_gem", gems & 1 == 0),
        (203, 60, "poi_gem", gems & 2 == 0),
        (198, 150, "poi_gem", gems & 4 == 0),
    ];
    for &(tx, ty, key, show) in pois {
        if !show {
            // claimed gem → checkmark
            if key == "poi_gem" {
                blit_poi(d, sprites, "poi_check", tx, ty, ox, oy, scale, true, fog);
            }
            continue;
        }
        let always = key == "poi_gem" && has_flag(flags, save_flags::QUEST_STARTED);
        blit_poi(d, sprites, key, tx, ty, ox, oy, scale, always, fog);
    }
    if let Some((tx, ty, _)) = objective {
        blit_poi(d, sprites, "poi_star", tx, ty, ox, oy, scale, true, fog);
    }
}

#[allow(clippy::too_many_arguments)]
fn blit_poi(
    d: &mut Draw,
    sprites: &SpriteMap,
    key: &str,
    tx: u32,
    ty: u32,
    ox: f32,
    oy: f32,
    scale: f32,
    always: bool,
    fog: &[u32],
) {
    let cx = (tx / CELL) as usize;
    let cy = (ty / CELL) as usize;
    if !always && !fog_get(fog, cy * GRID + cx) {
        return;
    }
    let x = ox + cx as f32 * scale - 1.0;
    let y = oy + cy as f32 * scale - 1.0;
    if let Some(h) = sprites.get(key) {
        d.sprite(h, 0, x, y, false);
    } else {
        d.rect(x, y, 3.0_f32.max(scale), 3.0_f32.max(scale), "#ffff80");
    }
}

fn draw_objective_arrow(
    d: &mut Draw,
    objective: Option<(u32, u32, TextId)>,
    world: &World,
    ox: f32,
    oy: f32,
    size: f32,
) {
    let Some((tx, ty, _)) = objective else {
        return;
    };
    let Some(p) = world.get(world.player_id) else {
        return;
    };
    let (pcx, pcy) = world_to_cell(p.pos);
    let tcx = (tx / CELL) as i32;
    let tcy = (ty / CELL) as i32;
    let dx = tcx - pcx as i32;
    let dy = tcy - pcy as i32;
    if dx.abs() < 30 && dy.abs() < 30 {
        return; // on panel-ish
    }
    let ang_x = if dx > 0 { 1.0 } else { -1.0 };
    let ang_y = if dy > 0 { 1.0 } else { -1.0 };
    let ax = ox + size / 2.0 + ang_x * (size / 2.0 - 4.0);
    let ay = oy + size / 2.0 + ang_y * (size / 2.0 - 4.0);
    d.rect(ax, ay, 3.0, 3.0, "#ffe040");
}
