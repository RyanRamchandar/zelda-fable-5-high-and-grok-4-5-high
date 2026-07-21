//! Title screen + chapter select.

use content::audio::sfx::SfxId;
use content::maps::MapId;
use content::text::{self, TextId};
use engine::input::{
    InputState, BUTTON_ATTACK, BUTTON_CONFIRM, BUTTON_DASH, BUTTON_INTERACT, BUTTON_PAUSE,
};
use engine::render::Draw;

use crate::save_data::{has_flag, heart_piece_count, save_flags, SaveGame};
use crate::state::{self, save_from_game};
use crate::world::entity::EntityData;
use crate::world::WorldEvent;
use crate::Game;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TitlePage {
    Main,
    Chapters,
}

pub struct TitleState {
    pub cursor: usize,
    pub page: TitlePage,
    pub confirm_wipe: bool,
    /// Chapter card index 0..2; within-card action 0=Play/locked, 1=Restart.
    pub chapter_cursor: usize,
    pub chapter_action: usize,
    pub confirm_restart: bool,
    prev_move_x: f32,
    prev_move_y: f32,
    sparkle_t: u32,
}

impl TitleState {
    pub fn new() -> Self {
        Self {
            cursor: 0,
            page: TitlePage::Main,
            confirm_wipe: false,
            chapter_cursor: 0,
            chapter_action: 0,
            confirm_restart: false,
            prev_move_x: 0.0,
            prev_move_y: 0.0,
            sparkle_t: 0,
        }
    }
}

impl Default for TitleState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn has_progress(game: &Game) -> bool {
    game.had_save && (game.world.checkpoint > 0 || !game.flags.is_empty())
}

pub fn update(game: &mut Game, input: &InputState) {
    game.ui.title.sparkle_t = game.ui.title.sparkle_t.wrapping_add(1);
    match game.ui.title.page {
        TitlePage::Main => update_main(game, input),
        TitlePage::Chapters => update_chapters(game, input),
    }
}

fn update_main(game: &mut Game, input: &InputState) {
    let rows = main_rows(game);
    let len = rows.len();
    let my = input.move_vec.1;
    let prev = game.ui.title.prev_move_y;
    if prev >= -0.4 && my < -0.55 {
        game.ui.title.cursor = game.ui.title.cursor.saturating_sub(1);
        blip(game, SfxId::MenuMove);
    } else if prev <= 0.4 && my > 0.55 {
        game.ui.title.cursor = (game.ui.title.cursor + 1).min(len.saturating_sub(1));
        blip(game, SfxId::MenuMove);
    }
    game.ui.title.prev_move_y = my;
    game.ui.title.prev_move_x = input.move_vec.0;

    if game.ui.title.confirm_wipe {
        if input.buttons[BUTTON_DASH].pressed || input.buttons[BUTTON_PAUSE].pressed {
            game.ui.title.confirm_wipe = false;
            game.ui.title.cursor = 0;
            blip(game, SfxId::MenuBack);
            return;
        }
        if input.buttons[BUTTON_ATTACK].pressed
            || input.buttons[BUTTON_CONFIRM].pressed
            || input.buttons[BUTTON_INTERACT].pressed
        {
            if game.ui.title.cursor == 0 {
                game.ui.title.confirm_wipe = false;
                blip(game, SfxId::MenuBack);
            } else {
                apply_new_game(game, false);
                game.mode = state::GameMode::Play;
                blip(game, SfxId::MenuConfirm);
            }
        }
        return;
    }

    if input.buttons[BUTTON_ATTACK].pressed
        || input.buttons[BUTTON_CONFIRM].pressed
        || input.buttons[BUTTON_INTERACT].pressed
    {
        activate_main(game, rows.get(game.ui.title.cursor).copied());
    }
}

fn main_rows(game: &Game) -> Vec<MainRow> {
    let mut rows = Vec::new();
    if has_progress(game) {
        rows.push(MainRow::Continue);
    }
    rows.push(MainRow::NewGame);
    rows.push(MainRow::Chapters);
    rows.push(MainRow::Sound);
    rows
}

