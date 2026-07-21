//! Interact verb: signs, NPCs, chests, gems, shrine seal, landmark secrets.

use content::audio::sfx::SfxId;
use content::maps::{Loot, MapId, TileLayer, TILE_PX};
use content::text::{npc_lines, TextId};
use engine::input::{InputState, BUTTON_INTERACT};

use crate::fx::FxKind;
use crate::items::pickups;
use crate::math::{Dir4, Vec2};
use crate::save_data::{has_flag, maybe_apply_heart_container, set_flag, save_flags};
use crate::world::entity::{EntityData, EntityKind};
use crate::world::{World, WorldEvent};
use crate::Game;

const INTERACT_PX: f32 = 28.0;

pub fn update(game: &mut Game, input: &InputState) -> Option<String> {
    // Face-toward idle anim for NPCs.
    face_npcs(&mut game.world);

    if game.ui.dialog.open {
        game.ui.dialog.update(input, &mut game.world);
        return None;
    }

    if !input.buttons[BUTTON_INTERACT].pressed {
        tick_chest_anim(&mut game.world);
        return None;
    }

    let player = game.world.get(game.world.player_id)?;
    let pcenter = player.center();
    let facing = player.facing;

    // Prefer entity interactables in front.
    if let Some(id) = nearest_interactable(&game.world, pcenter, facing) {
        return interact_entity(game, id);
    }

    // Shrine sealed door tile interact.
    if try_shrine_seal(game, pcenter) {
        return game.pending_save.clone();
    }

    // Landmark stones / braziers / hollow walls.
    try_landmark(game, pcenter);
    None
}

fn face_npcs(world: &mut World) {
    let Some(p) = world.get(world.player_id) else {
        return;
    };
    let pc = p.center();
    let ids = world.alive_ids();
    for id in ids {
        let Some(e) = world.get_mut(id) else {
            continue;
        };
        if e.kind != EntityKind::Npc {
            continue;
        }
        let c = e.center();
        if c.sub(pc).len() < 48.0 {
            let d = pc.sub(c);
            e.facing = Dir4::from_vec(d, e.facing);
        }
        if e.anim.timer % 20 == 0 {
            e.anim.frame = (e.anim.frame + 1) % 2;
        }
    }
}

fn tick_chest_anim(world: &mut World) {
    let ids = world.alive_ids();
    for id in ids {
        let Some(e) = world.get_mut(id) else {
            continue;
        };
        if let EntityData::Chest(cd) = &mut e.data {
            if cd.open_anim > 0 {
                cd.open_anim = cd.open_anim.saturating_sub(1);
                e.anim.frame = 1;
            }
        }
    }
}

fn nearest_interactable(world: &World, pcenter: Vec2, facing: Dir4) -> Option<crate::world::EntityId> {
    let fwd = facing.unit();
    let mut best: Option<(f32, crate::world::EntityId)> = None;
    for id in world.alive_ids() {
        let Some(e) = world.get(id) else {
            continue;
        };
        match e.kind {
            EntityKind::Sign | EntityKind::Npc | EntityKind::Chest | EntityKind::Gem => {}
            _ => continue,
        }
        if let EntityData::Gem(g) = &e.data {
            if g.taken {
                continue;
            }
        }
        if let EntityData::Chest(c) = &e.data {
            if c.open {
                continue;
            }
        }
        let c = e.center();
        let to = c.sub(pcenter);
        let dist = to.len();
        if dist > INTERACT_PX {
            continue;
        }
        // Prefer targets roughly in front, but allow nearby sides.
        let align = to.normalize_or_zero().dot(fwd);
        if align < -0.55 && dist > 14.0 {
            continue;
        }
        let score = dist - align * 4.0;
        if best.map(|(b, _)| score < b).unwrap_or(true) {
            best = Some((score, id));
        }
    }
    best.map(|(_, id)| id)
}

