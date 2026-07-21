//! Village shop menu (bombs, bag, heart piece, locked tunic).

use content::audio::sfx::SfxId;
use content::text;
use engine::input::{InputState, BUTTON_ATTACK, BUTTON_DASH, BUTTON_INTERACT, BUTTON_PAUSE};
use engine::render::Draw;

use crate::fx::FxKind;
use crate::save_data::{
    has_flag, maybe_apply_heart_container, set_flag, save_flags, SaveGame,
};
use crate::state::save_from_game;
use crate::world::entity::EntityData;
use crate::world::WorldEvent;
use crate::Game;

const STOCK_LEN: usize = 4;

#[derive(Clone, Debug, Default)]
pub struct ShopState {
    pub open: bool,
    pub cursor: usize,
    prev_move_y: f32,
}

impl ShopState {
    pub fn new() -> Self {
        Self::default()
    }
}

pub fn open(game: &mut Game) {
    game.ui.shop.open = true;
    game.ui.shop.cursor = 0;
    game.ui.shop.prev_move_y = 0.0;
    game.ui.dialog.open_text(shop_greeting(game));
}

fn shop_greeting(game: &Game) -> content::text::TextId {
    if has_flag(&game.flags, save_flags::HEART_PIECE_4) {
        content::text::TextId::ShopkeeperAfterHeart
    } else if has_flag(&game.flags, save_flags::SHOP_BOMB_BAG) {
        content::text::TextId::ShopkeeperAfterBag
    } else {
        content::text::TextId::ShopkeeperIntro
    }
}

pub fn update(game: &mut Game, input: &InputState) -> Option<String> {
    if !game.ui.shop.open {
        return None;
    }
    // Wait until greeting dialog closes before navigating stock.
    if game.ui.dialog.open {
        game.ui.dialog.update(input, &mut game.world);
        return None;
    }

    let my = input.move_vec.1;
    let prev = game.ui.shop.prev_move_y;
    if prev >= -0.4 && my < -0.55 {
        game.ui.shop.cursor = game.ui.shop.cursor.saturating_sub(1);
    } else if prev <= 0.4 && my > 0.55 {
        game.ui.shop.cursor = (game.ui.shop.cursor + 1).min(STOCK_LEN - 1);
    }
    game.ui.shop.prev_move_y = my;

    if input.buttons[BUTTON_PAUSE].pressed || input.buttons[BUTTON_DASH].pressed {
        game.ui.shop.open = false;
        return None;
    }

    if let Some(tap) = input.menu_tap {
        if let Some(idx) = crate::ui::touch::hit_rows(tap, 88.0, 28.0, STOCK_LEN, 50.0, 420.0) {
            if idx != game.ui.shop.cursor {
                game.ui.shop.cursor = idx;
                return None;
            }
            return try_buy(game, idx);
        }
    }

    if input.buttons[BUTTON_ATTACK].pressed || input.buttons[BUTTON_INTERACT].pressed {
        return try_buy(game, game.ui.shop.cursor);
    }
    None
}

fn try_buy(game: &mut Game, idx: usize) -> Option<String> {
    match idx {
        0 => buy_bombs(game),
        1 => buy_bag(game),
        2 => buy_heart(game),
        3 => buy_tunic(game),
        _ => None,
    }
}

fn buy_bombs(game: &mut Game) -> Option<String> {
    let price = 10u32;
    let (rupees, cap, bombs) = player_wallet(game);
    if cap == 0 {
        // First purchase unlocks.
    } else if bombs >= cap {
        refuse(game, "POUCH FULL");
        return None;
    }
    if rupees < price {
        refuse(game, "NOT ENOUGH");
        return None;
    }
    with_player(game, |pd, flags| {
        pd.rupees -= price;
        if !has_flag(flags, save_flags::SHOP_BOMBS_UNLOCKED) {
            set_flag(flags, save_flags::SHOP_BOMBS_UNLOCKED);
            pd.bomb_cap = 10;
            pd.selected_item = 1;
        }
        let room = pd.bomb_cap.saturating_sub(pd.bombs);
        let add = 5u8.min(room);
        pd.bombs = pd.bombs.saturating_add(add);
    });
    game.world.push_event(WorldEvent::Sfx(SfxId::BuyItem));
    game.world.push_event(WorldEvent::FxRequest(FxKind::Toast {
        text: "BOMBS +5",
    }));
    save_from_game(game).to_json()
}

fn buy_bag(game: &mut Game) -> Option<String> {
    if has_flag(&game.flags, save_flags::SHOP_BOMB_BAG) {
        refuse(game, "SOLD OUT");
        return None;
    }
    let price = 100u32;
    if player_wallet(game).0 < price {
        refuse(game, "NOT ENOUGH");
        return None;
    }
    with_player(game, |pd, flags| {
        pd.rupees -= price;
        set_flag(flags, save_flags::SHOP_BOMB_BAG);
        set_flag(flags, save_flags::SHOP_BOMBS_UNLOCKED);
        pd.bomb_cap = 20;
        pd.selected_item = 1;
        pd.bombs = pd.bombs.saturating_add(5).min(pd.bomb_cap);
    });
    game.world.push_event(WorldEvent::Sfx(SfxId::BuyItem));
    game.world.push_event(WorldEvent::FxRequest(FxKind::Toast {
        text: "BOMB BAG!",
    }));
    save_from_game(game).to_json()
}