#[derive(Clone, Copy)]
enum MainRow {
    Continue,
    NewGame,
    Chapters,
    Sound,
}

fn activate_main(game: &mut Game, row: Option<MainRow>) {
    let Some(row) = row else {
        return;
    };
    match row {
        MainRow::Continue => {
            game.mode = state::GameMode::Play;
            blip(game, SfxId::MenuConfirm);
        }
        MainRow::NewGame => {
            if has_progress(game) {
                game.ui.title.confirm_wipe = true;
                game.ui.title.cursor = 0; // NO default
                blip(game, SfxId::MenuConfirm);
            } else {
                apply_new_game(game, false);
                game.mode = state::GameMode::Play;
                blip(game, SfxId::MenuConfirm);
            }
        }
        MainRow::Chapters => {
            game.ui.title.page = TitlePage::Chapters;
            game.ui.title.chapter_cursor = 0;
            game.ui.title.chapter_action = 0;
            game.ui.title.confirm_restart = false;
            blip(game, SfxId::MenuConfirm);
        }
        MainRow::Sound => {
            game.settings.muted = !game.settings.muted;
            game.pending_muted = Some(game.settings.muted);
            if !game.settings.muted {
                blip(game, SfxId::MenuConfirm);
            }
            if let Some(json) = save_from_game(game).to_json() {
                game.pending_save = Some(json);
            }
        }
    }
}

fn update_chapters(game: &mut Game, input: &InputState) {
    let mx = input.move_vec.0;
    let my = input.move_vec.1;
    let prev_x = game.ui.title.prev_move_x;
    let prev_y = game.ui.title.prev_move_y;

    if input.buttons[BUTTON_DASH].pressed || input.buttons[BUTTON_PAUSE].pressed {
        if game.ui.title.confirm_restart {
            game.ui.title.confirm_restart = false;
            blip(game, SfxId::MenuBack);
        } else {
            game.ui.title.page = TitlePage::Main;
            game.ui.title.cursor = 0;
            blip(game, SfxId::MenuBack);
        }
        game.ui.title.prev_move_x = mx;
        game.ui.title.prev_move_y = my;
        return;
    }

    if game.ui.title.confirm_restart {
        if prev_y >= -0.4 && my < -0.55 {
            game.ui.title.chapter_action = 0;
            blip(game, SfxId::MenuMove);
        } else if prev_y <= 0.4 && my > 0.55 {
            game.ui.title.chapter_action = 1;
            blip(game, SfxId::MenuMove);
        }
        if input.buttons[BUTTON_ATTACK].pressed
            || input.buttons[BUTTON_CONFIRM].pressed
            || input.buttons[BUTTON_INTERACT].pressed
        {
            if game.ui.title.chapter_action == 0 {
                game.ui.title.confirm_restart = false;
                blip(game, SfxId::MenuBack);
            } else {
                apply_new_game(game, true);
                game.mode = state::GameMode::Play;
                blip(game, SfxId::MenuConfirm);
            }
        }
        game.ui.title.prev_move_x = mx;
        game.ui.title.prev_move_y = my;
        return;
    }

    if prev_x >= -0.4 && mx < -0.55 {
        game.ui.title.chapter_cursor = game.ui.title.chapter_cursor.saturating_sub(1);
        game.ui.title.chapter_action = 0;
        blip(game, SfxId::MenuMove);
    } else if prev_x <= 0.4 && mx > 0.55 {
        game.ui.title.chapter_cursor = (game.ui.title.chapter_cursor + 1).min(2);
        game.ui.title.chapter_action = 0;
        blip(game, SfxId::MenuMove);
    }

    if game.ui.title.chapter_cursor == 0 {
        if prev_y >= -0.4 && my < -0.55 {
            game.ui.title.chapter_action = 0;
            blip(game, SfxId::MenuMove);
        } else if prev_y <= 0.4 && my > 0.55 {
            game.ui.title.chapter_action = 1;
            blip(game, SfxId::MenuMove);
        }
    }

    if input.buttons[BUTTON_ATTACK].pressed
        || input.buttons[BUTTON_CONFIRM].pressed
        || input.buttons[BUTTON_INTERACT].pressed
    {
        match game.ui.title.chapter_cursor {
            0 => {
                if game.ui.title.chapter_action == 0 {
                    game.mode = state::GameMode::Play;
                    blip(game, SfxId::MenuConfirm);
                } else {
                    game.ui.title.confirm_restart = true;
                    game.ui.title.chapter_action = 0;
                    blip(game, SfxId::MenuConfirm);
                }
            }
            _ => blip(game, SfxId::Refused),
        }
    }

    game.ui.title.prev_move_x = mx;
    game.ui.title.prev_move_y = my;
}