fn interact_entity(game: &mut Game, id: crate::world::EntityId) -> Option<String> {
    let kind = game.world.get(id).map(|e| e.kind)?;
    match kind {
        EntityKind::Sign => {
            if let Some(EntityData::Sign(s)) = game.world.get(id).map(|e| e.data.clone()) {
                game.ui.dialog.open_text(s.text);
            }
            None
        }
        EntityKind::Npc => {
            if let Some(EntityData::Npc(n)) = game.world.get(id).map(|e| e.data.clone()) {
                if n.npc == content::text::NpcId::Shopkeeper {
                    crate::ui::shop::open(game);
                    return None;
                }
                let text = if n.npc == content::text::NpcId::Elder
                    && has_flag(&game.flags, save_flags::SHARD_OF_COURAGE)
                {
                    TextId::ElderVictory
                } else if n.npc == content::text::NpcId::Elder
                    && has_flag(&game.flags, save_flags::QUEST_STARTED)
                {
                    TextId::ElderReminder
                } else {
                    npc_lines(n.npc)
                };
                game.ui.dialog.open_text(text);
                if n.npc == content::text::NpcId::Elder {
                    set_flag(&mut game.flags, save_flags::QUEST_STARTED);
                    game.ui.minimap.refresh_objective(game.gems, &game.flags);
                }
            }
            None
        }
        EntityKind::Chest => open_chest(game, id),
        EntityKind::Gem => take_gem_entity(game, id),
        _ => None,
    }
}

fn open_chest(game: &mut Game, id: crate::world::EntityId) -> Option<String> {
    let (flag, loot, pos) = {
        let e = game.world.get(id)?;
        let EntityData::Chest(cd) = &e.data else {
            return None;
        };
        if cd.open {
            return None;
        }
        // Power gem chest locked until guard group cleared.
        if flag_is_power_chest(cd.flag)
            && !has_flag(&game.flags, save_flags::GROUP_CAMP_GUARD)
            && !crate::world::spawner::group_cleared(&game.spawner, 43)
        {
            game.ui.dialog.open_text(TextId::PowerChestLocked);
            return None;
        }
        (cd.flag, cd.loot, e.pos)
    };

    if let Some(e) = game.world.get_mut(id) {
        if let EntityData::Chest(cd) = &mut e.data {
            cd.open = true;
            cd.open_anim = 12;
            e.anim.frame = 1;
        }
    }
    set_flag(&mut game.flags, flag);
    game.world
        .push_event(WorldEvent::Sfx(SfxId::ChestOpen));

    match loot {
        Loot::Rupees(n) => {
            pickups::spawn_rupees(&mut game.world, pos, n);
            game.world.push_event(WorldEvent::FxRequest(FxKind::Toast {
                text: "CHEST!",
            }));
        }
        Loot::HeartPiece => {
            grant_heart_piece(game, flag);
        }
        Loot::Gem(gid) => {
            grant_gem(game, gid);
        }
        Loot::Boomerang => {
            set_flag(&mut game.flags, save_flags::ITEM_BOOMERANG);
            if let Some(p) = game.world.get_mut(game.world.player_id) {
                if let EntityData::Player(pd) = &mut p.data {
                    pd.selected_item = 2;
                }
            }
            game.ui.dialog.open_text(TextId::BoomerangGet);
            game.world.push_event(WorldEvent::FxRequest(FxKind::Toast {
                text: "GALE BOOMERANG",
            }));
            game.world.push_event(WorldEvent::Sfx(SfxId::GemGet));
            game.world.push_event(WorldEvent::FxRequest(FxKind::Impact { pos }));
        }
        Loot::SmallKey => {
            if flag == save_flags::DCHEST_KEY1 {
                set_flag(&mut game.flags, save_flags::DKEY_SMALL_1);
            } else {
                set_flag(&mut game.flags, save_flags::DKEY_SMALL_2);
            }
            game.world.push_event(WorldEvent::FxRequest(FxKind::Toast {
                text: "SMALL KEY",
            }));
            game.world.push_event(WorldEvent::Sfx(SfxId::KeyGet));
        }
        Loot::BossKey => {
            set_flag(&mut game.flags, save_flags::DKEY_BOSS);
            game.world.push_event(WorldEvent::FxRequest(FxKind::Toast {
                text: "BOSS KEY",
            }));
            game.world.push_event(WorldEvent::Sfx(SfxId::KeyGet));
        }
    }
    save_from(game)
}

fn flag_is_power_chest(flag: u16) -> bool {
    flag == save_flags::CHEST_POWER_GEM
}

