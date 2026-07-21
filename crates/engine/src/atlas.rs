//! Offscreen atlas bake: RGBA strips → shelf-packed canvas.

use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

#[derive(Clone, Copy, Debug)]
pub struct SpriteHandle {
    pub x: u32,
    pub y: u32,
    pub frame_w: u32,
    pub frame_h: u32,
    pub frames: u32,
}

pub struct Atlas {
    canvas: HtmlCanvasElement,
}

impl Atlas {
    pub fn canvas(&self) -> &HtmlCanvasElement {
        &self.canvas
    }
}

pub struct AtlasBuilder {
    canvas: HtmlCanvasElement,
    ctx: CanvasRenderingContext2d,
    width: u32,
    height: u32,
    cursor_x: u32,
    cursor_y: u32,
    row_h: u32,
}

impl AtlasBuilder {
    pub fn new(w: u32, h: u32) -> Result<Self, String> {
        let window = web_sys::window().ok_or("no window")?;
        let document = window.document().ok_or("no document")?;
        let canvas = document
            .create_element("canvas")
            .map_err(|_| "create canvas")?
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| "not canvas")?;
        canvas.set_width(w);
        canvas.set_height(h);
        // Keep atlas in the DOM (hidden). Some Chromium builds won't sample
        // pixels via drawImage from a never-attached canvas.
        if let Some(body) = document.body() {
            canvas.set_id("atlas");
            let style = canvas.style();
            style
                .set_property("display", "none")
                .map_err(|_| "atlas style")?;
            style
                .set_property("position", "absolute")
                .map_err(|_| "atlas style")?;
            body.append_child(&canvas).map_err(|_| "atlas append")?;
        }
        let ctx = canvas
            .get_context("2d")
            .map_err(|_| "getContext")?
            .ok_or("no 2d")?
            .dyn_into::<CanvasRenderingContext2d>()
            .map_err(|_| "not 2d")?;
        ctx.set_image_smoothing_enabled(false);
        Ok(Self {
            canvas,
            ctx,
            width: w,
            height: h,
            cursor_x: 0,
            cursor_y: 0,
            row_h: 0,
        })
    }

    /// `pixels`: RGBA rows for one horizontal frame strip (`frame_w * frames` × `frame_h`).
    pub fn add_strip(
        &mut self,
        frame_w: u32,
        frame_h: u32,
        frames: u32,
        pixels: &[u8],
    ) -> Result<SpriteHandle, String> {
        let strip_w = frame_w.saturating_mul(frames);
        let expected = (strip_w * frame_h * 4) as usize;
        if pixels.len() != expected {
            return Err(format!(
                "strip pixel len {} != expected {}",
                pixels.len(),
                expected
            ));
        }
        if strip_w > self.width || frame_h > self.height {
            return Err("strip larger than atlas".into());
        }

        if self.cursor_x + strip_w > self.width {
            self.cursor_x = 0;
            self.cursor_y += self.row_h;
            self.row_h = 0;
        }
        if self.cursor_y + frame_h > self.height {
            return Err("atlas full".into());
        }

        let clamped = wasm_bindgen::Clamped(pixels);
        let image = ImageData::new_with_u8_clamped_array_and_sh(clamped, strip_w, frame_h)
            .map_err(|_| "ImageData")?;
        self.ctx
            .put_image_data(&image, f64::from(self.cursor_x), f64::from(self.cursor_y))
            .map_err(|_| "putImageData")?;

        let handle = SpriteHandle {
            x: self.cursor_x,
            y: self.cursor_y,
            frame_w,
            frame_h,
            frames,
        };
        self.cursor_x += strip_w + 1; // 1px gutter
        self.row_h = self.row_h.max(frame_h + 1);
        Ok(handle)
    }

    pub fn finish(self) -> Atlas {
        Atlas {
            canvas: self.canvas,
        }
    }
}
