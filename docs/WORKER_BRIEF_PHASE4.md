# WORKER_BRIEF_PHASE4.md — Meta: title, chapter select, pause/help, touch + gamepad (Phase 4)

You are a Grok worker. This brief is self-contained; follow it exactly. Conflict
order: `DECISIONS.md` → `ARCHITECTURE.md` → this brief → `GAME_DESIGN.md`.
You run autonomously — do not ask the human anything except for credentials.

**GATE: Phase 3B is complete (`1004154`, Gate B). Read the "Phase 3B completion"
entry in `docs/WORKER_NOTES.md` first — its "Phase 4 inherits" list is your
starting contract. If code and this brief disagree, code + WORKER_NOTES win;
log drift and adapt.**

This phase has two **sequential** parts in one brief:

- **4A — Meta shell** (M1–M5): title screen, chapter select, pause/help
  overlay, save/settings UX, credits polish. Owns `game::state`, `game::ui`,
  tiny keyboard/audio engine touches.
- **4B — Touch + mobile + gamepad verification** (M6–M9): touch backend v2
  (full button set), skinned touch UI, mobile page polish, gamepad
  verification path. Owns `engine::input::touch`, `game::ui::touch` (new),
  `index.html`. **Do not start 4B milestones until all of 4A is committed** —
  4B draws pause/title affordances 4A creates.

## 0. Mission

Gate B shipped the full Act 1 critical path, but the game boots straight into
the world, Esc only opens a map, touch players cannot press Item/Interact/
Pause at all, and there is no New Game / Continue. Phase 4 wraps Act 1 in a
real meta shell and makes **all three input methods** first-class:

1. Cold load → **title screen** → New Game / Continue / Chapter Select all work.
2. The entire game is completable with **gamepad alone** and **touch alone**.
3. **Pause** shows the current objective + bindings for keyboard, gamepad, touch.
4. Reload mid-dungeon resumes at the last checkpoint with correct flags/rupees.

Gameplay feel stays priority #1: menus must be snappy (no eased 30-tick menu
fades), pausing must be instant, and nothing in this phase may regress the
play-mode input path.

## 1. HARD DEPENDENCY — code reality (verify each before building)

Written against post-3B main (`1004154`). Verify in code:

1. **GameMode** (`game/src/state.rs`) is `Play | Transition(Transition)` only.
   `Game::new` (`game/src/lib.rs`) builds the world immediately from
   `SaveGame` and boots into `Play`. `Game::from_storage_json(None, ..)` falls
   back to `SaveGame::default_spawn()` — the game currently cannot tell "no
   save" from "fresh save".
2. **Pause today**: `MinimapState::update` (`ui/minimap.rs` ~line 90) toggles
   `pause_open` on `BUTTON_PAUSE || BUTTON_CONFIRM` **itself**, and
   `Game::update` early-outs when `dialog.open || minimap.pause_open ||
   shop.open`. The dungeon pause page is `ui::dungeon_map::render_pause`.
   There is no menu, no objective text, no bindings display.
3. **Menu input precedent**: `ui/shop.rs` — cursor via `move_vec.1` edge
   detection against `prev_move_y` (thresholds −0.55/+0.55 with ±0.4
   hysteresis), select on `BUTTON_ATTACK || BUTTON_INTERACT`, close on
   `BUTTON_PAUSE || BUTTON_DASH`. Reuse this exact pattern for every menu so
   keyboard/gamepad/touch all work identically.
4. **Buttons** (`engine/src/input/mod.rs`): `BUTTON_ATTACK/ITEM/DASH/INTERACT/
   PAUSE/CONFIRM/CYCLE`, `BUTTON_COUNT = 7`. Keyboard map in `keyboard.rs`
   (J/Space, K, L/Shift, E, Esc, Enter, Q + KeyM minimap pulse). Gamepad map
   in `gamepad.rs` (0=Attack, 2=Item, 1=Dash, 3=Interact, 9=Pause, 4|5=Cycle,
   axes 0/1 + d-pad 12–15). Touch (`touch.rs`) has **only** joystick (left
   half, floating origin) + Attack (430,230 r18) + Dash (400,250 r18).
