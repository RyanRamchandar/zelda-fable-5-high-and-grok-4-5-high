use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

pub struct Draw {
    ctx: CanvasRenderingContext2d,
}

impl Draw {
    pub fn new(canvas: &HtmlCanvasElement) -> Result<Self, String> {
        let ctx = canvas
            .get_context("2d")
            .map_err(|_| "getContext")?
            .ok_or("no 2d context")?
            .dyn_into::<CanvasRenderingContext2d>()
            .map_err(|_| "not CanvasRenderingContext2d")?;
        Ok(Self { ctx })
    }

    pub fn clear(&self, color: &str) {
        self.ctx.set_fill_style_str(color);
        self.ctx.fill_rect(0.0, 0.0, super::canvas::WIDTH, super::canvas::HEIGHT);
    }

    pub fn rect(&self, x: f32, y: f32, w: f32, h: f32, color: &str) {
        self.ctx.set_fill_style_str(color);
        self.ctx
            .fill_rect(f64::from(x), f64::from(y), f64::from(w), f64::from(h));
    }

    pub fn circle(&self, x: f32, y: f32, radius: f32, color: &str) {
        self.ctx.set_fill_style_str(color);
        self.ctx.begin_path();
        let _ = self.ctx.arc(
            f64::from(x),
            f64::from(y),
            f64::from(radius),
            0.0,
            std::f64::consts::TAU,
        );
        self.ctx.fill();
    }

    pub fn text(&self, s: &str, x: f32, y: f32, color: &str) {
        self.ctx.set_fill_style_str(color);
        self.ctx.set_font("10px monospace");
        let _ = self.ctx.fill_text(s, f64::from(x), f64::from(y));
    }
}