fn take_gem_entity(game: &mut Game, id: crate::world::EntityId) -> Option<String> {
    let (gid, sealed) = {
        let e = game.world.get(id)?;
        let EntityData::Gem(g) = &e.data else {
            return None;
        };
        if g.taken {
            return None;
        }
        (g.id, g.sealed)
    };
    if game.current_map == content::maps::MapId::Overworld
        && gid == 0
        && (sealed || !has_flag(&game.flags, save_flags::PUZZLE_CHIMES_DONE))
    {
        game.ui.dialog.open_text(TextId::CourageGemSealed);
        return None;
    }
    if let Some(e) = game.world.get_mut(id) {
        if let EntityData::Gem(g) = &mut e.data {
            g.taken = true;
        }
    }
    grant_gem(game, gid);
    save_from(game)
}

fn grant_gem(game: &mut Game, gid: u8) {
    let bit = 1u8 << gid.min(2);
    game.gems |= bit;
    let (flag, toast, hold) = match gid {
        0 => (save_flags::GEM_COURAGE, "COURAGE GEM", TextId::CourageGemHold),
        1 => (save_flags::GEM_POWER, "POWER GEM", TextId::PowerGemHold),
        _ => (save_flags::GEM_WISDOM, "WISDOM GEM", TextId::WisdomGemHold),
    };
    set_flag(&mut game.flags, flag);
    // Checkpoint at gem sites (GAME_DESIGN §8).
    let cp = match gid {
        0 => 2,
        1 => 3,
        _ => 4,
    };
    game.world.checkpoint = cp;
    game.world
        .push_event(WorldEvent::Sfx(SfxId::GemGet));
    game.world
        .push_event(WorldEvent::FxRequest(FxKind::Toast { text: toast }));
    game.ui.dialog.open_text(hold);
    game.ui.minimap.refresh_objective(game.gems, &game.flags);
    game.world.camera.add_shake(2.5, 10);
}

fn grant_heart_piece(game: &mut Game, chest_flag: u16) {
    let piece = match chest_flag {
        f if f == save_flags::CHEST_CLIFFS_HEART => save_flags::HEART_PIECE_1,
        f if f == save_flags::CHEST_RUINS_CELLAR => save_flags::HEART_PIECE_2,
        f if f == save_flags::CHEST_GROVE_HEART => save_flags::HEART_PIECE_3,
        _ => save_flags::HEART_PIECE_3,
    };
    set_flag(&mut game.flags, piece);
    if piece == save_flags::HEART_PIECE_1 {
        set_flag(&mut game.flags, save_flags::SECRET_CLIFFS_CAVE);
    }
    game.world.push_event(WorldEvent::FxRequest(FxKind::Toast {
        text: "HEART PIECE",
    }));
    game.world
        .push_event(WorldEvent::Sfx(SfxId::SecretChime));
    let pid = game.world.player_id;
    if let Some(p) = game.world.get_mut(pid) {
        if let EntityData::Player(pd) = &mut p.data {
            if maybe_apply_heart_container(&mut game.flags, &mut pd.max_hearts) {
                pd.hearts = pd.max_hearts;
                if let Some(h) = p.health.as_mut() {
                    h.max = pd.max_hearts;
                    h.hp = pd.max_hearts;
                }
                game.world.push_event(WorldEvent::FxRequest(FxKind::Toast {
                    text: "MAX HEART UP!",
                }));
            }
        }
    }
    game.ui.minimap.mark_secret(piece);
}

