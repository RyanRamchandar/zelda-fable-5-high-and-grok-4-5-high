//! Map definitions v2: layered tiles, flags, spawns, triggers, regions.

pub mod arena;
pub mod catalog;
pub mod dungeon;
pub mod dungeon_rooms;
pub mod interiors;
pub mod overworld;
pub mod paint;

pub use arena::{arena, FLOOR, FOUNTAIN, WALL};
pub use catalog::{tile_info, TileInfo, T_VOID};

pub const TILE_PX: f32 = 16.0;

pub mod flags {
    pub const SOLID: u8 = 1 << 0;
    pub const WATER: u8 = 1 << 1;
    pub const LEDGE_N: u8 = 1 << 2;
    pub const LEDGE_S: u8 = 1 << 3;
    pub const LEDGE_E: u8 = 1 << 4;
    pub const LEDGE_W: u8 = 1 << 5;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TileLayer {
    Ground,
    Detail,
    Overhang,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapId {
    Overworld,
    Arena,
    House(u8),
    Shop,
    Cave(u8),
    ShrineLobby,
    Dungeon,
}

impl MapId {
    pub fn to_u8(self) -> u8 {
        match self {
            MapId::Overworld => 0,
            MapId::Arena => 1,
            MapId::Shop => 2,
            MapId::ShrineLobby => 3,
            MapId::Dungeon => 4,
            MapId::House(n) => 10 + n.min(5),
            MapId::Cave(n) => 20 + n.min(2),
        }
    }

    pub fn from_u8(v: u8) -> Self {
        match v {
            0 => MapId::Overworld,
            1 => MapId::Arena,
            2 => MapId::Shop,
            3 => MapId::ShrineLobby,
            4 => MapId::Dungeon,
            10..=15 => MapId::House(v - 10),
            20..=22 => MapId::Cave(v - 20),
            _ => MapId::Overworld,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Loot {
    Rupees(u32),
    HeartPiece,
    Gem(u8),
    Boomerang,
    SmallKey,
    BossKey,
}

#[derive(Clone, Copy, Debug)]
pub struct EntryPoint {
    pub id: u8,
    pub tx: u32,
    pub ty: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpawnKind {
    Slime,
    Bat,
    Octorok,
    RaiderSpear,
    RaiderTorch,
    Wisp,
    Skeleton,
    FairyFountain,
    Dummy,
    Sign { text: crate::text::TextId },
    Npc { npc: crate::text::NpcId },
    Chest { flag: u16, loot: Loot },
    Gem { id: u8 },
}

#[derive(Clone, Copy, Debug)]
pub struct SpawnDef {
    pub tx: u32,
    pub ty: u32,
    pub kind: SpawnKind,
    pub group: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TriggerKind {
    Door { target: MapId, entry: u8 },
    Banner { region: u8 },
    Checkpoint { id: u8 },
    Secret { flag: u16 },
}

#[derive(Clone, Copy, Debug)]
pub struct TriggerDef {
    pub tx: u32,
    pub ty: u32,
    pub w: u32,
    pub h: u32,
    pub kind: TriggerKind,
}

#[derive(Clone, Copy, Debug)]
pub struct RegionDef {
    pub name: &'static str,
    pub rect: (u32, u32, u32, u32),
}

#[derive(Clone, Debug)]
pub struct MapDef {
    pub width: u32,
    pub height: u32,
    pub ground: Vec<u16>,
    pub detail: Vec<u16>,
    pub overhang: Vec<u16>,
    pub collision: Vec<u8>,
    pub spawns: Vec<SpawnDef>,
    pub triggers: Vec<TriggerDef>,
    pub regions: Vec<RegionDef>,
    pub entries: Vec<EntryPoint>,
}

impl MapDef {
    pub fn new(width: u32, height: u32, fill_tile: u16) -> Self {
        let n = (width * height) as usize;
        let flags = catalog::tile_info(fill_tile).flags;
        Self {
            width,
            height,
            ground: vec![fill_tile; n],
            detail: vec![T_VOID; n],
            overhang: vec![T_VOID; n],
            collision: vec![flags; n],
            spawns: Vec::new(),
            triggers: Vec::new(),
            regions: Vec::new(),
            entries: Vec::new(),
        }
    }

    pub fn idx(&self, tx: u32, ty: u32) -> usize {
        (ty * self.width + tx) as usize
    }

    pub fn in_bounds(&self, tx: i32, ty: i32) -> bool {
        tx >= 0 && ty >= 0 && (tx as u32) < self.width && (ty as u32) < self.height
    }

    pub fn flags_at(&self, tx: i32, ty: i32) -> u8 {
        if !self.in_bounds(tx, ty) {
            return flags::SOLID;
        }
        self.collision[self.idx(tx as u32, ty as u32)]
    }

    pub fn solid_at(&self, tx: i32, ty: i32) -> bool {
        self.flags_at(tx, ty) & flags::SOLID != 0
    }

    pub fn blocks_walker(&self, tx: i32, ty: i32) -> bool {
        let f = self.flags_at(tx, ty);
        f & (flags::SOLID | flags::WATER) != 0
    }

    pub fn fill(&mut self, tile: u16) {
        let f = catalog::tile_info(tile).flags;
        for t in &mut self.ground {
            *t = tile;
        }
        for c in &mut self.collision {
            *c = f;
        }
    }

    pub fn set_flags(&mut self, x: u32, y: u32, f: u8) {
        if x >= self.width || y >= self.height {
            return;
        }
        let i = self.idx(x, y);
        self.collision[i] = f;
    }

    pub fn set(&mut self, x: u32, y: u32, layer: TileLayer, tile: u16) {
        self.set_ex(x, y, layer, tile, None);
    }

    pub fn get(&self, x: u32, y: u32, layer: TileLayer) -> u16 {
        if x >= self.width || y >= self.height {
            return 0;
        }
        let i = self.idx(x, y);
        match layer {
            TileLayer::Ground => self.ground[i],
            TileLayer::Detail => self.detail[i],
            TileLayer::Overhang => self.overhang[i],
        }
    }

    /// Paint a tile; collision comes from catalog unless `flags_override` is set.
    /// Ground paints also refresh collision from the tile (or override).
    pub fn set_ex(
        &mut self,
        x: u32,
        y: u32,
        layer: TileLayer,
        tile: u16,
        flags_override: Option<u8>,
    ) {
        if x >= self.width || y >= self.height {
            return;
        }
        let i = self.idx(x, y);
        match layer {
            TileLayer::Ground => {
                self.ground[i] = tile;
                self.collision[i] =
                    flags_override.unwrap_or_else(|| catalog::tile_info(tile).flags);
            }
            TileLayer::Detail => self.detail[i] = tile,
            TileLayer::Overhang => self.overhang[i] = tile,
        }
    }

    /// Legacy helper: set ground + explicit solid bit (arena / quick borders).
    pub fn set_ground_solid(&mut self, x: u32, y: u32, tile: u16, solid: bool) {
        let mut f = catalog::tile_info(tile).flags;
        if solid {
            f |= flags::SOLID;
        } else {
            f &= !flags::SOLID;
        }
        self.set_ex(x, y, TileLayer::Ground, tile, Some(f));
    }

    pub fn rect_border(&mut self, tile: u16, solid: bool) {
        let w = self.width;
        let h = self.height;
        for x in 0..w {
            self.set_ground_solid(x, 0, tile, solid);
            self.set_ground_solid(x, h - 1, tile, solid);
        }
        for y in 0..h {
            self.set_ground_solid(0, y, tile, solid);
            self.set_ground_solid(w - 1, y, tile, solid);
        }
    }

    pub fn fill_rect(&mut self, x0: u32, y0: u32, x1: u32, y1: u32, tile: u16, solid: bool) {
        for y in y0..=y1.min(self.height.saturating_sub(1)) {
            for x in x0..=x1.min(self.width.saturating_sub(1)) {
                self.set_ground_solid(x, y, tile, solid);
            }
        }
    }

    pub fn fill_rect_layer(
        &mut self,
        x0: u32,
        y0: u32,
        x1: u32,
        y1: u32,
        layer: TileLayer,
        tile: u16,
    ) {
        for y in y0..=y1.min(self.height.saturating_sub(1)) {
            for x in x0..=x1.min(self.width.saturating_sub(1)) {
                self.set(x, y, layer, tile);
            }
        }
    }

    pub fn entry_pos(&self, id: u8) -> Option<(u32, u32)> {
        self.entries
            .iter()
            .find(|e| e.id == id)
            .map(|e| (e.tx, e.ty))
    }

    pub fn entry_world_pos(&self, id: u8) -> Option<(f32, f32)> {
        self.entry_pos(id)
            .map(|(tx, ty)| (tx as f32 * TILE_PX, ty as f32 * TILE_PX))
    }
}

/// Single loader entry point for all maps.
pub fn build(id: MapId) -> MapDef {
    match id {
        MapId::Overworld => overworld::build(),
        MapId::Arena => arena::arena(),
        MapId::House(n) => interiors::house_for(n),
        MapId::Shop => interiors::shop(),
        MapId::Cave(0) => interiors::cave_grotto(),
        MapId::Cave(1) => interiors::cave_heart(),
        MapId::Cave(_) => interiors::cave_bomb(),
        MapId::ShrineLobby => interiors::shrine_lobby(),
        MapId::Dungeon => dungeon::build(),
    }
}
