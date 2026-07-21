//! Pause overlay: Map / Help / Options pages.

use content::audio::sfx::SfxId;
use content::text::{self, TextId};
use engine::input::{
    InputState, BUTTON_ATTACK, BUTTON_CONFIRM, BUTTON_CYCLE, BUTTON_DASH, BUTTON_INTERACT,
    BUTTON_ITEM, BUTTON_PAUSE, BUTTON_COUNT,
};
use engine::render::Draw;

use crate::save_data::{has_flag, heart_piece_count, save_flags};
use crate::state::{self, save_from_game};
use crate::world::WorldEvent;
use crate::Game;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PausePage {
    Map,
    Help,
    Options,
}

pub struct PauseState {
    pub open: bool,
    pub page: PausePage,
    pub cursor: usize,
    prev_move_x: f32,
    prev_move_y: f32,
}

impl PauseState {
    pub fn new() -> Self {
        Self {
            open: false,
            page: PausePage::Map,
            cursor: 0,
            prev_move_x: 0.0,
            prev_move_y: 0.0,
        }
    }
}

impl Default for PauseState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn objective_line(gems: u8, flags: &[u16]) -> TextId {
    if !has_flag(flags, save_flags::QUEST_STARTED) {
        return TextId::ObjVisitElder;
    }
    let n = gems.count_ones();
    if n < 3 {
        return match n {
            0 => TextId::ObjGems0,
            1 => TextId::ObjGems1,
            _ => TextId::ObjGems2,
        };
    }
    if !has_flag(flags, save_flags::DOOR_SHRINE_OPEN) {
        return TextId::ObjOpenShrine;
    }
    if !has_flag(flags, save_flags::ITEM_BOOMERANG) {
        return TextId::ObjExploreShrine;
    }
    let seals = u32::from(has_flag(flags, save_flags::SEAL_WEST))
        + u32::from(has_flag(flags, save_flags::SEAL_EAST));
    if seals < 2 {
        return TextId::ObjBreakSeals;
    }
    if !has_flag(flags, save_flags::WARDEN_DEFEATED) {
        return TextId::ObjDefeatWarden;
    }
    TextId::ObjAct1Complete
}

/// Returns true while pause holds the world frozen.
pub fn update(game: &mut Game, input: &InputState) -> bool {
    if input.buttons[BUTTON_PAUSE].pressed {
        if game.ui.pause.open {
            close(game);
        } else {
            open(game);
        }
        return game.ui.pause.open;
    }

    if !game.ui.pause.open {
        return false;
    }

    let mx = input.move_vec.0;
    let my = input.move_vec.1;
    let prev_x = game.ui.pause.prev_move_x;
    let prev_y = game.ui.pause.prev_move_y;

    // Left/right cycles pages.
    if prev_x >= -0.4 && mx < -0.55 {
        cycle_page(game, -1);
    } else if prev_x <= 0.4 && mx > 0.55 {
        cycle_page(game, 1);
    }

    if game.ui.pause.page == PausePage::Options {
        if prev_y >= -0.4 && my < -0.55 {
            game.ui.pause.cursor = game.ui.pause.cursor.saturating_sub(1);
            blip_move(game);
        } else if prev_y <= 0.4 && my > 0.55 {
            game.ui.pause.cursor = (game.ui.pause.cursor + 1).min(3);
            blip_move(game);
        }
        if input.buttons[BUTTON_ATTACK].pressed
            || input.buttons[BUTTON_CONFIRM].pressed
            || input.buttons[BUTTON_INTERACT].pressed
        {
            activate_option(game, game.ui.pause.cursor);
        }
    }

    game.ui.pause.prev_move_x = mx;
    game.ui.pause.prev_move_y = my;

    if input.buttons[BUTTON_DASH].pressed {
        close(game);
    }

    game.ui.pause.open
}

fn open(game: &mut Game) {
    game.ui.pause.open = true;
    game.ui.pause.page = PausePage::Map;
    game.ui.pause.cursor = 0;
    game.ui.pause.prev_move_x = 0.0;
    game.ui.pause.prev_move_y = 0.0;
    game.world
        .push_event(WorldEvent::Sfx(SfxId::MenuConfirm));
}

fn close(game: &mut Game) {
    game.ui.pause.open = false;
    game.world.push_event(WorldEvent::Sfx(SfxId::MenuBack));
}

fn cycle_page(game: &mut Game, dir: i32) {
    let pages = [PausePage::Map, PausePage::Help, PausePage::Options];
    let idx = pages
        .iter()
        .position(|p| *p == game.ui.pause.page)
        .unwrap_or(0) as i32;
    let next = (idx + dir).rem_euclid(3) as usize;
    game.ui.pause.page = pages[next];
    game.ui.pause.cursor = 0;
    blip_move(game);
}

fn blip_move(game: &mut Game) {
    game.world.push_event(WorldEvent::Sfx(SfxId::MenuMove));
}

