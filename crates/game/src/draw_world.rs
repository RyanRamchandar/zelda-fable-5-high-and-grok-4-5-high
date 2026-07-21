//! Data-driven map rendering with optional chunk cache.

use content::maps::{catalog, TILE_PX};
use engine::atlas::SpriteHandle;
use engine::chunks::{ChunkCache, ChunkKey, CHUNK_TILES};
use engine::render::Draw;

use crate::assets::SpriteMap;
use crate::math::Dir4;
use crate::world::entity::{Entity, EntityData, EntityKind, PlayerState};
use crate::world::World;

pub struct TileSprites {
    pub by_id: Vec<Option<SpriteHandle>>,
}

impl TileSprites {
    pub fn build(sprites: &SpriteMap) -> Self {
        let max = catalog::all_tile_ids()
            .iter()
            .copied()
            .max()
            .unwrap_or(0) as usize
            + 1;
        let mut by_id = vec![None; max];
        for &id in catalog::all_tile_ids() {
            let info = catalog::tile_info(id);
            if info.sprite.is_empty() {
                continue;
            }
            by_id[id as usize] = sprites.get(info.sprite);
        }
        Self { by_id }
    }

    fn get(&self, id: u16) -> Option<SpriteHandle> {
        self.by_id.get(id as usize).copied().flatten()
    }
}

pub struct MapRenderStats {
    pub chunks_ready: usize,
    pub chunks_budget: usize,
    pub bakes: u32,
    pub direct: bool,
}

/// Bake + blit visible chunks; falls back to per-tile when cache is None.
pub fn render_map(
    d: &mut Draw,
    world: &mut World,
    tiles: &TileSprites,
    cache: &mut Option<ChunkCache>,
    prebake: bool,
    stats: &mut MapRenderStats,
) {
    // Drain dirty.
    if let Some(c) = cache.as_mut() {
        for key in world.dirty_chunks.drain(..) {
            c.invalidate(key);
        }
        c.begin_frame(world.tick);
    } else {
        world.dirty_chunks.clear();
    }

    let cam = world.camera.offset();
    let (cx0, cy0, cx1, cy1) = visible_chunk_range(&world.map, cam);

    if let Some(c) = cache.as_mut() {
        stats.direct = false;
        stats.chunks_budget = c.budget();
        let bake_cap = if prebake { 32 } else { 2 };
        let mut baked = 0u32;
        for layer in [0u8, 1u8] {
            for cy in cy0..=cy1 {
                for cx in cx0..=cx1 {
                    let key = ChunkKey { layer, cx, cy };
                    if c.ready(key) {
                        continue;
                    }
                    if baked >= bake_cap {
                        continue;
                    }
                    if bake_chunk(d, world, tiles, c, key) {
                        baked += 1;
                    }
                }
            }
        }
        // Blit ground+detail
        for cy in cy0..=cy1 {
            for cx in cx0..=cx1 {
                let key = ChunkKey {
                    layer: 0,
                    cx,
                    cy,
                };
                if c.ready(key) {
                    let x = cx as f32 * CHUNK_TILES as f32 * TILE_PX;
                    let y = cy as f32 * CHUNK_TILES as f32 * TILE_PX;
                    d.chunk_blit(c, key, x, y);
                } else {
                    draw_chunk_direct(d, world, tiles, 0, cx, cy);
                }
            }
        }
        draw_animated(d, world, tiles, cam);
        stats.chunks_ready = c.ready_count();
        stats.bakes = c.bakes_this_frame;
    } else {
        stats.direct = true;
        draw_visible_direct(d, world, tiles, cam, false);
        draw_animated(d, world, tiles, cam);
        stats.chunks_ready = 0;
        stats.chunks_budget = 0;
        stats.bakes = 0;
    }
}

