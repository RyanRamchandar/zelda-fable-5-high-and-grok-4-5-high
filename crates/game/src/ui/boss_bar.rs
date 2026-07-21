//! Boss-only segmented HP bar (bottom-center).

use engine::render::Draw;

const BAR_W: f32 = 200.0;
const BAR_H: f32 = 10.0;
const BAR_X: f32 = (480.0 - BAR_W) * 0.5;
const BAR_Y: f32 = 248.0;

/// Notch fractions: phase gates at 75% and 35%.
const NOTCH_A: f32 = 0.75;
const NOTCH_B: f32 = 0.35;

pub fn draw(d: &mut Draw, hp: f32, max_hp: f32) {
    let max = max_hp.max(1.0);
    let frac = (hp / max).clamp(0.0, 1.0);

    d.rect(BAR_X - 2.0, BAR_Y - 2.0, BAR_W + 4.0, BAR_H + 4.0, "#1a1410");
    d.rect(BAR_X, BAR_Y, BAR_W, BAR_H, "#3a2820");

    let fill_w = BAR_W * frac;
    let color = if frac > NOTCH_A {
        "#c8a060"
    } else if frac > NOTCH_B {
        "#d08040"
    } else {
        "#e05040"
    };
    if fill_w > 0.5 {
        d.rect(BAR_X, BAR_Y, fill_w, BAR_H, color);
    }

    // Phase notches.
    for n in [NOTCH_A, NOTCH_B] {
        let x = BAR_X + BAR_W * n;
        d.rect(x - 1.0, BAR_Y - 1.0, 2.0, BAR_H + 2.0, "#f0e8d0");
    }

    d.text("GRANITE WARDEN", BAR_X + 52.0, BAR_Y - 12.0, "#e8e0c8");
}