fn activate_option(game: &mut Game, cursor: usize) {
    match cursor {
        0 => close(game),
        1 => {
            // Restart from checkpoint (death path without heart reset).
            let map = game.current_map;
            let cp = game.world.checkpoint;
            close(game);
            state::switch_map(game, map, cp);
            game.world
                .push_event(WorldEvent::Sfx(SfxId::MenuConfirm));
        }
        2 => {
            // Quit to title — flush save first.
            if let Some(json) = save_from_game(game).to_json() {
                game.pending_save = Some(json);
            }
            close(game);
            game.mode = crate::state::GameMode::Title;
            game.ui.title.page = crate::ui::title::TitlePage::Main;
            game.ui.title.cursor = 0;
            game.ui.title.confirm_wipe = false;
            game.had_save = true;
            game.world
                .push_event(WorldEvent::Sfx(SfxId::MenuConfirm));
        }
        3 => {
            game.settings.muted = !game.settings.muted;
            game.pending_muted = Some(game.settings.muted);
            if !game.settings.muted {
                game.world
                    .push_event(WorldEvent::Sfx(SfxId::MenuConfirm));
            }
            if let Some(json) = save_from_game(game).to_json() {
                game.pending_save = Some(json);
            }
        }
        _ => {}
    }
}

pub fn render(d: &mut Draw, game: &Game) {
    if !game.ui.pause.open {
        return;
    }

    match game.ui.pause.page {
        PausePage::Map => {
            if game.current_map == content::maps::MapId::Dungeon {
                crate::ui::dungeon_map::render_pause(d, game);
            } else {
                game.ui.minimap.render_pause(
                    d,
                    &game.world,
                    &game.sprites,
                    game.current_map,
                    game.gems,
                    &game.flags,
                );
            }
        }
        PausePage::Help => render_help(d, game),
        PausePage::Options => render_options(d, game),
    }

    // Tab bar on top of page content.
    d.rect(0.0, 0.0, 480.0, 22.0, "rgba(8,10,16,0.85)");
    for (i, (page, id)) in [
        (PausePage::Map, TextId::PauseMap),
        (PausePage::Help, TextId::PauseHelp),
        (PausePage::Options, TextId::PauseOptions),
    ]
    .iter()
    .enumerate()
    {
        let x = 40.0 + i as f32 * 100.0;
        let col = if game.ui.pause.page == *page {
            "#ffe080"
        } else {
            "#808890"
        };
        d.text(text::line(*id), x, 16.0, col);
    }
}

fn render_help(d: &mut Draw, game: &Game) {
    d.rect(0.0, 22.0, 480.0, 248.0, "rgba(0,0,0,0.55)");
    let obj = objective_line(game.gems, &game.flags);
    d.text("OBJECTIVE:", 16.0, 40.0, "#ffe080");
    d.text(text::line(obj), 100.0, 40.0, "#e8e8e8");

    d.text("KEYBOARD", 16.0, 60.0, "#a0c0e0");
    d.text("GAMEPAD", 170.0, 60.0, "#a0c0e0");
    d.text("TOUCH", 320.0, 60.0, "#a0c0e0");

    for (i, row) in text::binding_rows().iter().enumerate() {
        let y = 78.0 + i as f32 * 16.0;
        d.text(row.verb, 16.0, y, "#c0c8d0");
        d.text(row.keyboard, 90.0, y, "#a0a8b8");
        d.text(row.gamepad, 170.0, y, "#a0a8b8");
        d.text(row.touch, 320.0, y, "#a0a8b8");
    }

    // Live input echo (from last update snapshot).
    let input = &game.last_input;
    let (mx, my) = input.move_vec;
    let cx = 40.0 + mx * 14.0;
    let cy = 230.0 + my * 14.0;
    d.rect(24.0, 214.0, 36.0, 36.0, "rgba(40,48,64,0.8)");
    d.circle(40.0, 230.0, 14.0, "rgba(80,90,110,0.5)");
    d.circle(cx, cy, 4.0, "#ffffff");

    let labels = ["A", "I", "D", "E", "P", "C", "Q"];
    let idxs = [
        BUTTON_ATTACK,
        BUTTON_ITEM,
        BUTTON_DASH,
        BUTTON_INTERACT,
        BUTTON_PAUSE,
        BUTTON_CONFIRM,
        BUTTON_CYCLE,
    ];
    for (i, (&btn, lab)) in idxs.iter().zip(labels.iter()).enumerate() {
        let x = 80.0 + i as f32 * 28.0;
        let held = btn < BUTTON_COUNT && input.buttons[btn].held;
        let col = if held {
            "#ffe080"
        } else {
            "rgba(60,68,84,0.9)"
        };
        d.rect(x, 218.0, 22.0, 22.0, col);
        d.text(lab, x + 6.0, 234.0, "#101018");
    }
    let _ = heart_piece_count;
}

fn render_options(d: &mut Draw, game: &Game) {
    d.rect(0.0, 22.0, 480.0, 248.0, "rgba(0,0,0,0.55)");
    let sound = if game.settings.muted {
        TextId::MenuSoundOff
    } else {
        TextId::MenuSoundOn
    };
    let rows = [
        TextId::MenuResume,
        TextId::MenuRestartCheckpoint,
        TextId::MenuQuitTitle,
        sound,
    ];
    for (i, id) in rows.iter().enumerate() {
        let y = 80.0 + i as f32 * 28.0;
        let col = if i == game.ui.pause.cursor {
            "#ffffff"
        } else {
            "#a0a8b8"
        };
        let mark = if i == game.ui.pause.cursor { ">" } else { " " };
        d.text(&format!("{} {}", mark, text::line(*id)), 140.0, y, col);
    }
}