pub fn render_overhang(
    d: &mut Draw,
    world: &World,
    tiles: &TileSprites,
    cache: &Option<ChunkCache>,
) {
    let cam = world.camera.offset();
    let (cx0, cy0, cx1, cy1) = visible_chunk_range(&world.map, cam);
    if let Some(c) = cache.as_ref() {
        for cy in cy0..=cy1 {
            for cx in cx0..=cx1 {
                let key = ChunkKey {
                    layer: 1,
                    cx,
                    cy,
                };
                if c.ready(key) {
                    let x = cx as f32 * CHUNK_TILES as f32 * TILE_PX;
                    let y = cy as f32 * CHUNK_TILES as f32 * TILE_PX;
                    d.chunk_blit(c, key, x, y);
                } else {
                    draw_chunk_direct(d, world, tiles, 1, cx, cy);
                }
            }
        }
    } else {
        draw_visible_direct(d, world, tiles, cam, true);
    }
}

fn bake_chunk(
    d: &mut Draw,
    world: &World,
    tiles: &TileSprites,
    cache: &mut ChunkCache,
    key: ChunkKey,
) -> bool {
    if !d.chunk_bake_begin(cache, key) {
        return false;
    }
    let origin_x = key.cx * CHUNK_TILES;
    let origin_y = key.cy * CHUNK_TILES;
    for ty in 0..CHUNK_TILES {
        for tx in 0..CHUNK_TILES {
            let x = origin_x + tx;
            let y = origin_y + ty;
            if x >= world.map.width || y >= world.map.height {
                continue;
            }
            let lx = tx as f32 * TILE_PX;
            let ly = ty as f32 * TILE_PX;
            let i = world.map.idx(x, y);
            if key.layer == 0 {
                blit_tile(d, tiles, world.map.ground[i], lx, ly, 0);
                blit_tile(d, tiles, world.map.detail[i], lx, ly, 0);
            } else {
                blit_tile(d, tiles, world.map.overhang[i], lx, ly, 0);
            }
        }
    }
    d.chunk_bake_end(cache);
    true
}

fn draw_chunk_direct(
    d: &mut Draw,
    world: &World,
    tiles: &TileSprites,
    layer: u8,
    cx: u32,
    cy: u32,
) {
    let origin_x = cx * CHUNK_TILES;
    let origin_y = cy * CHUNK_TILES;
    for ty in 0..CHUNK_TILES {
        for tx in 0..CHUNK_TILES {
            let x = origin_x + tx;
            let y = origin_y + ty;
            if x >= world.map.width || y >= world.map.height {
                continue;
            }
            let px = x as f32 * TILE_PX;
            let py = y as f32 * TILE_PX;
            let i = world.map.idx(x, y);
            if layer == 0 {
                blit_tile(d, tiles, world.map.ground[i], px, py, 0);
                blit_tile(d, tiles, world.map.detail[i], px, py, 0);
            } else {
                blit_tile(d, tiles, world.map.overhang[i], px, py, 0);
            }
        }
    }
}

fn draw_visible_direct(
    d: &mut Draw,
    world: &World,
    tiles: &TileSprites,
    cam: crate::math::Vec2,
    overhang_only: bool,
) {
    let x0 = ((cam.x / TILE_PX).floor() as i32).max(0) as u32;
    let y0 = ((cam.y / TILE_PX).floor() as i32).max(0) as u32;
    let x1 = ((cam.x + 480.0) / TILE_PX).ceil() as u32 + 1;
    let y1 = ((cam.y + 270.0) / TILE_PX).ceil() as u32 + 1;
    let x1 = x1.min(world.map.width);
    let y1 = y1.min(world.map.height);
    for ty in y0..y1 {
        for tx in x0..x1 {
            let i = world.map.idx(tx, ty);
            let px = tx as f32 * TILE_PX;
            let py = ty as f32 * TILE_PX;
            if overhang_only {
                blit_tile(d, tiles, world.map.overhang[i], px, py, 0);
            } else {
                blit_tile(d, tiles, world.map.ground[i], px, py, 0);
                blit_tile(d, tiles, world.map.detail[i], px, py, 0);
            }
        }
    }
}