5. **Touch overlay conflict**: HUD item slot consts are `ITEM_X=450, ITEM_Y=240`
   (`ui/hud.rs`) — the touch Attack circle at (430,230) r18 already overlaps
   it. 4B re-layouts the touch cluster and moves the B-item readout into the
   touch Item button when `touch_active`.
6. **Save** (`game/src/save_data.rs`): `SaveGame` v2 (`SAVE_VERSION=2`,
   key `shard_save_v1`), fields incl. `map/entry/checkpoint/hearts/max_hearts/
   rupees/gems/flags/fog/bombs/bomb_cap/selected_item`, all new fields must be
   `#[serde(default)]` and version stays 2. Saves are written via
   `state::save_from_game(game).to_json()` → `game.pending_save` →
   `GameEvent::Save` drained in `events.rs` → app → `engine::save`.
7. **Victory / progress flags** (`content/src/flags.rs`): Act 1 complete
   predicate = `SHARD_OF_COURAGE (143)`. Checkpoint ids: 0 New-Game south
   gate, 1 village, 2–4 gems, 6 shrine, 7 vestibule, 8 antechamber, 9
   pre-boss. `QUEST_STARTED=1`, gems bitmask in `game.gems`.
8. **Credits** (`ui/credits.rs`): active overlay drawn from `Game::render`,
   skip = hold Attack 60 ticks, driven from `boss::update` victory flow
   (`boss/mod.rs`). GAME_DESIGN §8 wants **R to skip**.
9. **Audio** (`engine/src/audio.rs`): per-SFX one-shots, no master gain, no
   mute. App adapter: `GameEvent::Sfx(SfxId)` → `sfx::spec` → `audio.play`.
10. **Objective source**: `MinimapState::refresh_objective` +
    `minimap.objective: Option<(u32,u32,TextId)>` already computes the current
    goal (gems → shrine). Reuse; don't build a parallel quest tracker.
11. **File caps** (~600): `lib.rs` 526, `draw_world.rs` 521, `interact.rs` 463,
    `minimap.rs` 458. New UI goes in new files under `game/src/ui/`.

## 2. Locked constraints

- No new crates. No `web-sys` outside `engine`. No string literals for player-
  facing copy in `game` — add `TextId` variants / const arrays in
  `content::text`.
- `SAVE_VERSION` stays 2; every new `SaveGame` field is `#[serde(default)]` so
  Gate B saves keep working. Never wipe a save except explicit New Game.