pub fn apply_new_game(game: &mut Game, keep_rupees: bool) {
    let rupees = if keep_rupees {
        player_rupees(game)
    } else {
        0
    };
    let muted = game.settings.muted;
    game.flags.clear();
    game.gems = 0;
    game.ui.minimap.load_fog(&[]);
    game.ui.minimap.refresh_objective(0, &[]);
    game.boss = None;
    game.rooms = None;
    game.dungeon_puzzle = None;
    if let Some(p) = game.world.get_mut(game.world.player_id) {
        if let EntityData::Player(pd) = &mut p.data {
            pd.hearts = 6;
            pd.max_hearts = 6;
            pd.rupees = 0;
            pd.bombs = 0;
            pd.bomb_cap = 0;
            pd.selected_item = 0;
            pd.energy = 100.0;
            pd.style_points = 0.0;
            pd.style_rank = 0;
        }
        if let Some(h) = p.health.as_mut() {
            h.hp = 6;
            h.max = 6;
        }
    }
    game.world.checkpoint = 0;
    state::switch_map(game, MapId::Overworld, 0);
    if let Some(p) = game.world.get_mut(game.world.player_id) {
        if let EntityData::Player(pd) = &mut p.data {
            pd.hearts = 6;
            pd.max_hearts = 6;
            pd.rupees = rupees;
            pd.bombs = 0;
            pd.bomb_cap = 0;
            pd.selected_item = 0;
        }
        if let Some(h) = p.health.as_mut() {
            h.hp = 6;
            h.max = 6;
        }
    }
    game.settings.muted = muted;
    game.had_save = true;
    game.ui.title.confirm_wipe = false;
    game.ui.title.confirm_restart = false;
    if let Some(json) = save_from_game(game).to_json() {
        game.pending_save = Some(json);
    }
    let _ = SaveGame::default_spawn;
}

fn player_rupees(game: &Game) -> u32 {
    game.world
        .get(game.world.player_id)
        .and_then(|p| match &p.data {
            EntityData::Player(pd) => Some(pd.rupees),
            _ => None,
        })
        .unwrap_or(0)
}

fn blip(game: &mut Game, id: SfxId) {
    game.world.push_event(WorldEvent::Sfx(id));
}

pub fn render(d: &mut Draw, game: &Game) {
    d.rect(0.0, 0.0, 480.0, 270.0, "#0a0a12");

    // Sparkles
    let t = game.ui.title.sparkle_t;
    for i in 0..12u32 {
        let x = ((t.wrapping_mul(3) + i * 37) % 460) as f32 + 10.0;
        let y = ((t.wrapping_mul(2) + i * 53) % 250) as f32 + 8.0;
        let on = ((t / 8 + i) % 5) < 2;
        if on {
            d.rect(x, y, 2.0, 2.0, "#e8d080");
        }
    }

    if let Some(logo) = game.sprites.get("title_logo") {
        let frame = (t / 24) % 2;
        d.sprite(logo, frame, 216.0, 28.0, false);
    }
    d.text("SHARD OF THE TRIFORCE", 150.0, 60.0, "#e8d080");

    match game.ui.title.page {
        TitlePage::Main => render_main(d, game),
        TitlePage::Chapters => render_chapters(d, game),
    }
}

