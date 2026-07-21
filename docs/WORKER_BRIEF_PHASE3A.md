# WORKER_BRIEF_PHASE3A.md — Triforce Shrine dungeon: rooms, Gale Boomerang, curriculum, keys

You are a Grok worker. This brief is self-contained; follow it exactly. Conflict
order: `DECISIONS.md` → `ARCHITECTURE.md` → this brief → `GAME_DESIGN.md`.
You run autonomously — do not ask the human anything except for credentials.

## 0. Mission

Phase 2C finished the overworld: three real gem gates, bombs + shop economy,
seven enemy families, and the shrine seal that opens into a stub
`MapId::ShrineLobby`. You build the **Act 1 dungeon interior** (GAME_DESIGN §5)
up to — but not including — the Sanctum Core miniboss and the Granite Warden:

- **Room system**: one contiguous dungeon `MapDef` with camera-locked rooms and
  Zelda-style **slide transitions** between rooms (no fades indoors).
- **Gale Boomerang**: the Act 1 tool. Throw/return flight, enemy **stun**, a
  style verb, B-item slot 2 on the existing `BUTTON_CYCLE`/tap-Item seam, and —
  critically — a new **source of the same tool-agnostic puzzle hits** 2C built,
  so overworld chimes/cranks retro-enable at range for free.
- **Curriculum**: Hall of Trials (combat/plate gauntlet → boomerang chest with
  an item-get beat), Hall of Currents (wind-crystal blue/amber gates,
  around-the-corner multi-target throws, carry-a-flame), and the two **ordered
  seal rooms** (route one throw through 3 crystals in rune-telegraphed order).
- **Keys/doors**: 2 small keys + boss key routes (boss key chest itself is
  placed by you; the fight it opens is Phase 3B), locked doors, combat
  shutters, and a **discovered-room dungeon minimap with strict exit ⇄ map
  reciprocity**.

Phase 3B (separate worker, after you) adds the Ironshell duo miniboss, the
Granite Warden, and the victory flow. You leave the Sanctum Core and Guardian
Arena rooms **built but dormant** behind clearly marked stubs.

**Gameplay feel is priority #1.** The boomerang must feel snappy and musical
(distinct throw/flight/catch cues), room slides must be crisp, and every puzzle
readable without text.

## 1. HARD DEPENDENCY — code reality (verify each before building)

HEAD is `7fa19b3` (phase2c complete). Where this brief differs from code + the
2C-A/2C-B completion notes in `docs/WORKER_NOTES.md`, **code + notes win**; log
drift in WORKER_NOTES and adapt.

Facts this brief is written against:

1. **Puzzle hit layer is tool-agnostic and frozen.** `game/src/puzzle/mod.rs`:
   `process_hits` consumes `world.active_attacks` (cloned, non-destructively —
   `puzzle::update` runs between `integrate_non_player` and
   `combat::resolve_hits` in `lib.rs`) plus player-owned projectiles
   (`SwordBeam`/`OctorokRock` with `from_player && !hit`). Tile dedupe is
   `mark_tile_hit(puzzle, swing_id, tx, ty)`. The boomerang becomes a **third
   projectile arm** in this scan — one that does NOT set `hit`/despawn on tile
   contact, so a single throw can ring multiple chimes/crystals (dedupe per
   throw via its throw id as `swing_id`). Do not restructure the frozen
   2C-A API (`PuzzleState::for_map`, `paint_closed`, `restore`, `update`,
   `try_open_bomb_wall`, `bomb_break_barricades`, `content::puzzles` shapes).