fn draw_animated(d: &mut Draw, world: &World, tiles: &TileSprites, cam: crate::math::Vec2) {
    let mut count = 0u32;
    let coarsen = world.animated_tiles.len() > 180;
    for (i, &(tx, ty, id)) in world.animated_tiles.iter().enumerate() {
        if coarsen && i % 2 == 1 {
            continue;
        }
        let px = tx as f32 * TILE_PX;
        let py = ty as f32 * TILE_PX;
        if px + TILE_PX < cam.x || py + TILE_PX < cam.y || px > cam.x + 480.0 || py > cam.y + 270.0
        {
            continue;
        }
        let info = catalog::tile_info(id);
        let frame = if info.frames > 1 && info.anim_rate > 0 {
            ((world.tick / u64::from(info.anim_rate)) % u64::from(info.frames)) as u32
        } else {
            0
        };
        blit_tile(d, tiles, id, px, py, frame);
        count += 1;
        if count > 200 {
            break;
        }
    }
}

fn blit_tile(d: &mut Draw, tiles: &TileSprites, id: u16, x: f32, y: f32, frame: u32) {
    if id == 0 {
        return;
    }
    if let Some(h) = tiles.get(id) {
        d.sprite(h, frame, x, y, false);
    }
}

fn visible_chunk_range(map: &content::maps::MapDef, cam: crate::math::Vec2) -> (u32, u32, u32, u32) {
    let tx0 = ((cam.x / TILE_PX).floor() as i32).max(0) as u32;
    let ty0 = ((cam.y / TILE_PX).floor() as i32).max(0) as u32;
    let tx1 = ((cam.x + 480.0) / TILE_PX).ceil() as u32 + 1;
    let ty1 = ((cam.y + 270.0) / TILE_PX).ceil() as u32 + 1;
    let tx1 = tx1.min(map.width.saturating_sub(1));
    let ty1 = ty1.min(map.height.saturating_sub(1));
    let cx0 = tx0 / CHUNK_TILES;
    let cy0 = ty0 / CHUNK_TILES;
    let cx1 = tx1 / CHUNK_TILES;
    let cy1 = ty1 / CHUNK_TILES;
    (cx0, cy0, cx1, cy1)
}

