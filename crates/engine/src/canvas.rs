use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, Window};

pub const WIDTH: f64 = 480.0;
pub const HEIGHT: f64 = 270.0;

pub struct Canvas {
    element: HtmlCanvasElement,
    scale: i32,
}

impl Canvas {
    pub fn new(window: &Window) -> Result<Self, String> {
        let document = window.document().ok_or("no document")?;
        let element = document
            .get_element_by_id("game")
            .ok_or("missing #game canvas")?
            .dyn_into::<HtmlCanvasElement>()
            .map_err(|_| "#game is not a canvas")?;

        element.set_width(WIDTH as u32);
        element.set_height(HEIGHT as u32);

        let mut canvas = Self { element, scale: 1 };
        canvas.resize(window)?;
        Ok(canvas)
    }

    pub fn element(&self) -> &HtmlCanvasElement {
        &self.element
    }

    pub fn scale(&self) -> i32 {
        self.scale
    }

    pub fn resize(&mut self, window: &Window) -> Result<(), String> {
        let inner_w = window
            .inner_width()
            .map_err(|_| "inner_width")?
            .as_f64()
            .ok_or("inner_width not f64")?;
        let inner_h = window
            .inner_height()
            .map_err(|_| "inner_height")?
            .as_f64()
            .ok_or("inner_height not f64")?;

        let scale_x = (inner_w / WIDTH).floor() as i32;
        let scale_y = (inner_h / HEIGHT).floor() as i32;
        self.scale = scale_x.min(scale_y).max(1);

        let css_w = WIDTH * f64::from(self.scale);
        let css_h = HEIGHT * f64::from(self.scale);
        let style = self.element.style();
        style
            .set_property("width", &format!("{css_w}px"))
            .map_err(|_| "set width")?;
        style
            .set_property("height", &format!("{css_h}px"))
            .map_err(|_| "set height")?;
        Ok(())
    }

    /// Map client (CSS pixel) coordinates to logical 480×270 space.
    pub fn client_to_logical(&self, client_x: f64, client_y: f64) -> (f32, f32) {
        let rect = self.element.get_bounding_client_rect();
        let scale = f64::from(self.scale).max(1.0);
        let x = ((client_x - rect.left()) / scale) as f32;
        let y = ((client_y - rect.top()) / scale) as f32;
        (x, y)
    }
}
