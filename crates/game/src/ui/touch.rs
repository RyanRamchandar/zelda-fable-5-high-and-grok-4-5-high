//! Skinned touch overlay + portrait rotate hint.

use engine::input::{
    TouchOverlay, BUTTON_ATTACK, BUTTON_CYCLE, BUTTON_DASH, BUTTON_INTERACT, BUTTON_ITEM,
    BUTTON_PAUSE, JOYSTICK_MAX_RADIUS,
};
use engine::render::Draw;

use crate::assets::SpriteMap;
use crate::world::entity::EntityData;
use crate::world::World;

fn icon_frame(button: usize) -> u32 {
    match button {
        BUTTON_ATTACK => 0,
        BUTTON_ITEM => 1,
        BUTTON_DASH => 2,
        BUTTON_INTERACT => 3,
        BUTTON_CYCLE => 4,
        BUTTON_PAUSE => 5,
        _ => 0,
    }
}

pub fn render(d: &mut Draw, overlay: &TouchOverlay, world: &World, sprites: &SpriteMap) {
    if let Some((ox, oy)) = overlay.joystick_origin {
        d.circle(ox, oy, JOYSTICK_MAX_RADIUS, "rgba(255,255,255,0.25)");
    }
    if let Some((kx, ky)) = overlay.joystick_knob {
        d.circle(kx, ky, 8.0, "rgba(255,255,255,0.45)");
    }

    for b in &overlay.buttons {
        let fill = if b.held {
            "rgba(60,90,140,0.7)"
        } else {
            "rgba(20,28,40,0.5)"
        };
        d.circle(b.cx, b.cy, b.r + 1.0, "rgba(10,14,20,0.85)");
        d.circle(b.cx, b.cy, b.r, fill);

        if b.button == BUTTON_ITEM {
            draw_item_in_disc(d, world, sprites, b.cx, b.cy);
        } else if let Some(icons) = sprites.get("touch_icons") {
            let frame = icon_frame(b.button);
            d.sprite(icons, frame, b.cx - 6.0, b.cy - 6.0, false);
        } else {
            let label = match b.button {
                BUTTON_ATTACK => "A",
                BUTTON_DASH => "D",
                BUTTON_INTERACT => "E",
                BUTTON_CYCLE => "Q",
                BUTTON_PAUSE => "=",
                _ => "?",
            };
            d.text(label, b.cx - 3.0, b.cy + 4.0, "#e8e8e8");
        }
    }
}

fn draw_item_in_disc(d: &mut Draw, world: &World, sprites: &SpriteMap, cx: f32, cy: f32) {
    let Some(p) = world.get(world.player_id) else {
        return;
    };
    let EntityData::Player(pd) = &p.data else {
        return;
    };
    match pd.selected_item {
        2 => {
            if let Some(h) = sprites.get("boomerang") {
                d.sprite(h, 0, cx - 8.0, cy - 8.0, false);
            } else {
                d.text("B", cx - 3.0, cy + 4.0, "#40e0c0");
            }
        }
        1 if pd.bomb_cap > 0 => {
            if let Some(h) = sprites.get("prop_bomb") {
                d.sprite(h, 0, cx - 8.0, cy - 8.0, false);
            } else {
                d.text("O", cx - 3.0, cy + 4.0, "#e8e8e8");
            }
            d.text(&format!("{}", pd.bombs), cx + 2.0, cy + 10.0, "#e8e8e8");
        }
        _ => {
            d.text("—", cx - 3.0, cy + 4.0, "#808080");
        }
    }
}

pub fn render_portrait_hint(d: &mut Draw) {
    d.rect(0.0, 0.0, 480.0, 270.0, "rgba(8,10,16,0.88)");
    d.text("ROTATE DEVICE", 170.0, 130.0, "#e8d080");
    d.text("↺", 230.0, 160.0, "#ffffff");
}

/// Hit-test a vertical menu row list. Returns row index if tap is inside.
pub fn hit_rows(tap: (f32, f32), origin_y: f32, row_h: f32, count: usize, x0: f32, x1: f32) -> Option<usize> {
    let (x, y) = tap;
    if x < x0 || x > x1 {
        return None;
    }
    if y < origin_y {
        return None;
    }
    let idx = ((y - origin_y) / row_h) as usize;
    if idx < count {
        Some(idx)
    } else {
        None
    }
}

pub const CORNER_MAP_X: f32 = 408.0;
pub const CORNER_MAP_Y: f32 = 8.0;
pub const CORNER_MAP_S: f32 = 68.0;

pub fn hit_corner_minimap(tap: (f32, f32)) -> bool {
    let (x, y) = tap;
    (CORNER_MAP_X..=CORNER_MAP_X + CORNER_MAP_S).contains(&x)
        && (CORNER_MAP_Y..=CORNER_MAP_Y + CORNER_MAP_S).contains(&y)
}