pub fn render_entity(d: &mut Draw, e: &Entity, sprites: &SpriteMap) {
    match e.kind {
        EntityKind::Player => render_player(d, e, sprites),
        EntityKind::Dummy => {
            if let EntityData::Dummy(dd) = &e.data {
                if dd.dead_ticks.is_some() {
                    return;
                }
            }
            let flash = e.health.map(|h| h.flash > 0).unwrap_or(false);
            if flash && e.health.unwrap().flash.is_multiple_of(2) {
                return;
            }
            if let Some(h) = sprites.get("slime_dummy") {
                let frame = ((e.anim.timer / 16) % 2) as u32;
                d.sprite(h, frame, e.pos.x, e.pos.y, false);
            }
        }
        EntityKind::Slime => {
            if e.body.is_none() {
                d.circle(e.pos.x + 8.0, e.pos.y + 8.0, 6.0, "rgba(120,220,140,0.35)");
                return;
            }
            let flash = e.health.map(|h| h.flash > 0).unwrap_or(false);
            if flash && e.health.unwrap().flash.is_multiple_of(2) {
                return;
            }
            let angry = e.health.map(|h| h.hp <= 1).unwrap_or(false);
            let key = if angry { "slime_angry" } else { "slime" };
            if let Some(h) = sprites.get(key) {
                d.sprite(h, e.anim.frame as u32, e.pos.x, e.pos.y, false);
            }
        }
        EntityKind::Bat => {
            if e.body.is_none() {
                d.circle(e.pos.x + 8.0, e.pos.y + 8.0, 5.0, "rgba(160,120,200,0.35)");
                return;
            }
            let flash = e.health.map(|h| h.flash > 0).unwrap_or(false);
            if flash && e.health.unwrap().flash.is_multiple_of(2) {
                return;
            }
            if let Some(h) = sprites.get("bat") {
                d.sprite(h, e.anim.frame as u32, e.pos.x, e.pos.y, e.facing == Dir4::Left);
            }
        }
        EntityKind::Octorok => {
            if e.body.is_none() {
                d.circle(e.pos.x + 8.0, e.pos.y + 8.0, 6.0, "rgba(220,120,80,0.35)");
                return;
            }
            let flash = e.health.map(|h| h.flash > 0).unwrap_or(false);
            if flash && e.health.unwrap().flash.is_multiple_of(2) {
                return;
            }
            if let Some(h) = sprites.get("octorok") {
                d.sprite(
                    h,
                    e.anim.frame as u32,
                    e.pos.x,
                    e.pos.y,
                    e.facing == Dir4::Left,
                );
            }
        }
        EntityKind::OctorokRock => {
            let key = match &e.data {
                EntityData::Rock(r) if r.from_player => "octorok_rock_warm",
                _ => "octorok_rock",
            };
            if let Some(h) = sprites.get(key) {
                d.sprite(h, e.anim.frame as u32, e.pos.x, e.pos.y, false);
            }
        }
        EntityKind::Pickup => {
            if let EntityData::Pickup(pd) = &e.data {
                if pd.life < crate::combat::tuning::PICKUP_BLINK && (pd.life / 4) % 2 == 0 {
                    return;
                }
                match pd.kind {
                    crate::world::entity::PickupKind::Rupee => {
                        d.rect(e.pos.x, e.pos.y, 6.0, 8.0, "#40e080");
                    }
                    crate::world::entity::PickupKind::Heart => {
                        if let Some(h) = sprites.get("heart_full") {
                            d.sprite(h, 0, e.pos.x, e.pos.y, false);
                        }
                    }
                    crate::world::entity::PickupKind::Energy => {
                        d.circle(e.pos.x + 3.0, e.pos.y + 3.0, 3.0, "#40e0ff");
                    }
                }
            }
        }
        EntityKind::SwordBeam => {
            d.rect(e.pos.x, e.pos.y, 6.0, 6.0, "#c0e0ff");
        }
        EntityKind::DebugShot => {
            d.rect(e.pos.x, e.pos.y, 6.0, 6.0, "#ff6060");
        }
        EntityKind::FairyFountain => {}
        EntityKind::Sign => {
            if let Some(h) = sprites.get("prop_sign") {
                d.sprite(h, 0, e.pos.x, e.pos.y, false);
            } else {
                d.rect(e.pos.x, e.pos.y, 16.0, 16.0, "#c0a060");
            }
        }
        EntityKind::Npc => {
            let key = match &e.data {
                EntityData::Npc(n) => content::text::npc_sprite(n.npc),
                _ => "npc_villager_a",
            };
            if let Some(h) = sprites.get(key) {
                d.sprite(
                    h,
                    e.anim.frame as u32,
                    e.pos.x,
                    e.pos.y - 8.0,
                    e.facing == Dir4::Left,
                );
            } else {
                d.rect(e.pos.x, e.pos.y - 8.0, 16.0, 24.0, "#80c080");
            }
        }
        EntityKind::Chest => {
            let frame = match &e.data {
                EntityData::Chest(c) if c.open => 1,
                _ => 0,
            };
            if let Some(h) = sprites.get("prop_chest") {
                d.sprite(h, frame, e.pos.x, e.pos.y, false);
            } else {
                d.rect(e.pos.x, e.pos.y, 16.0, 16.0, "#b08040");
            }
        }
        EntityKind::Gem => {
            let taken = matches!(&e.data, EntityData::Gem(g) if g.taken);
            if taken {
                if let Some(h) = sprites.get("prop_pedestal") {
                    d.sprite(h, 0, e.pos.x, e.pos.y, false);
                }
            } else if let Some(h) = sprites.get("prop_gem") {
                let frame = ((e.anim.timer / 16) % 2) as u32;
                d.sprite(h, frame, e.pos.x, e.pos.y, false);
            } else {
                d.rect(e.pos.x + 4.0, e.pos.y + 4.0, 8.0, 8.0, "#40e0ff");
            }
        }
    }
}