- **No gameplay tuning changes** in this phase (that's Phase 5). Play-mode
  update order in `Game::update` may only gain early-outs, never reorder.
- Rupees **carry** across chapter-select restarts (DECISIONS §6); everything
  else (flags, gems, checkpoint, bombs, fog) resets on "Restart Act 1".
- Menus pause the world completely (viewer/dialog precedent: early-out before
  `world.tick` advances).
- Engine changes allowed in this phase, exhaustively: keyboard `KeyR` →
  `BUTTON_CONFIRM`; gamepad button 8 (Back/Select) → `BUTTON_CONFIRM`;
  `Audio::set_muted(bool)` + early-return in `play()`; touch backend v2 (M6);
  `TouchOverlay`/`InputState` additions (M6). Nothing else.
- Commit prefix `phase4:`, at least one commit per milestone; `cargo check` +
  `clippy -D warnings` (wasm32) + `env -u NO_COLOR trunk build --release`
  clean at every commit. Files <600 lines.

## 3. Part 4A — Meta shell (M1–M5)

### M1 — Mode plumbing + pause routing rework

1. `GameMode` gains `Title`. New file `game/src/ui/title.rs`:
   `TitleState { cursor, page: TitlePage, confirm_wipe: bool }` with
   `TitlePage::{Main, Chapters}`.
2. `Game::from_storage_json` records `had_save: bool` (json was `Some` AND
   parsed at version 2 — expose a `SaveGame::try_from_json(json) ->
   Option<SaveGame>` beside the lossy `from_json`). `Game::new` gains a
   `boot_to_title: bool` (app passes `true`); when set, `mode =
   GameMode::Title` after building the world exactly as today (world exists
   behind the title, camera snapped — cheap and lets Continue be instant).
3. New `game/src/ui/pause.rs`: `PauseState { open: bool, page: PausePage,
   cursor: usize }`, `PausePage::{Map, Help, Options}`. Route ALL pause input
   here from `Game::update` **before** the dialog/shop early-out block:
   - `BUTTON_PAUSE.pressed` toggles `pause.open` (world freezes via early-out,
     same pattern as `minimap.pause_open` today).
   - While open: left/right (`move_vec.0` edges, shop hysteresis pattern)
     cycles Map ↔ Help ↔ Options; Esc/Dash closes.
   - **Remove** the `BUTTON_PAUSE || BUTTON_CONFIRM` toggle from
     `MinimapState::update`; `minimap.pause_open` is replaced by
     `pause.open && page == Map` (keep the KeyM corner-map toggle pulse).
     Sweep all `minimap.pause_open` readers (`lib.rs` render + update,
     dungeon_map) — compiler-walk this.
   - Map page renders the existing `minimap.render_pause` (overworld) or
     `dungeon_map::render_pause` (dungeon) unchanged.
4. Keyboard: add `"KeyR"` → `key_held[BUTTON_CONFIRM]`. Gamepad: button 8 →
   `pad_buttons[BUTTON_CONFIRM]`. (R doubles as menu-confirm; acceptable,
   log it.)

### M2 — Title screen

1. Render: full-screen panel (`d.rect` bg `#0a0a12`), game logo — author a
   `SpriteDef` wordmark ("SHARD OF THE TRIFORCE", ~200×48, 2 frames for a
   subtle shimmer) in a new `content/src/art/ui_meta.rs` registered in
   `all_bakes()`; triforce-shard accent + drifting particle sparkles via
   `d.rect` (no new fx systems).
2. Menu rows: **CONTINUE** (only when `had_save` and save has progress:
   `checkpoint > 0 || !flags.is_empty()`), **NEW GAME**, **CHAPTER SELECT**,
   **SOUND: ON/OFF**. Cursor = shop pattern; select on Attack/Confirm/
   Interact.
   - CONTINUE → `mode = Play` (world already built from the save).
   - NEW GAME with an existing progressed save → inline "ERASE SAVE?"
     confirm row pair (NO default). Confirmed: build fresh
     `SaveGame::default_spawn()`, push `pending_save`, rebuild via
     `state::switch_map(game, MapId::Overworld, 0)` after resetting
     `game.flags/gems` and player fields — verify player stats reset (hearts
     6/6, 0 rupees, no bombs, `selected_item=0`, empty fog via
     `minimap.load_fog(&[])`).
   - SOUND toggles `muted` (M5) and plays a confirm blip when unmuting.
3. Title ambience: reuse `SfxId::TextBlip`/`RankUp` for cursor/confirm (new
   `SfxId::{MenuMove, MenuConfirm, MenuBack}` appends preferred — adapter
   covers them automatically).
4. In-game "Quit to Title" (from pause Options, M4) re-enters
   `GameMode::Title` after a `save_from_game` flush — never lose progress.

### M3 — Chapter select

1. `TitlePage::Chapters`: three cards side by side (~140×150 each).
   - **Act 1 — Shard of Courage**: unlocked. Shows live progress from the
     loaded save: gems ×3 (sprite pips), heart pieces n/4, rupees, and a
     "COMPLETE" ribbon when `SHARD_OF_COURAGE` is set. Actions: **PLAY**
     (= Continue) and **RESTART** (rupee-carry reset: keep `pd.rupees`, wipe
     flags/gems/bombs/fog/checkpoint→0, same rebuild path as New Game;
     confirm row).
   - **Act 2 — Storm Sigils** / **Act 3 — Molten Crucible**: locked contract
     cards (GAME_DESIGN §9 flavor: one line each + "LOCKED" chip, dimmed).
     Selecting plays `SfxId::Refused`. Copy goes in `content::text`
     (`TextId::Act2Card`, `TextId::Act3Card`).
2. Left/right moves between cards, up/down within a card's actions,
   Dash/Esc back to main title page.

### M4 — Pause pages: Objective + Help, Options

1. **Help page** (this satisfies acceptance 3): three columns — KEYBOARD /
   GAMEPAD / TOUCH — one row per verb: Move, Attack, Item (tap=use,
   hold=shield), Cycle item, Dash, Interact, Map, Pause. Copy from a new
   `content::text::binding_rows()` const table (no literals in `game`).
   Above the table: **OBJECTIVE:** line — derive from game state:
   `!QUEST_STARTED` → "VISIT THE ELDER"; gems <3 → "RECOVER THE THREE GEMS
   (n/3)"; shrine not open → "OPEN THE TRIFORCE SHRINE"; no boomerang →
   "EXPLORE THE SHRINE DEPTHS"; seals <2 → "BREAK THE TWO SEALS";
   `!WARDEN_DEFEATED` → "DEFEAT THE GRANITE WARDEN"; else "ACT 1 COMPLETE".
   Implement as `fn objective_line(gems, &flags) -> TextId` in
   `ui/pause.rs` with variants in `content::text`.
   - **Input echo** (gamepad verification seam, 4B M9 finishes it): at the
     bottom of Help, render the live `InputState` — move vector as a small
     stick dot + one chip per button that lights while held. Pure read of the
     `input` param; ~15 lines.
2. **Options page** rows: RESUME / RESTART FROM CHECKPOINT (=
   `switch_map(current_map, world.checkpoint)`, the death path without the
   heart reset) / QUIT TO TITLE / SOUND ON-OFF.
3. Pause opens with `SfxId::MenuConfirm`, closes with `MenuBack`; a dim
   `rgba(0,0,0,0.55)` scrim (map-page precedent).

### M5 — Settings persistence, mute, credits polish

1. `SaveGame` += `#[serde(default)] pub muted: bool`. Thread through
   `save_from_game` (read from a new `game.settings: Settings { muted }`) and
   apply on boot.
2. New `GameEvent::SetMuted(bool)`; emit on toggle AND once on boot; app
   forwards to `platform.audio.set_muted(b)` (engine: store flag, early-return
   in `play()`; keep `resume()` untouched so unlock still happens).
3. Credits: skip = `BUTTON_CONFIRM.pressed` (R / Enter / pad Back) **or** the
   existing hold-Attack fallback (touch/gamepad-A). Update the footer copy to
   "R / HOLD ATTACK TO SKIP". After credits end, land in `GameMode::Title`
   with the Chapters page pre-selected (post-victory beat already returned the
   world to the village — verify against `boss::update` victory flow before
   rewiring, and keep the village return intact when credits are skipped).
4. Reload-mid-dungeon acceptance check: kill the tab at checkpoint 8/9, reload
   → title → CONTINUE → boots dungeon at the checkpoint with keys/flags/
   rupees intact (`Game::new` already handles dungeon boot; you are only
   adding the title hop in front — verify nothing double-applies).

## 4. Part 4B — Touch v2 + mobile + gamepad verification (M6–M9)

### M6 — Touch backend v2 (`engine/src/input/touch.rs`)

1. Replace the two hardcoded circles with a data-driven button table:

```rust
struct TouchButton { button: usize, cx: f32, cy: f32, r: f32 }
const TOUCH_BUTTONS: &[TouchButton] = &[
    // right-hand cluster, thumb arc (480×270 logical space)
    TouchButton { button: BUTTON_ATTACK,   cx: 446.0, cy: 224.0, r: 20.0 },
    TouchButton { button: BUTTON_ITEM,     cx: 404.0, cy: 246.0, r: 16.0 },
    TouchButton { button: BUTTON_DASH,     cx: 462.0, cy: 184.0, r: 14.0 },
    TouchButton { button: BUTTON_INTERACT, cx: 412.0, cy: 196.0, r: 13.0 },
    TouchButton { button: BUTTON_CYCLE,    cx: 376.0, cy: 218.0, r: 10.0 },
    TouchButton { button: BUTTON_PAUSE,    cx: 466.0, cy: 12.0,  r: 10.0 },
];
```

   Hit test on touchstart (nearest button whose radius contains the point,
   +4 px grace); a touch keeps its role until release (existing `TouchRole`
   map — extend the enum to `Button(usize)`). Tune positions freely for thumb
   reach but keep: Attack biggest and outermost; Pause isolated top-right
   (never near combat buttons); ≥8 px gaps between circles.
2. Joystick: unchanged floating-origin left-half behavior. A joystick touch
   that starts on the left half must never claim buttons (all buttons sit at
   x>370, fine).
3. `TouchOverlay` v2: replace `attack_pos/dash_pos` with
   `buttons: Vec<TouchButtonGeom> { button, cx, cy, r, held }` (+ keep
   joystick origin/knob). Compiler-walk the two consumers (`game/src/lib.rs`
   render block, this brief's M7 replaces it anyway).
4. `InputState` += `menu_tap: Option<(f32, f32)>` — logical coords of a
   touchstart that claimed **no** role (right half only, to avoid joystick
   noise), latched like `minimap_pulse`, cleared in `end_frame`. Menus use it
   for direct row taps.
5. Multi-touch: joystick + any two buttons concurrently must work (existing
   HashMap roles give this — verify attack-hold + dash-tap while moving).

### M7 — Touch skin + menu taps (`game/src/ui/touch.rs`, new)

1. Replace the raw circles in `Game::render`'s `touch_active` block with
   `ui::touch::render(d, &overlay, world, &sprites)`:
   - Buttons as skinned discs: dark ring + translucent fill
     (`rgba(20,28,40,0.5)`), glyph per verb (sword icon / item / boots-dash /
     "!" interact / rotate-cycle / pause bars) — author one 12×12
     `SpriteDef` strip `touch_icons` in `content/src/art/ui_meta.rs`;
     `held == true` brightens the fill. Labels beat beauty: if in doubt,
     letter glyphs A/B/D/E/Q.
   - **Item button doubles as the B-item readout**: draw the selected item
     sprite (bomb + count / boomerang, logic from `ui/hud.rs` item-slot arm)
     inside the Item disc. When `touch_active`, `hud::draw` skips its
     450,240 item slot (pass a flag) so nothing overlaps the cluster.
   - Joystick: keep engine geometry, restyle (outer ring + knob disc,
     `rgba` alphas ~0.25/0.45 as today).
2. Menus (title/chapter/pause/shop): on `menu_tap`, hit-test row/card rects
   and treat as move-cursor-there + confirm (single tap = select; two-step
   for destructive rows — first tap moves cursor, second confirms — reuse
   the confirm_wipe pattern). Add tap support to shop rows too (it's the
   same rect list its render already uses).
3. Dialog advance: any `menu_tap` OR Attack (already works) advances
   `ui/dialog.rs`.
4. Minimap corner toggle on touch: tapping the corner minimap rect (408,8
   68×68) toggles `show_corner` — the KeyM pulse path stays for keyboard.

### M8 — Mobile page + iPhone polish (`index.html`)

1. CSS: `height: 100dvh` on html/body (fallback 100%), `overscroll-behavior:
   none`, `-webkit-touch-callout: none`, `user-select: none`,
   `-webkit-user-select: none`. Add `<meta name="apple-mobile-web-app-capable"
   content="yes">` + status-bar meta. Keep `touch-action: none` +
   `viewport-fit=cover` (already present).
2. Portrait hint: in `game::ui::touch`, when `touch_active` and the canvas CSS
   scale suggests portrait (engine exposes nothing — add
   `InputState.viewport_portrait: bool` set in the engine resize path from
   `inner_width < inner_height`), draw a "ROTATE DEVICE ↺" scrim overlay and
   early-out world updates (treat as paused). Small, honest, effective.
3. Verify (Playwright mobile emulation, iPhone 14 viewport, touch enabled):
   boot → title tap NEW GAME → move via joystick, attack a dummy is not
   available (overworld: slash grass-side bat), open pause via touch pause
   button, read Help page, tap-resume, tap Item to place a bomb after F1+H
   grant, interact with a sign via touch Interact, cycle item. Screenshot
   each step to `/tmp/p4_smoke/`. Real-iPhone pass is Phase 5's Gate C item;
   log that it remains owed if you have no device.

### M9 — Gamepad verification path

1. Hardware checklist (run if any pad is available; otherwise leave the
   checklist in WORKER_NOTES marked "owed — no hardware"):
   title nav + confirm/back, full verb set in play (attack/item-tap/
   item-hold-shield/cycle LB-RB/dash/interact), pause + all three pages,
   chapter select, shop purchase, credits skip via Back(8), map page.
2. Code-level: the Help-page input echo (M4) doubles as the on-screen gamepad
   tester — verify every mapped button lights it. Add d-pad (12–15) to the
   echo's stick dot.
3. Non-standard mapping fallback: if `pad.mapping() != "standard"`, log one
   console warning (engine, `gamepad.rs`) and continue with the same indices —
   document in WORKER_NOTES; do NOT build a remapping UI.

## 5. Definition of Done (Phase 4)

1. Cold load (no localStorage) → title with NEW GAME + CHAPTER SELECT (no
   CONTINUE); with a progressed save → CONTINUE first and default.
2. New Game erases (with confirm) and starts the elder intro path; Continue
   resumes exact checkpoint incl. mid-dungeon (cp 7/8/9) with flags/rupees.
3. Chapter select: Act 1 card with live progress + RESTART (rupees carry,
   everything else fresh); Acts 2–3 locked cards.
4. Esc/Start/touch-pause → pause with Map / Help / Options pages; Help shows
   objective + full 3-column bindings + live input echo.
5. Touch alone can: navigate title, play every verb (attack, item tap/hold,
   cycle, dash, interact), pause, buy in shop, skip credits. Playwright
   mobile-emulation screenshots prove it.
6. Gamepad alone can do the same (hardware pass or documented as owed).
7. Mute persists across reload; credits skippable with R.
8. `cargo check` + `clippy -D warnings` (wasm32) clean;
   `env -u NO_COLOR trunk build --release` ok; files <600 lines; no play-mode
   behavior/tuning changes.
9. Append a "Phase 4 completion" entry to WORKER_NOTES: what landed,
   deviations, frozen seams for Phase 5 (title/pause APIs, touch button
   table, `Settings`, `GameEvent::SetMuted`), residual risks.

## 6. File ownership (Phase 4)

- **New**: `game/src/ui/{title,pause,touch}.rs`, `content/src/art/ui_meta.rs`.
- **Edit**: `game/src/state.rs` (GameMode::Title), `game/src/lib.rs` (mode
  routing, render), `game/src/ui/{mod,minimap,credits,hud,shop,dialog}.rs`,
  `game/src/save_data.rs` (+`muted`), `game/src/events.rs` (SetMuted),
  `content/src/text.rs` (+TextIds, binding_rows, act cards),
  `content/src/audio/sfx.rs` (+menu ids), `content/src/art/mod.rs` (bake reg),
  `crates/app/src/lib.rs` (SetMuted arm, boot_to_title),
  `engine/src/input/{touch,keyboard,gamepad,mod}.rs` (scoped per §2),
  `engine/src/audio.rs` (set_muted), `index.html` (M8).
- **Do not touch**: `game::{player,combat,enemies,boss,puzzle,rooms,items}`,
  `content::maps`, `engine::{chunks,atlas,render,save,time}` beyond reading.

## 7. Verification protocol

Per milestone: `cargo check --workspace --target wasm32-unknown-unknown` &&
`cargo clippy --workspace --target wasm32-unknown-unknown -- -D warnings` &&
`env -u NO_COLOR trunk build --release`. Browser smoke via
`python3 -m http.server 8090 --directory dist` + Playwright (desktop for 4A,
mobile emulation for 4B); kill the server + headless_shell afterwards.
Remember: never ship `trunk serve` output as dist.