2. **B-item seam**: `BUTTON_CYCLE = 6` (Q / pad LB+RB), `BUTTON_COUNT = 7`;
   tap-Item = release within `ITEM_TAP_MAX_TICKS = 8` while shield state
   allows (`player/mod.rs update_shield_and_dash_intent`); hold-Item still
   shields. `pd.selected_item`: 0 = none, 1 = bombs, **2 = boomerang (reserved
   for you)**. `update_item_cycle` currently just sets `selected_item = 1` if
   `bomb_cap > 0` — you rewrite it to cycle through unlocked items. Touch item
   parity remains Phase 4 (log, don't build).
3. **Stun hook**: `pub fn skeleton::stagger(world, id)` exists exactly for the
   boomerang (`enemies/skeleton.rs`). `DamagedPlayer` carries
   `source: Option<EntityId>`. Other families have no stun hook — you add a
   small generic one (§3 M2).
4. **Transitions**: `GameMode::{Play, Transition}` with 16-tick fade via
   `state::begin_transition` / `switch_map` — this is for **map** switches
   (lobby↔dungeon). Room-to-room movement inside the dungeon must NOT use it;
   rooms live in one `MapDef` and the camera slides (§3 M1).
5. **Camera** (`world/camera.rs`, 96 lines): dead-zone + lookahead + shake,
   `update(map_w, map_h, ...)` clamps to full map bounds, `snap_to`. You add a
   rect-bounds clamp (additive method) so a camera can be locked to a room.
6. **MapId codec** (`content/src/maps/mod.rs`): Overworld 0, Arena 1, Shop 2,
   ShrineLobby 3, House 10–15, Cave 20–22. **4 is free — `MapId::Dungeon`
   takes it.** `maps::build(id)` is the single loader.
7. **Interiors pattern**: `interiors::shrine_lobby()` is a 16×12 stub room with
   a `TextId::ShrineLobby` sign ("Phase 3") and exit → Overworld entry 6. You
   replace the sign with a real door to the dungeon.
8. **Save/persist**: `SaveGame` v2 with `#[serde(default)]` additive fields
   only (precedent: `fog`, `bombs`). `PlayerPersist` (`state.rs`) must carry
   anything player-attached across `switch_map` or it drops at every door.
   **Prefer flags over new save fields**: keys, seals, room discovery are all
   representable in `flags: Vec<u16>` (registry `content::flags`).
   `Game::new` currently **forces boot map to Overworld unless Arena** — you
   extend it so a save inside `MapId::Dungeon` boots the dungeon at its
   checkpoint entry (death/reload mid-dungeon must not eject to the field).
9. **Checkpoints**: `TriggerKind::Checkpoint { id }` fires once per id change;
   gems set 2/3/4; shrine terrace is 6. `check_player_death` does
   `switch_map(current_map, checkpoint)` — dungeon checkpoints therefore need
   matching dungeon `EntryPoint` ids. Use **7 (Vestibule)** and **8 (Sanctum
   antechamber)**. Exiting to the overworld self-heals via the existing shrine
   checkpoint trigger.
10. **Spawner**: groups + `locked_groups` + `GroupCleared(u16)` events; the
    camp chain shows the unlock/force-activate pattern (`lib.rs
    drain_events`). Dungeon combat rooms reuse **spawner groups + shutter
    tiles**, not the Arena `WaveDirector`.
11. **File caps (~600)**: `world/entity.rs` **598**, `lib.rs` **591**,
    `puzzle/mod.rs` 552, `draw_world.rs` 509, `minimap.rs` 458, `interact.rs`
    427. You MUST extract before adding (§3 M1 headroom). Never touch
    `minimap.rs` beyond its existing public surface — the dungeon map is a new
    file.
12. **Catalog**: tile ids used through 266; `tile_info` returns sprite key /
    frames / anim_rate / collision flags; ground paints refresh collision.
    You take the **280–309 dungeon block** (267–279 stay reserved).
13. **Art**: indexed grids in `content/src/art/*`, registered in
    `all_bakes()`; startup assertion catches missing keys. Palette has ~48
    colors + swap rows; dungeon family is "cool stone + teal water accents"
    (GAME_DESIGN §5) — add palette entries only if needed, log them.
14. **Text**: all copy in `content/src/text.rs` (`TextId` + pages of ≤3 lines
    × ~38 chars). No string literals in game code (HUD toasts as
    `&'static str` consts are fine).
15. **Chests**: `Loot::{Rupees, HeartPiece, Gem}` — you append
    `Loot::Boomerang`, `Loot::SmallKey`, `Loot::BossKey` (exhaustive matches:
    `interact::open_chest` + anything else the compiler flags).
16. **Style**: `StyleVerb` enum + `verb_cooldowns: [u16; 6]` in `PlayerData`
    (`combat/style.rs`) — boomerang stun is a new verb (GAME_DESIGN §3);
    extend the array and the verb list.
17. Toolchain: rustup `~/.cargo/bin` ahead of Homebrew rustc;
    `env -u NO_COLOR trunk build --release`; kill smoke-test servers/browsers
    after every run.

## 2. Locked constraints

- Stack per DECISIONS.md; no new crates; `content` depends on nothing; `game`
  stays web-sys-free; art = handcrafted indexed grids only.
- **No engine changes** except: `engine::chunks` untouched, `engine::input`
  untouched (2C already added the cycle button). If you believe you need one,
  WORKER_NOTES first + nearest non-blocking interpretation.
- Ownership (ARCHITECTURE §11): dungeon room/camera logic in a new
  `game::rooms`; boomerang in `game::items::boomerang`; dungeon puzzle
  runtime in `game::puzzle::dungeon` (submodule — the frozen overworld API
  stays as-is); dungeon map data in `content::maps::dungeon`; dungeon puzzle
  site data in `content::puzzles_dungeon`.
- **Do not implement**: the Ironshell miniboss, the Granite Warden, boss-bar
  UI, credits, tunic effect (`TUNIC_UNLOCKED = 98` stays reserved). Sanctum
  Core and Guardian Arena rooms exist as dressed-but-dormant shells with a
  blocking sign each (`TextId` stubs) so 3B swaps fights in without map
  surgery.
- No file >~600 lines. Commits on `main`, prefixed `phase3a:`, small and
  frequent. No push, no deploy. Every commit: wasm32 `cargo check` +
  `clippy -- -D warnings` clean; `trunk build --release` at milestones.
- Tuning ±30% with a WORKER_NOTES log. Puzzle states resettable: unsolved
  rooms rebuild fresh on reload; solved ones restore open via flags. **No
  soft-locks**: any room you can enter you can leave or reset (leaving and
  re-entering a room resets its unsolved puzzle state — this is the reset
  story, mirror the 2C reload-reset rule at room granularity where cheap,
  map granularity where not; log the choice per puzzle).

## 3. Milestones (commit at least once each)

### M1 — Headroom, dungeon map seam, room slides

**Headroom first (mandatory, before any feature code):**
- Split `world/entity.rs` (598): move the per-family enemy data structs +
  enums (`SlimeData` … `TorchFlameData`) into a new `world/entity_data.rs`
  re-exported from `entity`. Target entity.rs ≤ 450 before you add
  `Boomerang`/key data.
- Extract `Game::drain_events` + the `WorldEvent` match from `lib.rs` (591)
  into a new `game/src/events.rs` (`pub(crate) fn drain(game, input) -> Vec<GameEvent>`).
  Target lib.rs ≤ 450.

**`MapId::Dungeon`** (codec value 4) + `content/src/maps/dungeon.rs` (split a
`dungeon_rooms.rs` helper if you near 600): ONE contiguous `MapDef` (suggest
~150×80 tiles, room cells on a grid — see M3 layout) built with the existing
painters. `maps::build(MapId::Dungeon)` arm. `ShrineLobby` gets a north door
tile + `TriggerDef` → `Dungeon` entry 0 (replace the Phase-3 stub sign);
Dungeon entry 0 sits in the Vestibule with a return door → `ShrineLobby`
entry 1 (add that entry to the lobby).

**Room system** — new `game/src/rooms.rs`:

- `content::maps::dungeon` exports `RoomDef { id: u8, rect: (u32, u32, u32, u32),
  exits: [Option<ExitDef>; 4] }` (N/E/S/W; `ExitDef { to_room: u8, door: DoorKind }`,
  `DoorKind::{Open, SmallKey, BossKey, SealWest, SealEast, Shutter}`) + a
  `rooms()` table. Room rects must tile the map exactly; a room may span
  multiple grid cells (halls, arena). **Debug assert at map load**
  (`game::debug`): every exit has a reciprocal exit in the target room, and
  every door tile in the map lies on a declared exit — this is acceptance
  criterion 4's foundation.
- `RoomsState` on `Game` (built on entering `MapId::Dungeon`, `None`
  elsewhere): current room id, `slide: Option<Slide { from_rect, to_rect,
  t }>`.
- **Camera lock**: add an additive `Camera::set_bounds(min: Vec2, max: Vec2)`
  (or an `Option<(Vec2, Vec2)>` field consulted in `update`) clamping the
  camera to the current room's pixel rect; rooms smaller than 480×270 center.
  Overworld path unchanged.
- **Slide transition**: player's feet crossing onto a declared exit's door
  tile (and the door is open) → start a ~24-tick slide: world sim pauses
  (early-return in `Game::update` like dialog/shop), camera lerps (smoothstep)
  from the old room clamp to the new, player is nudged ~20 px through the
  doorway during the slide, `SfxId::RoomSlide` (soft whoosh). On finish:
  current room = target, mark room discovered (flag), spawner proceeds
  normally (distance activation works because rooms are spatially separate;
  verify activation radius 480 px doesn't wake neighbors through walls — if
  it does, gate hostile activation by `current_room` in a small additive
  check in `rooms.rs`, not by rewriting the spawner).
- **Doors as tiles** (2C precedent — no entity solidity): `D_DOOR_OPEN`
  (walk), `D_DOOR_LOCKED` (SOLID), `D_DOOR_BOSS` (SOLID), `D_SHUTTER`
  (SOLID), plus seal doors (M4). Walking into a locked door with a key in
  inventory consumes it: swap tiles to open, set the per-door opened flag,
  `SfxId::DoorUnlock` + clunk shake; without a key, a dialog ("Locked. A key
  is needed.") — implement in `rooms.rs` or `interact.rs`, whichever stays
  under cap; door state restores from flags on load via
  `puzzle::dungeon::restore`.

**New catalog tiles (280–309)** + art (`content/src/art/tiles_dungeon.rs`,
`props_dungeon.rs`; register in `all_bakes()`), cool-stone/teal family:

| ids | consts | notes |
|---|---|---|
| 280–285 | `D_FLOOR_A/B`, `D_FLOOR_RUNE`, `D_WALL`, `D_WALL_TOP`, `D_PIT` | pit = SOLID (visual depth, no fall system) |
| 286–288 | `D_WATER`, `D_WATER_EDGE`, `D_STAIRS` | teal accent water (WATER flags), 2-frame shimmer ok |
| 289–293 | `D_DOOR_OPEN`, `D_DOOR_LOCKED`, `D_DOOR_BOSS`, `D_SHUTTER`, `D_LIFT` | boss door visually ornate; lift = walk, 2-frame glow |
| 294–299 | `D_CRYSTAL_BLUE`, `D_CRYSTAL_AMBER`, `D_GATE_BLUE_UP/DOWN`, `D_GATE_AMBER_UP/DOWN` | crystals SOLID; `*_UP` SOLID, `*_DOWN` walk |
| 300–304 | `D_TORCH_LIT`, `D_TORCH_UNLIT`, `D_BRAZIER_ETERNAL`, `D_RUNE_1..` (1–3 as one strip or three ids) | torches SOLID; runes walk |
| 305–309 | spares (seal door tiles live here if not reusing gates) | document each |

**New flags** (`content/src/flags.rs`, 100–139 dungeon block — document):
`ITEM_BOOMERANG = 100`, `DKEY_SMALL_1 = 101`, `DKEY_SMALL_2 = 102`,
`DKEY_BOSS = 103`, `DDOOR_WING = 104`, `DDOOR_INNER = 105`, `DDOOR_BOSS_USED
= 106` (3B opens it), `SEAL_WEST = 107`, `SEAL_EAST = 108`, `DCHEST_* =
110–116` (map chests incl. one dungeon heart-piece-quarter is NOT placed —
heart pieces are complete at 4; use rupee/key loot), `DROOM_0.. = 120+`
(one per room, discovery), group ids `GRP_DNG_TRIALS_1/2/3 = 90–92`,
`GRP_DNG_CURRENTS = 93`, `GRP_DNG_SANCTUM = 94` (3B's miniboss group —
reserve, don't populate hostiles).

Key inventory = **derived from flags**: small keys held =
(`DKEY_SMALL_*` set) − (locked doors whose opened-flag is set). No new
SaveGame fields for keys. HUD (M5) shows the count while in the dungeon.

**New SFX** (`SfxId` + specs): `RoomSlide`, `DoorUnlock`, `ShutterSlam`,
`BoomerangThrow`, `BoomerangCatch`, `CrystalBlue`, `CrystalAmber`,
`RuneGood`, `RuneBad`, `SealBreak`, `TorchLight`, `TorchSnuff`, `KeyGet`.

DoD: a 2-room test slice of the dungeon map is walkable from the lobby; slides
work both directions; locked door consumes a debug-granted key; reciprocity
assert passes; F3 Arena + overworld untouched.

### M2 — Gale Boomerang (the tool)

`game/src/items/boomerang.rs` (new), art in a new
`content/src/art/items.rs` (16×16, 4-frame spin strip + a held/chest sparkle
frame), `EntityKind::Boomerang` + `BoomerangData` in `entity_data.rs`:

- **Throw**: tap-Item with `selected_item == 2` and no boomerang already in
  flight (one at a time). Generalize the tap dispatch in
  `player::update_shield_and_dash_intent`: `selected_item == 1` → bombs (as
  today), `== 2` → `boomerang::try_throw`. Throw direction = 8-way input
  vector if held, else facing. No energy cost; the one-in-flight rule is the
  limiter. `SfxId::BoomerangThrow`.
- **Flight**: outbound straight at ~3.4 px/tick for up to ~7 tiles (112 px)
  or until it hits SOLID collision or max range → **return phase**: homes to
  the player's live center at ~4.2 px/tick (steering, so it bends around
  corners — this is what makes "multi-target throws around corners" work).
  Catch when within 10 px of the player: despawn + `SfxId::BoomerangCatch`
  (+ tiny hitstop-free flash). If somehow lost (player dies/room slides), it
  despawns quietly. Spin anim 4 frames @ 3 ticks.
- **Data**: `BoomerangData { dir: Vec2, phase: Out/Return, traveled: f32,
  throw_id: u32 }` — `throw_id` from the player's `swing_id` counter
  (increment on throw, same as bombs do).
- **Hits — enemies**: register an `ActiveAttack`-equivalent each tick? No —
  follow the projectile pattern: in `combat::resolve_hits`' projectile scan,
  add a Boomerang arm: overlap vs `ENEMY_BODY`, dedupe via
  `world.already_hit(throw_id, target.index)`, push `AttackHit` with a new
  `AttackKind::Boomerang` (damage 0.5, knockback 1.0) — the boomerang does
  NOT despawn or flip to return on enemy hit (it passes through, hitting each
  enemy once per throw). In `combat::damage::apply_attack_hit`, the
  `AttackKind::Boomerang` arm additionally applies **stun**:
  - Skeleton → `skeleton::stagger(world, id)` (the prepared hook; drops the
    shield — a boomerang answer to the front-guard).
  - Other families → a small generic stun: add a `pub fn enemies::stun(world,
    id, ticks)` that zeroes velocity and holds the family's state machine
    (simplest additive: a `stun_ticks: u16` on each family's data checked at
    the top of `update_one` — do slime/bat/octorok/raiders/wisp-visible; wisp
    while phased is immune, raider-spear guard is pierced by stun — that's
    the point). ~60 ticks, blue dizzy-sparkle FX overhead
    (`FxKind`-additive), `SfxId` reuse `SkeletonRattle` or a soft chime.
  - Style: `StyleVerb::BoomerangStun` on any successful stun (extend
    `verb_cooldowns` to 7 and the style tables; GAME_DESIGN §3).
- **Hits — puzzle tiles**: in `puzzle::process_hits`, add the Boomerang arm
  alongside SwordBeam/Rock: `from_player`-implicit (boomerangs are always the
  player's), test center vs the same overworld chime/crank/barricade rects
  with `swing_id = throw_id`, and DO NOT mark `hit`/despawn — one throw can
  ring gate chimes at range, turn the bridge/far-switch cranks, and (with a
  good line + the return path) contribute multiple finale chimes. Barricade
  damage arm: `AttackKind::Boomerang → 1`. This is the **overworld
  retro-enable** — zero special-casing, verify it live (M5).
- **Pickups**: boomerang overlap with `EntityKind::Pickup` magnetizes them to
  it (classic). Cheap version: on overlap, set the pickup's position to
  follow the boomerang; they collect on catch. Keep under 20 lines; skip if
  it fights the pickup magnetism code — log either way.
- **Item-get**: `Loot::Boomerang` chest arm in `interact::open_chest`: sets
  `ITEM_BOOMERANG` flag, `selected_item = 2`, full item-get beat — world
  pauses via dialog (`TextId::BoomerangGet`: "The Gale Boomerang! / Tap ITEM
  to throw. Q cycles gear. / The wind answers in order."), hold-up toast
  "GALE BOOMERANG", `SfxId::GemGet`-class fanfare (own `SfxId` fine),
  sparkle FX.
- **Cycle rework** (`player::update_item_cycle`): build the unlocked list
  (bombs if `bomb_cap > 0`, boomerang if `ITEM_BOOMERANG`), Q/LB+RB advances
  through it, `SfxId::ItemCycle`, HUD flash. HUD item slot draws boomerang
  icon when selected (no count).
- **Persistence**: `selected_item` already flows through
  `PlayerPersist`/`SaveGame`; the boomerang itself is a flag. Verify a
  save/reload mid-dungeon keeps the item.
- Exhaustive-match sweep: `EntityKind::Boomerang` arms in
  `integrate_non_player` (boomerang module moves it itself — no knockback
  physics), `enemies::update_enemies` (no), `resolve_hits` (per above),
  `render_entity` (spin sprite), `sword.rs update_beams` match (no),
  spawner `is_hostile_kind` (no).

DoD: with a debug flag grant (F1+H style, behind overlay, logged), the
boomerang throws/returns/catches; stuns a skeleton through its shield and a
raider through guard; rings an overworld gate chime from across the gap;
turns the ruins far-switch crank (the 2C "Phase 3 preview" site pays out);
style chip reacts; reload keeps it.

### M3 — Dungeon layout, keys, shutters, minimap

**Layout** (GAME_DESIGN §5 flow; concrete geometry is yours, keep the flow):

```
            [Seal W]   [Sanctum antechamber]  [Seal E]
                |               |                |
[Trials 3] - [Trials 2]   [Hall of Currents hub]
                |               |          \
[Boomerang]  [Trials 1]    [Currents N]   [Flame room]
       (left wing)              |
                          [Vestibule]        [Guardian Arena]
                                |                  |
                          (ShrineLobby)      [Sanctum Core] (above antechamber)
```

- **Vestibule** (entry 0, checkpoint 7): safe, lore tablet
  (`TextId::DungeonLore`: teal-stone flavor + "the wind sleeps in the west
  wing"), locked **west wing door is OPEN** (design says locked left/right
  wings — planner decision: west wing opens by walking, the **east/Currents
  door is `SmallKey`-locked** so key #1 from Trials teaches the loop),
  shuttered north door to the antechamber (opens only when both seals break —
  M4).
- **Hall of Trials** (west wing, 3 rooms, groups 90–92): each room slams
  shutters (`ShutterSlam`) on entry until its spawner group clears
  (`GroupCleared` → reopen via `set_tile`, handled in an additive
  `events.rs` arm calling `puzzle::dungeon`). Room 1: slimes + skeleton
  (sword check). Room 2: raiders + a plate the player must stand on while
  fighting (2C plate logic on a 1-plate court holding the exit gate open —
  reuse `T_PLATE_*` tiles + a dungeon-local plate def). Room 3: wisps +
  octorok (ranged pressure). Reward room: **Gale Boomerang chest**
  (cinematic per M2) + small key #1 chest (`DKEY_SMALL_1`,
  visible on the way out — "on visible, clued paths").
- **Hall of Currents** (east, key #1 door): the curriculum hub (M4 details).
  A side room holds small key #2 (`DKEY_SMALL_2`) behind a
  carry-a-flame puzzle; key #2 opens the inner door (`DDOOR_INNER`) between
  the hub and the seal-room approaches.
- **Seal rooms** (west + east of the antechamber): M4.
- **Sanctum antechamber** (checkpoint 8): both seals broken → central
  `D_LIFT` glows + the north shutter opens into **Sanctum Core** — dressed,
  with a sign stub (`TextId::SanctumStub`: "Something ancient stirs…
  (Phase 3B)") and the dormant `GRP_DNG_SANCTUM` spawn slots placed but
  group-locked at populate (locked_groups += 94 unconditionally this phase).
  **Boss key chest** is here, visible behind the miniboss floor —
  reachable this phase (the fight arrives in 3B; do not gate the chest on
  it, gate the ARENA door on the key). Guardian Arena sits behind the
  `D_DOOR_BOSS` boss door (openable with the boss key — inside is dressed
  empty with a stub sign `TextId::ArenaStub`).
- **Water accents**: teal channels along Currents halls; boomerang flies over
  water (it ignores tile collision except SOLID — decide + log whether
  WATER blocks it; recommendation: it flies over WATER, giving cross-channel
  crystal shots).
- Checkpoints: 7 on Vestibule entry, 8 on antechamber entry
  (`TriggerKind::Checkpoint`), matching `EntryPoint`s. Extend `Game::new`
  boot rule per §1.8: saved map Dungeon → boot Dungeon at checkpoint entry.

**Dungeon minimap** — new `game/src/ui/dungeon_map.rs` (do not grow
`minimap.rs`): room-grid map on M (corner chip: current room + explored
neighbors) and the pause map (Esc) swaps to the dungeon page while inside.
Discovered rooms from `DROOM_*` flags; each discovered room draws its exits
exactly from `RoomDef.exits` (locked = small dot, boss = skull chip, seals =
rune chip); current-room blink; boomerang/keys/boss-key pips. **Reciprocity
is data-driven** — the same `rooms()` table drives both the physical doors
and the map, so they cannot disagree; the M1 assert enforces the map data
against painted tiles.

DoD: full dungeon walkable with debug key grants; three Trials rooms clear
with shutters; both keys route correctly (key 1 → Currents door, key 2 →
inner door); boss key chest collectable; boss door opens with it; minimap
matches topology exactly; death in the dungeon respawns at the last dungeon
checkpoint; reload mid-dungeon boots back inside.

### M4 — Wind-crystal curriculum + ordered seal rooms

Site data in new `content/src/puzzles_dungeon.rs` (same `&'static` style as
`puzzles.rs`; keep `OverworldPuzzles` untouched); runtime in new
`game/src/puzzle/dungeon.rs` with its own `DungeonPuzzleState` hung off
`Game` (rebuild per map like `PuzzleState`; call from `puzzle::update` when
`current_map == Dungeon` — the mod.rs entry point stays tiny).

- **Crystal toggles (taught)**: a `D_CRYSTAL_BLUE`/`D_CRYSTAL_AMBER` crystal
  tile toggles the room family's gate pair on ANY player hit (sword, beam,
  bomb, boomerang — same `try_hit_tiles` discipline, crystal rects from the
  dungeon def). Toggle swaps `D_GATE_BLUE_UP ⇄ DOWN` and `AMBER DOWN ⇄ UP`
  everywhere in the room (`set_tile`), crystal sprite swaps color,
  `SfxId::CrystalBlue/Amber` (two pitches). Room 1 of Currents: crystal
  right next to the gate (sword works — teaches the toggle). Room 2: crystal
  across a water channel (sword can't reach — **boomerang required**; beam
  works at full hearts, fine). Room 3: two gates interleaved so the player
  must toggle from a specific side — teaches throw lines.
- **Around-the-corner multi-target**: a room where two crystals must end up
  in the SAME state to open a double gate, positioned so one curved
  throw-and-return can flip both (and a sloppy throw flips one — recoverable
  by another throw; never lockable).
- **Carry-a-flame** (key #2 room): `D_BRAZIER_ETERNAL` (always lit) + 2
  `D_TORCH_UNLIT`. Boomerang passing within a tile of the eternal brazier
  ignites (`BoomerangData.flame = true`, flame trail FX, `TorchLight` cue);
  passing an unlit torch while aflame lights it (flame transfers, boomerang
  keeps flying); passing a LIT torch while unlit **snuffs** it
  (`TorchSnuff`) — so a careless return path undoes work: the puzzle is
  choosing a line that lights both without re-crossing. Both lit → key chest
  gate opens. Torches reset when you leave the room (unsolved) — flame
  state is room-local, no flags until solved.
- **Seal rooms (west/east)**: 3 crystals + `D_RUNE_1/2/3` floor runes
  telegraphing the order + a rune-marked throw plinth. All 3 crystals hit
  **within one throw** (same `throw_id`) **in rune order** → seal breaks:
  `SEAL_WEST/EAST` flag, seal door shatter (`SealBreak` + shake + shard FX),
  antechamber pillar lights. Wrong order or a partial throw: `RuneBad` dull
  cue + crystals pulse-reset (per-throw state only — nothing to un-solve).
  Hit order tracked in `DungeonPuzzleState` as `(throw_id, Vec<crystal_idx>)`.
  Geometry: outbound line takes crystals 1→2, the homing return sweeps 3 —
  author each room so the intended plinth throw works reliably and a
  freestyle solve is possible; west and east rooms differ in geometry (one
  favors a corner-bend return, one a cross-water line).
- **Both seals** → antechamber shutter opens (persisted via the two flags;
  restore path in `puzzle::dungeon::restore` mirrors 2C `restore`), toast
  "THE WAY OPENS", checkpoint 8 save.
- Solved-state persistence: crystals/gates restore to a canonical solved pose
  for finished rooms; unsolved rooms repaint closed at load
  (`paint_closed`-equivalent for the dungeon def, called from `switch_map`/
  `Game::new` beside the overworld calls).

Text appends: `TextId::{DungeonLore, CurrentsSign, FlameSign, SealSignW,
SealSignE, SanctumStub, ArenaStub, BoomerangGet, DoorLocked}` — short,
diegetic, never naming buttons except the item-get.

DoD: curriculum beats in order (toggle → range → lines → multi-target →
flame → ordered seals) each solvable with intended tools and none
soft-lockable; seals persist; reload restores exactly.

### M5 — Feel pass, HUD, overworld retro-check, completion

- **HUD**: small-key count + boss-key pip while in dungeon (near the rupee
  counter or item slot — your call, consts in `ui/hud.rs` style); boomerang
  icon in the item slot; dungeon room name banners optional (skip if noisy).
- **Boomerang feel pass**: throw buffer (a tap during catch animation
  re-throws next tick), catch magnetism radius, stun durations, flight
  speeds — iterate ±30%, log. Distinct pitch for throw vs catch; flight adds
  a soft per-throw whir (repeat a short spec 2–3× if the engine has no loop —
  keep it subtle or drop it, log).
- **Overworld retro-enable verification** (acceptance-critical): from a
  post-dungeon-item save, verify in the browser: gate chimes ring from range,
  bridge crank + ruins far-switch turn on one throw, finale chimes are
  markedly easier (2 throws or throw+dash), skeleton/raider stuns work in
  the field, barricades chip. Screenshot evidence.
- Perf: F1 in the busiest Trials room + a Currents room with water + FX —
  hold ~60; dungeon chunk-cache behavior is the same map pipeline (one map,
  chunks LRU) — verify no thrash on room slides (slides don't switch maps).
- Full keyboard run: New Game → (debug-skip allowed via F4 teleports for
  iteration, but do one honest run) 3 gems → shrine → lobby → Vestibule →
  Trials → **Boomerang** → Currents → both seals → antechamber → boss key →
  boss door opens → Arena stub sign. Reload tests: mid-Trials-shutter,
  post-key-1, mid-seal-throw, post-seals.
- WORKER_NOTES completion entry: landed/deviations/tuning, screenshots,
  **explicit frozen-seams list for 3B** (see §6).

## 4. Definition of Done (3A)

1. Dungeon reachable only through the opened shrine; rooms slide (no fades
   between rooms); camera locks per room; death/reload inside the dungeon
   resumes at dungeon checkpoints.
2. Gale Boomerang: throw/return/catch feel; stuns every stumable family incl.
   skeleton-through-shield; `StyleVerb::BoomerangStun` live; B-item cycle
   bombs⇄boomerang with HUD; persists across doors + reloads.
3. Boomerang is a first-class puzzle hit source: overworld chimes/cranks/
   barricades react with zero special-casing (verified live); dungeon
   crystals/torches/seals are tool-agnostic where physically reachable and
   boomerang-required only by geometry.
4. Keys/doors: 2 small keys + boss key on visible clued paths; locked/boss
   doors consume correctly and persist; combat shutters never soft-lock
   (clearing or leaving always possible).
5. Curriculum: crystal toggle → ranged toggle → throw lines → multi-target →
   carry-flame → two ordered seal rooms; both seals open the antechamber;
   all states reset/restore correctly.
6. Dungeon minimap: discovered rooms + exact exit reciprocity (data-driven +
   asserted); keys/boomerang pips.
7. Sanctum Core + Guardian Arena rooms dressed, dormant, sign-stubbed;
   `GRP_DNG_SANCTUM` reserved + locked; boss door functional.
8. check/clippy/trunk clean; no file >~600; F3 Arena + full overworld
   critical path unbroken.
9. WORKER_NOTES completion entry with frozen 3B seams.

## 5. File ownership (3A)

**Creates**: `game/src/rooms.rs`, `game/src/items/boomerang.rs`,
`game/src/puzzle/dungeon.rs`, `game/src/ui/dungeon_map.rs`,
`game/src/events.rs` (extraction), `world/entity_data.rs` (extraction),
`content/src/maps/dungeon.rs` (+`dungeon_rooms.rs` if needed),
`content/src/puzzles_dungeon.rs`, `content/src/art/tiles_dungeon.rs`,
`content/src/art/props_dungeon.rs`, `content/src/art/items.rs`.

**Edits (additive)**: MapId codec + `build()`, `interiors::shrine_lobby`,
catalog 280–309, flags 100–139 + groups 90–94, SFX appends, TextId appends,
`EntityKind::Boomerang` + data + match sweep, `AttackKind::Boomerang`,
`StyleVerb::BoomerangStun` + `verb_cooldowns` 7, `Loot` variants,
`update_item_cycle` + tap dispatch, `puzzle::process_hits` boomerang arm +
`barricade_damage` arm, `resolve_hits` boomerang arm,
`apply_attack_hit` stun arm + `enemies::stun`, `Camera::set_bounds`,
`Game::new` dungeon-boot rule, `events.rs` shutter/GroupCleared arm, HUD
key/item additions, `lib.rs` wiring (rooms update + slide pause).

**Must NOT touch**: `engine/*` (all of it), `minimap.rs` internals,
`WaveDirector`, frozen 2C-A/2C-B seams (puzzle overworld API, shop, camp
chain), boss/miniboss content, docs other than WORKER_NOTES.

## 6. Frozen seams for 3B (state them in your completion entry)

1. `RoomDef`/`rooms()` shapes + `RoomsState` + slide API.
2. Dungeon map geometry: Sanctum Core + Guardian Arena rects, their entries,
   `GRP_DNG_SANCTUM = 94`, boss-door tiles, checkpoint 8.
3. `AttackKind::Boomerang` semantics (pass-through, stun-on-hit, damage 0.5)
   + `enemies::stun` + `throw_id` dedupe rule.
4. Flags 100–139 as allocated; 140–149 reserved for 3B.
5. `DungeonPuzzleState` + crystal-toggle API (3B's boss crystals are
   ENTITIES, not these tiles — but the prime/pair logic may crib the code).
6. Catalog 280–309; 310–319 reserved for 3B (boss arena dressing).

## 7. Verification protocol

- Per milestone: wasm32 check + clippy, `env -u NO_COLOR trunk build
  --release`, targeted Playwright smoke vs `python3 -m http.server 8090
  --directory dist` (scripted keys; screenshots to `/tmp/p3a_smoke/`;
  localStorage assertions for new flags). Kill server + headless browser
  after every run.
- Dungeon-specific asserts: exit reciprocity (compile-time table + load-time
  debug assert), no-soft-lock sweep (enter each room with nothing, verify
  exit or reset), seal wrong-order → reset → correct-order pass, key
  double-spend impossible (flags, not counters).
- Save-shape: old 2C saves load clean (no new required fields).
- Human feel notes at completion: boomerang throw/catch, slide speed, shutter
  fight difficulty, seal-room throw tolerance.
