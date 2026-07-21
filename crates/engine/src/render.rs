use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

pub struct Draw {
    ctx: CanvasRenderingContext2d,
    ox: f32,
    oy: f32,
}

impl Draw {
    pub fn new(canvas: &HtmlCanvasElement) -> Result<Self, String> {
        let ctx = canvas
            .get_context("2d")
            .map_err(|_| "getContext")?
            .ok_or("no 2d context")?
            .dyn_into::<CanvasRenderingContext2d>()
            .map_err(|_| "not CanvasRenderingContext2d")?;
        Ok(Self {
            ctx,
            ox: 0.0,
            oy: 0.0,
        })
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
}