fn render_player(d: &mut Draw, e: &Entity, sprites: &SpriteMap) {
    let iframes = e.health.map(|h| h.iframes > 0).unwrap_or(false);
    if iframes && (e.health.unwrap().iframes / 2).is_multiple_of(2) {
        return;
    }
    let flash = e.health.map(|h| h.flash > 0).unwrap_or(false);
    let EntityData::Player(pd) = &e.data else {
        return;
    };

    let (key, frame, flip) = player_sprite(e.facing, pd, e);
    let _ = flash;
    if matches!(pd.state, PlayerState::Charging { tick } if tick >= 20) {
        if let Some(h) = sprites.get("player_charge_glow") {
            d.sprite(h, 0, e.pos.x, e.pos.y - 8.0, flip);
        }
    }
    if let Some(h) = sprites.get(key) {
        d.sprite(h, frame, e.pos.x, e.pos.y - 8.0, flip);
    }
}

fn player_sprite(
    facing: Dir4,
    pd: &crate::world::entity::PlayerData,
    e: &Entity,
) -> (&'static str, u32, bool) {
    let flip = facing == Dir4::Left;
    let dir_slot = match facing {
        Dir4::Down => 0,
        Dir4::Up => 1,
        Dir4::Right | Dir4::Left => 2,
    };

    if e.health.map(|h| h.flash > 0).unwrap_or(false) && pd.state == PlayerState::Idle {
        return ("player_hurt", 0, flip);
    }

    match pd.state {
        PlayerState::Swing { stage, tick } => {
            let frames = 3u32;
            let f = ((tick as u32 * frames) / crate::combat::tuning::SLASH_TICKS as u32).min(2);
            let key = match (stage, dir_slot) {
                (0, 0) => "player_slash_d",
                (0, 1) => "player_slash_u",
                (0, _) => "player_slash_r",
                (1, 0) => "player_bslash_d",
                (1, 1) => "player_bslash_u",
                (1, _) => "player_bslash_r",
                (_, 0) => "player_lunge_d",
                (_, 1) => "player_lunge_u",
                _ => "player_lunge_r",
            };
            let f = if stage >= 2 { f.min(1) } else { f };
            (key, f, flip)
        }
        PlayerState::Spin { tick } => ("player_spin", (tick as u32 / 2) % 8, false),
        PlayerState::Dash { tick } | PlayerState::DashRecovery { tick } => {
            let key = match dir_slot {
                0 => "player_dash_d",
                1 => "player_dash_u",
                _ => "player_dash_r",
            };
            (key, (tick as u32 / 4) % 2, flip)
        }
        PlayerState::Charging { .. } => {
            let base = idle_frame(dir_slot, pd.walk_timer);
            ("player_idle", base, flip)
        }
        PlayerState::LedgeHop { .. } => {
            let key = match dir_slot {
                0 => "player_dash_d",
                1 => "player_dash_u",
                _ => "player_dash_r",
            };
            (key, 0, flip)
        }
        PlayerState::Idle => {
            if pd.shield_held {
                ("player_shield", dir_slot.min(2) as u32, flip)
            } else if pd.move_blend > 0.2 || e.vel.len() > 0.2 {
                let key = match dir_slot {
                    0 => "player_walk_d",
                    1 => "player_walk_u",
                    _ => "player_walk_r",
                };
                (key, ((pd.walk_timer / 8) % 4) as u32, flip)
            } else {
                ("player_idle", idle_frame(dir_slot, e.anim.timer), flip)
            }
        }
    }
}

fn idle_frame(dir_slot: i32, timer: u16) -> u32 {
    let breath = ((timer / 24) % 2) as u32;
    match dir_slot {
        0 => breath,
        1 => 2 + breath,
        _ => 4 + breath,
    }
}
