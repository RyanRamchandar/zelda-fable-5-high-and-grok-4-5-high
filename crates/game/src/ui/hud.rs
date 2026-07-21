//! Atlas-skinned HUD. Layout constants preserved from 1A.

use engine::render::Draw;

use crate::assets::SpriteMap;
use crate::combat::style;
use crate::world::entity::EntityData;
use crate::world::World;

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

pub fn draw(d: &mut Draw, world: &World, sprites: &SpriteMap) {
    let Some(p) = world.get(world.player_id) else {
        return;
    };
    let EntityData::Player(pd) = &p.data else {
        return;
    };
    let hp = p.health.map(|h| h.hp).unwrap_or(pd.hearts);
    let max = p.health.map(|h| h.max).unwrap_or(pd.max_hearts);
    let hearts = max / 2;

    let full = sprites.get("heart_full");
    let half = sprites.get("heart_half");
    let empty = sprites.get("heart_empty");

    for i in 0..hearts {
        let x = HEART_X + i as f32 * HEART_GAP;
        let units = hp - i * 2;
        let h = if units >= 2 {
            full
        } else if units == 1 {
            half
        } else {
            empty
        };
        if let Some(h) = h {
            d.sprite(h, 0, x, HEART_Y, false);
        } else {
            d.rect(x, HEART_Y, HEART_W, HEART_H, "#e04040");
        }
    }

    if let Some(frame) = sprites.get("energy_frame") {
        d.sprite(frame, 0, ENERGY_X - 1.0, ENERGY_Y - 2.0, false);
    } else {
        d.rect(
            ENERGY_X - 1.0,
            ENERGY_Y - 1.0,
            ENERGY_W + 2.0,
            ENERGY_H + 2.0,
            "#202020",
        );
    }
    let fill = (pd.energy / 100.0).clamp(0.0, 1.0);
    let fh = ENERGY_H * fill;
    let flash = pd.energy_deny_flash > 0 && pd.energy_deny_flash % 2 == 0;
    if let Some(fill_h) = sprites.get("energy_fill") {
        // draw bottom portion of fill strip
        let src_h = (60.0 * fill) as u32;
        if src_h > 0 {
            // approximate with scaled rect tint over sprite crop via stacked pixels
            let ecol = if flash {
                "#ffffff"
            } else if pd.energy < 25.0 {
                "#4060a0"
            } else {
                "#40a0ff"
            };
            d.rect(ENERGY_X + 1.0, ENERGY_Y + (ENERGY_H - fh), 4.0, fh, ecol);
            let _ = fill_h;
        }
    } else {
        let ecol = if flash { "#ffffff" } else { "#40a0ff" };
        d.rect(ENERGY_X, ENERGY_Y + (ENERGY_H - fh), ENERGY_W, fh, ecol);
    }

    if let Some(chip) = sprites.get("style_chip") {
        d.sprite(chip, 0, STYLE_X - 1.0, STYLE_Y - 1.0, false);
    } else {
        d.rect(STYLE_X, STYLE_Y, 12.0, 12.0, "#303030");
    }
    let letter = style::rank_letter(pd.style_rank);
    let pulse = pd.style_pulse > 0;
    let scol = if pulse { "#ffff80" } else { "#f0f0f0" };
    d.text(letter, STYLE_X + 3.0, STYLE_Y + 10.0, scol);

    if let Some(slot) = sprites.get("item_slot") {
        d.sprite(slot, 0, ITEM_X - 1.0, ITEM_Y - 1.0, false);
    } else {
        d.rect(ITEM_X, ITEM_Y, ITEM_S, ITEM_S, "#303030");
    }
    if pd.bomb_cap > 0 {
        let flash = pd.item_cycle_flash > 0;
        if let Some(h) = sprites.get("prop_bomb") {
            d.sprite(h, 0, ITEM_X, ITEM_Y, false);
        } else {
            d.rect(ITEM_X + 4.0, ITEM_Y + 4.0, 12.0, 12.0, "#303848");
        }
        let col = if flash { "#40ffff" } else { "#e8e8e8" };
        d.text(&format!("{}", pd.bombs), ITEM_X + 12.0, ITEM_Y + 18.0, col);
    } else {
        d.text("—", ITEM_X + 6.0, ITEM_Y + 14.0, "#808080");
    }

    d.text(&format!("₹{}", pd.rupees), 8.0, 26.0, "#40e080");
}
