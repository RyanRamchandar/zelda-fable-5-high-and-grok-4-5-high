//! Functional plain-rect HUD. Layout constants stay here for 1B skinning.

use engine::render::Draw;

use crate::combat::style;
use crate::world::entity::EntityData;
use crate::world::World;

// Layout (screen space) — 1B replaces drawing, keeps positions.
pub const HEART_X: f32 = 8.0;
pub const HEART_Y: f32 = 8.0;
pub const HEART_W: f32 = 10.0;
pub const HEART_H: f32 = 8.0;
pub const HEART_GAP: f32 = 12.0;

pub const ENERGY_X: f32 = 4.0;
pub const ENERGY_Y: f32 = 40.0;
pub const ENERGY_W: f32 = 6.0;
pub const ENERGY_H: f32 = 60.0;

pub const STYLE_X: f32 = 14.0;
pub const STYLE_Y: f32 = 42.0;

pub const ITEM_X: f32 = 450.0;
pub const ITEM_Y: f32 = 240.0;
pub const ITEM_S: f32 = 20.0;

pub fn draw(d: &mut Draw, world: &World) {
    let Some(p) = world.get(world.player_id) else {
        return;
    };
    let EntityData::Player(pd) = &p.data else {
        return;
    };
    let hp = p.health.map(|h| h.hp).unwrap_or(pd.hearts);
    let max = p.health.map(|h| h.max).unwrap_or(pd.max_hearts);
    let hearts = max / 2;

    for i in 0..hearts {
        let x = HEART_X + i as f32 * HEART_GAP;
        let units = hp - i * 2;
        let color = if units >= 2 {
            "#e04040"
        } else if units == 1 {
            "#a03030"
        } else {
            "#402020"
        };
        d.rect(x, HEART_Y, HEART_W, HEART_H, color);
        if units == 1 {
            d.rect(x + HEART_W * 0.5, HEART_Y, HEART_W * 0.5, HEART_H, "#402020");
        }
    }

    // Energy bar
    d.rect(ENERGY_X - 1.0, ENERGY_Y - 1.0, ENERGY_W + 2.0, ENERGY_H + 2.0, "#202020");
    let fill = (pd.energy / 100.0).clamp(0.0, 1.0);
    let fh = ENERGY_H * fill;
    let flash = pd.energy_deny_flash > 0 && pd.energy_deny_flash % 2 == 0;
    let ecol = if flash {
        "#ffffff"
    } else if pd.energy < 25.0 {
        "#4060a0"
    } else {
        "#40a0ff"
    };
    d.rect(ENERGY_X, ENERGY_Y + (ENERGY_H - fh), ENERGY_W, fh, ecol);

    // Style chip
    let letter = style::rank_letter(pd.style_rank);
    let pulse = pd.style_pulse > 0;
    let scol = if pulse { "#ffff80" } else { "#d0d0d0" };
    d.rect(STYLE_X, STYLE_Y, 12.0, 12.0, "#303030");
    d.text(letter, STYLE_X + 3.0, STYLE_Y + 10.0, scol);

    // B-item slot empty
    d.rect(ITEM_X, ITEM_Y, ITEM_S, ITEM_S, "#303030");
    d.text("—", ITEM_X + 5.0, ITEM_Y + 14.0, "#808080");

    // Rupees
    d.text(&format!("₹{}", pd.rupees), 8.0, 24.0, "#40e080");
}