fn buy_heart(game: &mut Game) -> Option<String> {
    if has_flag(&game.flags, save_flags::HEART_PIECE_4) {
        refuse(game, "SOLD OUT");
        return None;
    }
    let price = 200u32;
    if player_wallet(game).0 < price {
        refuse(game, "NOT ENOUGH");
        return None;
    }
    let mut max_up = false;
    with_player(game, |pd, flags| {
        pd.rupees -= price;
        set_flag(flags, save_flags::HEART_PIECE_4);
        if maybe_apply_heart_container(flags, &mut pd.max_hearts) {
            pd.hearts = pd.max_hearts;
            max_up = true;
        }
    });
    if let Some(p) = game.world.get_mut(game.world.player_id) {
        if let Some(h) = p.health.as_mut() {
            if let EntityData::Player(pd) = &p.data {
                h.max = pd.max_hearts;
                h.hp = pd.hearts;
            }
        }
    }
    game.world.push_event(WorldEvent::Sfx(SfxId::BuyItem));
    game.world.push_event(WorldEvent::FxRequest(FxKind::Toast {
        text: if max_up {
            "MAX HEART UP!"
        } else {
            "HEART PIECE!"
        },
    }));
    save_from_game(game).to_json()
}

fn buy_tunic(game: &mut Game) -> Option<String> {
    if !has_flag(&game.flags, save_flags::TUNIC_UNLOCKED) {
        refuse(game, "AFTER THE SHRINE");
        return None;
    }
    if has_flag(&game.flags, save_flags::TUNIC_BOUGHT) {
        refuse(game, "SOLD OUT");
        return None;
    }
    let price = 300u32;
    let (rupees, _, _) = player_wallet(game);
    if rupees < price {
        refuse(game, "NOT ENOUGH");
        return None;
    }
    with_player(game, |pd, flags| {
        pd.rupees -= price;
        set_flag(flags, save_flags::TUNIC_BOUGHT);
    });
    game.world.push_event(WorldEvent::Sfx(SfxId::BuyItem));
    game.ui.dialog.open_text(content::text::TextId::TunicBought);
    // Cosmetic palette swap deferred to Phase 5 polish (logged in WORKER_NOTES).
    save_from_game(game).to_json()
}

fn refuse(game: &mut Game, toast: &'static str) {
    game.world.push_event(WorldEvent::Sfx(SfxId::Refused));
    game.world
        .push_event(WorldEvent::FxRequest(FxKind::Toast { text: toast }));
}

fn player_wallet(game: &Game) -> (u32, u8, u8) {
    game.world
        .get(game.world.player_id)
        .and_then(|p| match &p.data {
            EntityData::Player(pd) => Some((pd.rupees, pd.bomb_cap, pd.bombs)),
            _ => None,
        })
        .unwrap_or((0, 0, 0))
}

fn with_player(game: &mut Game, f: impl FnOnce(&mut crate::world::entity::PlayerData, &mut Vec<u16>)) {
    let pid = game.world.player_id;
    let mut flags = std::mem::take(&mut game.flags);
    if let Some(p) = game.world.get_mut(pid) {
        if let EntityData::Player(pd) = &mut p.data {
            f(pd, &mut flags);
        }
    }
    game.flags = flags;
}

pub fn render(d: &mut Draw, shop: &ShopState, flags: &[u16]) {
    if !shop.open {
        return;
    }
    d.rect(40.0, 40.0, 400.0, 190.0, "rgba(16,20,28,0.92)");
    d.rect(42.0, 42.0, 396.0, 186.0, "rgba(48,56,72,0.95)");
    d.text("MOSLIGHT GOODS", 56.0, 62.0, "#ffe080");

    let rows = [
        ("Bombs x5", "10", stock_bombs(flags)),
        ("Bomb Bag", "100", stock_bag(flags)),
        ("Heart Piece", "200", stock_heart(flags)),
        ("Hero's Tunic", "300", stock_tunic(flags)),
    ];
    for (i, (name, price, note)) in rows.iter().enumerate() {
        let y = 88.0 + i as f32 * 28.0;
        let locked = i == 3 && !has_flag(flags, save_flags::TUNIC_UNLOCKED);
        let col = if locked {
            "#606870"
        } else if i == shop.cursor {
            "#ffffff"
        } else {
            "#a0a8b8"
        };
        let mark = if i == shop.cursor { ">" } else { " " };
        d.text(&format!("{mark} {name}"), 56.0, y, col);
        d.text(&format!("{price} R"), 220.0, y, col);
        d.text(note, 280.0, y, "#808890");
    }
    d.text("J/E buy  Esc/L close", 56.0, 210.0, "#707880");
    let _ = text::text;
    let _ = SaveGame::default_spawn;
}

fn stock_bombs(flags: &[u16]) -> &'static str {
    if has_flag(flags, save_flags::SHOP_BOMBS_UNLOCKED) {
        "restock"
    } else {
        "unlock pouch"
    }
}

fn stock_bag(flags: &[u16]) -> &'static str {
    if has_flag(flags, save_flags::SHOP_BOMB_BAG) {
        "sold out"
    } else {
        "once"
    }
}

fn stock_heart(flags: &[u16]) -> &'static str {
    if has_flag(flags, save_flags::HEART_PIECE_4) {
        "sold out"
    } else {
        "once"
    }
}

fn stock_tunic(flags: &[u16]) -> &'static str {
    if !has_flag(flags, save_flags::TUNIC_UNLOCKED) {
        "After the shrine's trial."
    } else if has_flag(flags, save_flags::TUNIC_BOUGHT) {
        "sold out"
    } else {
        "cosmetic"
    }
}