fn render_main(d: &mut Draw, game: &Game) {
    if game.ui.title.confirm_wipe {
        d.text(text::line(TextId::MenuEraseSave), 180.0, 120.0, "#ff8080");
        for (i, id) in [TextId::MenuEraseNo, TextId::MenuEraseYes]
            .iter()
            .enumerate()
        {
            let y = 150.0 + i as f32 * 24.0;
            let col = if i == game.ui.title.cursor {
                "#ffffff"
            } else {
                "#a0a8b8"
            };
            let mark = if i == game.ui.title.cursor { ">" } else { " " };
            d.text(&format!("{} {}", mark, text::line(*id)), 200.0, y, col);
        }
        return;
    }

    let rows = main_rows(game);
    for (i, row) in rows.iter().enumerate() {
        let y = 110.0 + i as f32 * 26.0;
        let label = match row {
            MainRow::Continue => text::line(TextId::MenuContinue),
            MainRow::NewGame => text::line(TextId::MenuNewGame),
            MainRow::Chapters => text::line(TextId::MenuChapterSelect),
            MainRow::Sound => {
                if game.settings.muted {
                    text::line(TextId::MenuSoundOff)
                } else {
                    text::line(TextId::MenuSoundOn)
                }
            }
        };
        let col = if i == game.ui.title.cursor {
            "#ffffff"
        } else {
            "#a0a8b8"
        };
        let mark = if i == game.ui.title.cursor { ">" } else { " " };
        d.text(&format!("{mark} {label}"), 170.0, y, col);
    }
}

fn render_chapters(d: &mut Draw, game: &Game) {
    let cards = [
        ("ACT 1", "Shard of Courage", true),
        ("ACT 2", text::line(TextId::Act2Card), false),
        ("ACT 3", text::line(TextId::Act3Card), false),
    ];
    for (i, (title, sub, unlocked)) in cards.iter().enumerate() {
        let x = 30.0 + i as f32 * 150.0;
        let selected = game.ui.title.chapter_cursor == i;
        let bg = if selected {
            "rgba(40,48,72,0.95)"
        } else {
            "rgba(20,24,36,0.9)"
        };
        d.rect(x, 80.0, 140.0, 150.0, bg);
        let tcol = if *unlocked { "#e8d080" } else { "#606870" };
        d.text(title, x + 40.0, 100.0, tcol);
        d.text(sub, x + 8.0, 120.0, if *unlocked { "#c0c8d0" } else { "#505860" });
        if !*unlocked {
            d.text("LOCKED", x + 40.0, 160.0, "#804040");
        } else {
            let gems = game.gems.count_ones();
            let hp = heart_piece_count(&game.flags);
            let rupees = player_rupees(game);
            d.text(&format!("GEMS {gems}/3"), x + 20.0, 140.0, "#80e0a0");
            d.text(&format!("HEARTS {hp}/4"), x + 16.0, 156.0, "#e08080");
            d.text(&format!("R {rupees}"), x + 40.0, 172.0, "#40e080");
            if has_flag(&game.flags, save_flags::SHARD_OF_COURAGE) {
                d.text("COMPLETE", x + 30.0, 188.0, "#ffe080");
            }
            let play_col = if selected && game.ui.title.chapter_action == 0 {
                "#ffffff"
            } else {
                "#a0a8b8"
            };
            let rest_col = if selected && game.ui.title.chapter_action == 1 {
                "#ffffff"
            } else {
                "#a0a8b8"
            };
            d.text(text::line(TextId::MenuPlay), x + 48.0, 204.0, play_col);
            d.text(text::line(TextId::MenuRestart), x + 36.0, 220.0, rest_col);
        }
    }

    if game.ui.title.confirm_restart {
        d.rect(120.0, 100.0, 240.0, 90.0, "rgba(10,12,20,0.95)");
        d.text("RESTART ACT 1?", 170.0, 120.0, "#ff8080");
        d.text("(rupees carry)", 175.0, 138.0, "#808890");
        for (i, id) in [TextId::MenuEraseNo, TextId::MenuEraseYes]
            .iter()
            .enumerate()
        {
            let y = 156.0 + i as f32 * 18.0;
            let col = if i == game.ui.title.chapter_action {
                "#ffffff"
            } else {
                "#a0a8b8"
            };
            d.text(text::line(*id), 200.0, y, col);
        }
    }
}