fn try_shrine_seal(game: &mut Game, pcenter: Vec2) -> bool {
    if game.current_map != MapId::Overworld {
        return false;
    }
    if has_flag(&game.flags, save_flags::DOOR_SHRINE_OPEN) {
        return false;
    }
    // Sealed door tile around (120, 10) from stamp at (117,8) center D.
    let tx = (pcenter.x / TILE_PX).floor() as i32;
    let ty = (pcenter.y / TILE_PX).floor() as i32;
    let near = (tx - 120).abs() <= 2 && (ty - 10).abs() <= 2;
    if !near {
        return false;
    }
    let count = game.gems.count_ones();
    if count < 3 {
        let tid = match count {
            0 => TextId::SealHolds0,
            1 => TextId::SealHolds1,
            _ => TextId::SealHolds2,
        };
        game.ui.dialog.open_text(tid);
        return true;
    }
    // Open door tiles.
    set_flag(&mut game.flags, save_flags::DOOR_SHRINE_OPEN);
    game.world
        .set_tile(TileLayer::Ground, 120, 10, content::maps::catalog::T_CAVE_MOUTH);
    game.world.set_tile(
        TileLayer::Ground,
        119,
        10,
        content::maps::catalog::T_PATH,
    );
    game.world.set_tile(
        TileLayer::Ground,
        121,
        10,
        content::maps::catalog::T_PATH,
    );
    // Antechamber door → ShrineLobby
    game.world.map.triggers.push(content::maps::TriggerDef {
        tx: 120,
        ty: 10,
        w: 1,
        h: 1,
        kind: content::maps::TriggerKind::Door {
            target: MapId::ShrineLobby,
            entry: 0,
        },
    });
    game.world.camera.add_shake(3.0, 14);
    game.world.push_event(WorldEvent::Sfx(SfxId::SealOpen));
    game.world.push_event(WorldEvent::FxRequest(FxKind::Toast {
        text: "SEAL OPEN!",
    }));
    game.ui.dialog.open_text(TextId::SealOpens);
    game.ui.minimap.refresh_objective(game.gems, &game.flags);
    game.pending_save = save_from(game);
    true
}

fn try_landmark(game: &mut Game, pcenter: Vec2) {
    let tx = (pcenter.x / TILE_PX).floor() as u32;
    let ty = (pcenter.y / TILE_PX).floor() as u32;

    // Summit vista stone ~ (100, 30) cliffs
    if in_tile(tx, ty, 98, 28, 104, 32)
        && set_flag(&mut game.flags, save_flags::SECRET_SUMMIT_VISTA)
    {
        discover_secret(game, TextId::SummitVista, 30);
        pickups::spawn_rupees(&mut game.world, pcenter, 30);
        return;
    }

    // Hollow bomb wall grove ~ (30, 185) — hint only while closed.
    if in_tile(tx, ty, 28, 183, 32, 187)
        && !has_flag(&game.flags, save_flags::WALL_GROVE_OPEN)
    {
        game.ui.dialog.open_text(TextId::HollowWall);
        return;
    }

    // Twin braziers shrine terrace (112,14) + (128,14)
    if in_tile(tx, ty, 110, 12, 114, 16) || in_tile(tx, ty, 126, 12, 130, 16) {
        game.ui.minimap.note_brazier(game.world.tick);
        if game.ui.minimap.braziers_linked(game.world.tick)
            && set_flag(&mut game.flags, save_flags::SECRET_SHRINE_BRAZIERS)
        {
            discover_secret(game, TextId::TwinFlames, 50);
            // Spawn reward chest if not present — grant rupees directly.
            pickups::spawn_rupees(&mut game.world, pcenter, 50);
            set_flag(&mut game.flags, save_flags::CHEST_SHRINE_BRAZIERS);
        }
    }
}

fn discover_secret(game: &mut Game, dialog: TextId, _rupees: u32) {
    game.world
        .push_event(WorldEvent::Sfx(SfxId::SecretChime));
    game.world.push_event(WorldEvent::FxRequest(FxKind::Toast {
        text: "SECRET!",
    }));
    game.ui.dialog.open_text(dialog);
    game.ui.minimap.mark_discovered_secret();
}

fn in_tile(tx: u32, ty: u32, x0: u32, y0: u32, x1: u32, y1: u32) -> bool {
    tx >= x0 && tx <= x1 && ty >= y0 && ty <= y1
}

fn save_from(game: &Game) -> Option<String> {
    crate::state::save_from_game(game).to_json()
}

/// Prompt marker: true if an interactable is in range.
pub fn prompt_target(world: &World) -> Option<Vec2> {
    let p = world.get(world.player_id)?;
    let c = p.center();
    nearest_interactable(world, c, p.facing).and_then(|id| {
        world.get(id).map(|e| Vec2::new(e.pos.x + 6.0, e.pos.y - 10.0))
    })
}

