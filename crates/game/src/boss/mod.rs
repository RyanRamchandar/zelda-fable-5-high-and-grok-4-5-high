//! Granite Warden boss fight + arena lifecycle.

mod granite_warden;
mod warden_attacks;
mod warden_crystals;
pub(crate) mod pebble;

use content::audio::sfx::SfxId;
use content::maps::dungeon;
use content::maps::MapId;
use content::text::TextId;
use engine::input::InputState;
use engine::render::Draw;

use crate::fx::FxKind;
use crate::math::Vec2;
use crate::save_data::{has_flag, set_flag};
use crate::state::save_from_game;
use crate::world::entity::{EntityData, EntityId, EntityKind};
use crate::world::WorldEvent;
use crate::Game;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CutsceneKind {
    IntroAssemble,
    IntroNamePlate,
    IntroReturn,
    ShortIntro,
    Collapse,
}

#[derive(Clone, Debug)]
pub struct Cutscene {
    pub kind: CutsceneKind,
    pub t: u16,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BossPhase {
    Intro,
    Fight,
    Victory,
}

#[derive(Clone, Debug)]
pub struct BossState {
    pub warden: EntityId,
    pub crystals: [EntityId; 2],
    pub cutscene: Option<Cutscene>,
    pub phase: BossPhase,
    pub bar_visible: bool,
    pub rim_crumbled: bool,
    pub name_plate: u16,
    pub victory_step: u16,
    pub short_intro: bool,
}

pub fn clear(game: &mut Game) {
    if let Some(boss) = game.boss.take() {
        if game.world.get(boss.warden).is_some() {
            game.world.despawn(boss.warden);
        }
        for cid in boss.crystals {
            if game.world.get(cid).is_some() {
                game.world.despawn(cid);
            }
        }
    }
    // Despawn leftover pebbles / rocks in arena.
    let ids: Vec<_> = game
        .world
        .alive_ids()
        .into_iter()
        .filter(|&id| {
            matches!(
                game.world.get(id).map(|e| e.kind),
                Some(EntityKind::PebbleCrawler)
            )
        })
        .collect();
    for id in ids {
        game.world.despawn(id);
    }
}

pub fn on_enter_arena(game: &mut Game) {
    if game.current_map != MapId::Dungeon {
        return;
    }
    clear(game);
    // Seal the boss-door entrance (BossKey tiles aren't shutter-typed).
    use content::maps::{catalog, TileLayer};
    game.world
        .set_tile(TileLayer::Ground, 60, 7, catalog::D_SHUTTER);
    game.world
        .set_tile(TileLayer::Ground, 59, 7, catalog::D_SHUTTER);
    if let Some(rs) = game.rooms.as_mut() {
        if (dungeon::ROOM_ARENA as usize) < rs.shutter_closed.len() {
            rs.shutter_closed[dungeon::ROOM_ARENA as usize] = true;
        }
    }
    game.world
        .push_event(WorldEvent::Sfx(SfxId::ShutterSlam));

    let short = has_flag(&game.flags, content::flags::WARDEN_INTRO_SEEN);
    granite_warden::spawn_fight(game);
    if let Some(boss) = game.boss.as_mut() {
        boss.short_intro = short;
        boss.phase = BossPhase::Intro;
        boss.bar_visible = false;
        boss.cutscene = Some(Cutscene {
            kind: if short {
                CutsceneKind::ShortIntro
            } else {
                CutsceneKind::IntroAssemble
            },
            t: 0,
        });
    }
}

pub fn on_warden_damaged(world: &mut crate::world::World, id: EntityId) {
    granite_warden::on_damaged(world, id);
}

/// Returns true if world sim should pause (cutscene / victory hold).
pub fn update(game: &mut Game, input: &InputState) -> bool {
    if game.boss.is_none() {
        return false;
    }
    if game.current_map != MapId::Dungeon {
        clear(game);
        return false;
    }

    // Credits overlay owns the pause when active.
    if game.ui.credits.active {
        let skip = ui_credits_tick(game, input);
        if skip {
            finish_credits_return(game);
        }
        return true;
    }

    let phase = game.boss.as_ref().map(|b| b.phase);
    match phase {
        Some(BossPhase::Intro) => {
            tick_intro(game);
            true
        }
        Some(BossPhase::Fight) => {
            granite_warden::tick_fight(game);
            false
        }
        Some(BossPhase::Victory) => {
            tick_victory(game, input);
            true
        }
        None => false,
    }
}

fn tick_intro(game: &mut Game) {
    let Some(boss) = game.boss.as_mut() else {
        return;
    };
    let Some(cs) = boss.cutscene.as_mut() else {
        boss.phase = BossPhase::Fight;
        boss.bar_visible = true;
        return;
    };
    cs.t = cs.t.saturating_add(1);
    let t = cs.t;
    let kind = cs.kind;

    match kind {
        CutsceneKind::ShortIntro => {
            if t == 1 {
                game.world.push_event(WorldEvent::Sfx(SfxId::WardenRoar));
            }
            if t >= 30 {
                boss.cutscene = None;
                boss.phase = BossPhase::Fight;
                boss.bar_visible = true;
            }
        }
        CutsceneKind::IntroAssemble => {
            // Camera push toward warden.
            if let Some(w) = game.world.get(boss.warden) {
                let target = w.center();
                let cam = game.world.camera.pos;
                game.world.camera.pos = Vec2::new(
                    cam.x + (target.x - 240.0 - cam.x) * 0.08,
                    cam.y + (target.y - 135.0 - cam.y) * 0.08,
                );
            }
            if t == 1 {
                game.world.push_event(WorldEvent::Sfx(SfxId::WardenRoar));
            }
            if t == 20 || t == 40 || t == 60 {
                if let Some(w) = game.world.get(boss.warden) {
                    game.world.push_event(WorldEvent::FxRequest(FxKind::Impact {
                        pos: w.center(),
                    }));
                }
                game.world.camera.add_shake(1.5, 6);
            }
            if let Some(e) = game.world.get_mut(boss.warden) {
                e.anim.frame = ((t / 20) % 4).min(3);
            }
            if t >= 90 {
                cs.kind = CutsceneKind::IntroNamePlate;
                cs.t = 0;
                boss.name_plate = 90;
            }
        }
        CutsceneKind::IntroNamePlate => {
            boss.name_plate = boss.name_plate.saturating_sub(1);
            if t >= 90 {
                cs.kind = CutsceneKind::IntroReturn;
                cs.t = 0;
            }
        }
        CutsceneKind::IntroReturn => {
            if t >= 40 {
                set_flag(&mut game.flags, content::flags::WARDEN_INTRO_SEEN);
                boss.cutscene = None;
                boss.phase = BossPhase::Fight;
                boss.bar_visible = true;
                boss.name_plate = 0;
                let save = save_from_game(game);
                if let Some(json) = save.to_json() {
                    game.pending_save = Some(json);
                }
            }
        }
        CutsceneKind::Collapse => {
            if granite_warden::tick_collapse(game) {
                // still collapsing
            }
        }
    }
}

fn tick_victory(game: &mut Game, _input: &InputState) {
    // Collapse cutscene first.
    if game
        .boss
        .as_ref()
        .and_then(|b| b.cutscene.as_ref())
        .map(|c| c.kind == CutsceneKind::Collapse)
        .unwrap_or(false)
    {
        granite_warden::tick_collapse(game);
        return;
    }

    let step = game.boss.as_ref().map(|b| b.victory_step).unwrap_or(0);
    match step {
        1 => {
            // Heart container (skip if already claimed).
            if !has_flag(&game.flags, content::flags::WARDEN_HEART) {
                set_flag(&mut game.flags, content::flags::WARDEN_HEART);
                if let Some(p) = game.world.get_mut(game.world.player_id) {
                    if let EntityData::Player(pd) = &mut p.data {
                        pd.max_hearts = pd.max_hearts.saturating_add(2);
                        pd.hearts = pd.max_hearts;
                    }
                }
                sync_player_hearts(game);
                game.world.push_event(WorldEvent::FxRequest(FxKind::Toast {
                    text: "HEART CONTAINER",
                }));
                game.world
                    .push_event(WorldEvent::Sfx(SfxId::PickupHeart));
            }
            if let Some(b) = game.boss.as_mut() {
                b.victory_step = 2;
            }
            let save = save_from_game(game);
            if let Some(json) = save.to_json() {
                game.pending_save = Some(json);
            }
        }
        2 => {
            if !has_flag(&game.flags, content::flags::SHARD_OF_COURAGE) {
                set_flag(&mut game.flags, content::flags::SHARD_OF_COURAGE);
                set_flag(&mut game.flags, content::flags::WARDEN_DEFEATED);
                set_flag(&mut game.flags, content::flags::TUNIC_UNLOCKED);
                game.ui.dialog.open_text(TextId::ShardOfCourage);
                game.world.push_event(WorldEvent::Sfx(SfxId::GemGet));
            } else {
                // Re-fight: skip rewards.
                set_flag(&mut game.flags, content::flags::WARDEN_DEFEATED);
            }
            if let Some(b) = game.boss.as_mut() {
                b.victory_step = 3;
            }
            let save = save_from_game(game);
            if let Some(json) = save.to_json() {
                game.pending_save = Some(json);
            }
        }
        3 if !game.ui.dialog.open => {
            if let Some(b) = game.boss.as_mut() {
                b.victory_step = 4;
            }
            game.ui.credits.begin();
        }
        _ => {}
    }
}

fn sync_player_hearts(game: &mut Game) {
    if let Some(p) = game.world.get_mut(game.world.player_id) {
        if let EntityData::Player(pd) = &p.data {
            let max = pd.max_hearts;
            let hp = pd.hearts;
            if let Some(h) = p.health.as_mut() {
                h.max = max;
                h.hp = hp;
            }
        }
    }
}

fn ui_credits_tick(game: &mut Game, input: &InputState) -> bool {
    game.ui.credits.update(input)
}

fn finish_credits_return(game: &mut Game) {
    game.ui.credits.active = false;
    clear(game);
    // Return to village fountain.
    crate::state::begin_transition(game, MapId::Overworld, village_fountain_entry());
    game.world.push_event(WorldEvent::FxRequest(FxKind::Toast {
        text: "ACT 1 COMPLETE",
    }));
    let save = save_from_game(game);
    if let Some(json) = save.to_json() {
        game.pending_save = Some(json);
    }
}

fn village_fountain_entry() -> u8 {
    // Prefer an existing village entry; fall back to 0 (south gate).
    // Checkpoint-style entry near fountain if authored as id 1 in OW.
    1
}

pub fn render_overlay(d: &mut Draw, game: &Game) {
    let Some(boss) = game.boss.as_ref() else {
        return;
    };
    if boss.name_plate > 0 {
        // Letterbox + name.
        d.rect(0.0, 0.0, 480.0, 28.0, "#000000");
        d.rect(0.0, 242.0, 480.0, 28.0, "#000000");
        d.text("GRANITE WARDEN", 168.0, 120.0, "#e8e0c8");
    }
    if boss.bar_visible {
        if let Some(e) = game.world.get(boss.warden) {
            if let EntityData::GraniteWarden(wd) = &e.data {
                crate::ui::boss_bar::draw(d, wd.hp, wd.max_hp);
            }
        }
    }
    // Crystal prime rings.
    for &cid in &boss.crystals {
        if let Some(e) = game.world.get(cid) {
            if let EntityData::WindCrystal(cd) = &e.data {
                if cd.primed > 0 {
                    let r = 6.0 + (cd.primed as f32 / 300.0) * 6.0;
                    let c = e.center();
                    d.circle(c.x, c.y, r, "rgba(120,200,255,0.45)");
                }
            }
        }
    }
}

/// On player death during boss — clear fight; respawn handles map rebuild.
pub fn on_player_death(game: &mut Game) {
    clear(game);
}
