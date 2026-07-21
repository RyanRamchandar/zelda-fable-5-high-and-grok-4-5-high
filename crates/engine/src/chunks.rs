//! Offscreen chunk cache for tile layers (ARCHITECTURE §3).

use std::collections::HashMap;

use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

pub const CHUNK_TILES: u32 = 16;
pub const CHUNK_PX: u32 = CHUNK_TILES * 16; // 256

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ChunkKey {
    pub layer: u8, // 0 = ground+detail, 1 = overhang
    pub cx: u32,
    pub cy: u32,
}

struct ChunkEntry {
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    last_frame: u64,
}

pub struct ChunkCache {
    entries: HashMap<ChunkKey, ChunkEntry>,
    max_chunks: usize,
    pub bakes_this_frame: u32,
    frame: u64,
}

impl ChunkCache {
    pub fn new(max_chunks: usize) -> Result<Self, String> {
        // Probe that offscreen canvases can be created.
        let _ = make_chunk_canvas()?;
        Ok(Self {
            entries: HashMap::new(),
            max_chunks,
            bakes_this_frame: 0,
            frame: 0,
        })
    }

    pub fn ready(&self, key: ChunkKey) -> bool {
        self.entries.contains_key(&key)
    }

    pub fn invalidate(&mut self, key: ChunkKey) {
        self.entries.remove(&key);
    }

    pub fn begin_frame(&mut self, frame: u64) {
        self.frame = frame;
        self.bakes_this_frame = 0;
    }

    pub fn budget(&self) -> usize {
        self.max_chunks
    }

    pub fn ready_count(&self) -> usize {
        self.entries.len()
    }

    pub(crate) fn ensure_slot(&mut self, key: ChunkKey) -> Result<bool, String> {
        if self.entries.contains_key(&key) {
            if let Some(e) = self.entries.get_mut(&key) {
                e.last_frame = self.frame;
            }
            return Ok(true);
        }
        if self.entries.len() >= self.max_chunks {
            // Evict LRU not used this frame.
            let victim = self
                .entries
                .iter()
                .filter(|(_, e)| e.last_frame < self.frame)
                .min_by_key(|(_, e)| e.last_frame)
                .map(|(k, _)| *k);
            if let Some(k) = victim {
                self.entries.remove(&k);
            } else {
                return Ok(false);
            }
        }
        let (canvas, ctx) = make_chunk_canvas()?;
        self.entries.insert(
            key,
            ChunkEntry {
                canvas,
                ctx,
                last_frame: self.frame,
            },
        );
        self.bakes_this_frame = self.bakes_this_frame.saturating_add(1);
        Ok(true)
    }

    pub(crate) fn ctx_mut(&mut self, key: ChunkKey) -> Option<&mut CanvasRenderingContext2d> {
        self.entries.get_mut(&key).map(|e| {
            e.last_frame = self.frame;
            &mut e.ctx
        })
    }

    pub(crate) fn canvas(&self, key: ChunkKey) -> Option<&HtmlCanvasElement> {
        self.entries.get(&key).map(|e| &e.canvas)
    }

}

fn make_chunk_canvas() -> Result<(HtmlCanvasElement, CanvasRenderingContext2d), String> {
    let doc = web_sys::window()
        .ok_or("no window")?
        .document()
        .ok_or("no document")?;
    let canvas = doc
        .create_element("canvas")
        .map_err(|_| "create canvas")?
        .dyn_into::<HtmlCanvasElement>()
        .map_err(|_| "not canvas")?;
    canvas.set_width(CHUNK_PX);
    canvas.set_height(CHUNK_PX);
    // Keep detached — chunk blits from canvas element work without DOM attach.
    let ctx = canvas
        .get_context("2d")
        .map_err(|_| "getContext")?
        .ok_or("no 2d")?
        .dyn_into::<CanvasRenderingContext2d>()
        .map_err(|_| "not 2d")?;
    ctx.set_image_smoothing_enabled(false);
    Ok((canvas, ctx))
}
