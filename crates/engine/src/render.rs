use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use crate::atlas::{Atlas, SpriteHandle};

pub struct Draw {
    ctx: CanvasRenderingContext2d,
    ox: f32,
    oy: f32,
    atlas: Option<Atlas>,
    smoothing_off: bool,
}

impl Draw {
    pub fn new(canvas: &HtmlCanvasElement) -> Result<Self, String> {
        let ctx = canvas
            .get_context("2d")
            .map_err(|_| "getContext")?
            .ok_or("no 2d context")?
            .dyn_into::<CanvasRenderingContext2d>()
            .map_err(|_| "not CanvasRenderingContext2d")?;
        ctx.set_image_smoothing_enabled(false);
        Ok(Self {
            ctx,
            ox: 0.0,
            oy: 0.0,
            atlas: None,
            smoothing_off: true,
        })
    }

    pub fn set_atlas(&mut self, atlas: Atlas) {
        self.atlas = Some(atlas);
        if !self.smoothing_off {
            self.ctx.set_image_smoothing_enabled(false);
            self.smoothing_off = true;
        }
    }

    /// Camera / UI offset applied inside draw primitives. World: `(-cam.x, -cam.y)`; HUD: `(0,0)`.
    pub fn set_offset(&mut self, dx: f32, dy: f32) {
        self.ox = dx;
        self.oy = dy;
    }

    pub fn clear(&mut self, color: &str) {
        self.ox = 0.0;
        self.oy = 0.0;
        self.ctx.set_fill_style_str(color);
        self.ctx
            .fill_rect(0.0, 0.0, super::canvas::WIDTH, super::canvas::HEIGHT);
    }

    pub fn rect(&self, x: f32, y: f32, w: f32, h: f32, color: &str) {
        self.ctx.set_fill_style_str(color);
        self.ctx.fill_rect(
            f64::from(x + self.ox),
            f64::from(y + self.oy),
            f64::from(w),
            f64::from(h),
        );
    }

    pub fn circle(&self, x: f32, y: f32, radius: f32, color: &str) {
        self.ctx.set_fill_style_str(color);
        self.ctx.begin_path();
        let _ = self.ctx.arc(
            f64::from(x + self.ox),
            f64::from(y + self.oy),
            f64::from(radius),
            0.0,
            std::f64::consts::TAU,
        );
        self.ctx.fill();
    }

    pub fn line(&self, x1: f32, y1: f32, x2: f32, y2: f32, w: f32, color: &str) {
        self.ctx.set_stroke_style_str(color);
        self.ctx.set_line_width(f64::from(w));
        self.ctx.begin_path();
        self.ctx
            .move_to(f64::from(x1 + self.ox), f64::from(y1 + self.oy));
        self.ctx
            .line_to(f64::from(x2 + self.ox), f64::from(y2 + self.oy));
        self.ctx.stroke();
    }

    pub fn text(&self, s: &str, x: f32, y: f32, color: &str) {
        self.ctx.set_fill_style_str(color);
        self.ctx.set_font("10px monospace");
        let _ = self
            .ctx
            .fill_text(s, f64::from(x + self.ox), f64::from(y + self.oy));
    }

    pub fn sprite(&self, h: SpriteHandle, frame: u32, x: f32, y: f32, flip_x: bool) {
        let Some(atlas) = self.atlas.as_ref() else {
            return;
        };
        let frame = frame.min(h.frames.saturating_sub(1));
        let sx = f64::from(h.x + frame * h.frame_w);
        let sy = f64::from(h.y);
        let sw = f64::from(h.frame_w);
        let sh = f64::from(h.frame_h);
        let dx = f64::from(x + self.ox);
        let dy = f64::from(y + self.oy);

        if flip_x {
            self.ctx.save();
            let _ = self.ctx.translate(dx + sw, dy);
            let _ = self.ctx.scale(-1.0, 1.0);
            let _ = self
                .ctx
                .draw_image_with_html_canvas_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                    atlas.canvas(),
                    sx,
                    sy,
                    sw,
                    sh,
                    0.0,
                    0.0,
                    sw,
                    sh,
                );
            self.ctx.restore();
        } else {
            let _ = self
                .ctx
                .draw_image_with_html_canvas_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                    atlas.canvas(),
                    sx,
                    sy,
                    sw,
                    sh,
                    dx,
                    dy,
                    sw,
                    sh,
                );
        }
    }

    /// Draw sprite scaled (debug viewer 3×).
    pub fn sprite_scaled(
        &self,
        h: SpriteHandle,
        frame: u32,
        x: f32,
        y: f32,
        scale: f32,
        flip_x: bool,
    ) {
        let Some(atlas) = self.atlas.as_ref() else {
            return;
        };
        let frame = frame.min(h.frames.saturating_sub(1));
        let sx = f64::from(h.x + frame * h.frame_w);
        let sy = f64::from(h.y);
        let sw = f64::from(h.frame_w);
        let sh = f64::from(h.frame_h);
        let dw = sw * f64::from(scale);
        let dh = sh * f64::from(scale);
        let dx = f64::from(x + self.ox);
        let dy = f64::from(y + self.oy);

        if flip_x {
            self.ctx.save();
            let _ = self.ctx.translate(dx + dw, dy);
            let _ = self.ctx.scale(-1.0, 1.0);
            let _ = self
                .ctx
                .draw_image_with_html_canvas_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                    atlas.canvas(),
                    sx,
                    sy,
                    sw,
                    sh,
                    0.0,
                    0.0,
                    dw,
                    dh,
                );
            self.ctx.restore();
        } else {
            let _ = self
                .ctx
                .draw_image_with_html_canvas_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh(
                    atlas.canvas(),
                    sx,
                    sy,
                    sw,
                    sh,
                    dx,
                    dy,
                    dw,
                    dh,
                );
        }
    }
}
