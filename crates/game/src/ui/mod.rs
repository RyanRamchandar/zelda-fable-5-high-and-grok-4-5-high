//! HUD and overlays (functional gray-box; 1B skins with atlas).

pub mod hud;

use engine::render::Draw;

use crate::fx::FxState;
use crate::world::World;

pub struct UiState {
    pub debug_overlay: bool,
    pub renders: u32,
    pub fps_est: f32,
    pub fps_accum: u32,
}

impl UiState {
    pub fn new() -> Self {
        Self {
            debug_overlay: false,
            renders: 0,
            fps_est: 60.0,
            fps_accum: 0,
        }
    }
}

impl Default for UiState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn render_hud(d: &mut Draw, world: &World) {
    hud::draw(d, world);
}

pub fn render_debug(d: &mut Draw, world: &World, ui: &UiState, fx: &FxState, player_state: &str) {
    if !ui.debug_overlay {
        return;
    }
    let pid = world.player_id;
    let (pos, energy, style, hitstop) = {
        let Some(p) = world.get(pid) else {
            return;
        };
        match &p.data {
            crate::world::entity::EntityData::Player(pd) => (
                p.pos,
                pd.energy,
                pd.style_points,
                world.hitstop,
            ),
            _ => (p.pos, 0.0, 0.0, world.hitstop),
        }
    };
    let lines = [
        format!("fps~{:.0}", ui.fps_est),
        format!("ents {}", world.entity_count()),
        format!("pos {:.0},{:.0}", pos.x, pos.y),
        format!("state {player_state}"),
        format!("en {:.0} st {:.0}", energy, style),
        format!("hitstop {hitstop}"),
        format!("fx {}", fx.particle_count()),
    ];
    for (i, line) in lines.iter().enumerate() {
        d.text(line, 300.0, 12.0 + i as f32 * 11.0, "#a0ffa0");
    }
}
