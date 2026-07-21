# PHASE_PLAN.md — Phased Execution Plan

Gates (DECISIONS §7): Gate A = end of Phase 0. Gate B = end of Phase 3. Gate C = end
of Phase 5. Workers run autonomously inside a phase. Every phase ends with the game
**building and playable** (`cargo check` clean, `trunk build --release` succeeds,
commit on `main`).

Global invariants for every phase:
- No file >~600 lines; respect ownership map in ARCHITECTURE §11.
- No new crates without a DECISIONS.md entry.
- Commit at each acceptance criterion, message prefixed `phaseN:`.

---

## Phase 0 — Scaffold (Gate A)

**Goal:** Whole pipeline proven end-to-end: Rust workspace → WASM → canvas stub with a
moving square, all three input backends, one synth beep, save roundtrip, Netlify-ready
`dist/`. Full spec: `WORKER_BRIEF_PHASE0.md`.

**Acceptance criteria**
1. `trunk serve` shows 480×270 letterboxed canvas; a 16×24 rect moves at fixed 60 Hz
   via keyboard, gamepad, and touch joystick.
2. Attack button triggers a WebAudio beep (context unlocked on first gesture).
3. Position persists across reload via localStorage.
4. `cargo check`/`clippy` clean; `trunk build --release` emits working `dist/`.
5. Git repo with clean history; README with run instructions.

**File ownership:** everything (single worker, greenfield).

---

## Phase 1 — Core loop: move / combat / juice

**Goal:** The game *feels* great in a gray-box arena before any real content. Player
kit complete (GAME_DESIGN §1–3): movement feel, full sword kit, shield/perfect block,
dash, energy meter, style ranks, damage pipeline with all juice items (§2 checklist),
3 enemy families (slime, bat, octorok), pickups, HUD (hearts, energy, style chip,
B-item), atlas pipeline with first real sprites (player full sheet, 3 enemies, tiles
for a test arena), SFX set for all combat verbs, debug sprite-motion viewer.

Split into two **sequential** sub-phases (briefs are authoritative):

- **Phase 1A** (`WORKER_BRIEF_PHASE1A.md`): shared seams first — `game::math`,
  `game::world/{mod,entity,physics,camera}`, `WorldEvent` queue, rewritten
  `game::lib` system order, `content::maps::arena` gray-box `MapDef` — then the
  full player kit, damage pipeline + juice, energy/style, pickups/fountain,
  functional rect HUD, SFX seam (`content::audio::sfx` specs + `engine::audio`
  synth + `app` adapter), target dummies, F1 debug overlay. Extends existing
  `engine::render::Draw` (camera offset, line) and `engine::input` (debug keys).
- **Phase 1B** (`WORKER_BRIEF_PHASE1B.md`, starts only after 1A lands): creates
  `engine::atlas` (does not exist in the scaffold yet) + `Draw::sprite`,
  `content::art/*` (palette + grids: player, enemies, tiles, ui),
  `game::assets` bake glue, `game::enemies/*` (slime, bat, octorok, rocks,
  wave director), F2 sprite-motion viewer, HUD sprite skin, enemy SFX.

**Acceptance criteria**
1. Gray-box arena: kill waves of 3 enemy families with the full verb set; combo,
   charge-spin, beam, shield, perfect block, dash all work per spec numbers.
2. Juice checklist (§2) fully present; hitstop/knockback/damage numbers/callouts.
3. Energy + style systems live and readable; fairy fountain object restores both.
4. Player sprite sheet passes in-motion QA at gameplay scale (F2 viewer).
5. 60 fps on a mid laptop; all SFX distinct.

**Ownership:** worker A (1A) = `game::{lib,math,world,player,combat,fx,items,ui
(functional),enemies (stub)}`, `content::{maps,audio::sfx}`, small `engine`
render/audio/input extensions, `app` event routing. Worker B (1B) =
`engine::atlas` + `Draw::sprite`, `content::art`, `game::{enemies,assets}`, HUD
skin, debug viewer. Seam contract frozen at 1A completion (WORKER_NOTES entry).

---

## Phase 2 — Ambitious Act 1 overworld

**Goal:** The 240×240 contiguous overworld (GAME_DESIGN §4), built in two
**sequential** sub-phases (briefs are authoritative):

- **Phase 2A** (`WORKER_BRIEF_PHASE2A.md`) — foundation: `MapDef` v2 (ground/
  detail/overhang layers, collision flags incl. water + one-way ledges, spawns/
  triggers/regions/entries, tile catalog, painter helpers), core terrain art,
  chunk-cached rendering in `engine::chunks` (LRU budget, dirty invalidation,
  animated-tile overdraw), all six region terrain shells + river/bridges/roads,
  Minish-Cap camera (dead-zone + eased lookahead), map switching with fade
  transitions to interior stubs (6 houses, shop, 2 caves), distance-based
  spawner, region banners, checkpoints, save v2 (map/entry/checkpoint/gems/
  flags), debug F3 map-cycle + F4 teleport. Arena kept behind F3.
- **Phase 2B** (`WORKER_BRIEF_PHASE2B.md`, starts only after 2A lands) —
  content fill: props + region-distinct decoration, furnished interiors,
  Interact verb + dialogue box + NPC/sign stubs, chests + rupee counter,
  three gems as guarded objectives (grove pedestal, camp guard-group chest,
  ruins pedestal) with the 3-gem shrine-door soft gate, authored encounters
  from the Phase 1 roster, overworld minimap (corner + pause, fog persisted,
  POIs, objective marker), 10 telegraphed secrets + 3 heart pieces.

- **Phase 2C** (`WORKER_BRIEF_PHASE2C.md`, starts only after 2B landed —
  which it has at `ea4edd9`) — interactivity depth, split into two
  **sequential** parts inside one brief:
  - **2C-A** — `game::puzzle` tile-interaction layer (`content::puzzles`
    site data), the real gem gates: grove wind-chime curriculum (2 taught
    single-chime gates + 3-chimes-in-one-window finale sealing the Courage
    Gem) and ruins plate-and-block court (2 plates + 2 pushable blocks
    gating the Wisdom Gem), bombs + B-item seam (tap-Item fires, hold-Item
    still shields, Q/LB-RB cycles; `AttackKind::Bomb`), grove bomb wall →
    `Cave(2)` rupee cache, barricade destructibility, shop economy UI
    (bombs 10₹×5, bomb bag 100₹, **heart piece #4** 200₹, tunic teased),
    broken-bridge crank shortcut. All puzzles solvable with the current kit
    (no boomerang until Phase 3 — puzzle hit-checks are tool-agnostic so
    the boomerang retro-enables one-throw solves for free).
  - **2C-B** — four new enemy families (raider spear, raider torch, wisp,
    skeleton) with art + AI + SFX, camp 3-wave war-chest battle via chained
    spawner groups (41→42→43) behind breakable barricades upgrading the
    Power Gem gate, region encounter re-dress (camp reads raider, ruins
    gains wisps/skeletons), difficulty/feel pass.

**Acceptance criteria**
1. Walk village → all six regions with zero loading/screen transitions outdoors; 60 fps
   via chunk cache (2A).
2. Interiors enter/exit with fades; death/reload → last checkpoint; save v2 roundtrips (2A).
3. Soft critical path: elder quest → 3 guarded gems → shrine door opens (2B).
4. Minimap fog-reveals and persists; objective marker tracks gem/shrine progression (2B).
5. Every region visually distinct and every interactable legible; ≥8 secrets
   placed and telegraphed; encounters match region identity (2B).
6. All three gems behind their real mechanisms: chime finale, 3-wave camp
   battle, plate court; puzzle states reset on reload, persist when solved (2C).
7. Bombs + shop economy live: purchases persist, heart piece #4 completes the
   max-heart set, bomb wall and bridge crank open permanently (2C).
8. New families telegraphed and distinct; ≤12 active AI; ~60 fps in camp
   wave 3 (2C).

**Ownership:** 2A = `content::maps` v2 + terrain art + `engine::chunks` +
`game::{state,world::spawner,draw_world,physics,camera,save_data,ui::banner}`.
2B = placement/props/NPC art + `content::text` + `game::ui::{dialog,minimap}` +
interact/chests + spawner groups + flag registry. 2C-A = `game::{puzzle,
items::bombs,ui::shop}` + `content::{puzzles,art::props_puzzle}` + catalog
ids 260–279 + flags 90–98 + `engine::input` BUTTON_CYCLE only. 2C-B =
`game::enemies::{raider,wisp,skeleton}` + `content::art::enemies_act1b` +
spawner wave-chain + placement re-dress. All sequential — each part edits
files the previous one owns; no parallel runs anywhere in Phase 2.

---

## Phase 3 — Dungeon, puzzles, boss (Gate B)

**Goal:** Triforce Shrine dungeon (GAME_DESIGN §5): room system + slide
transitions, dungeon minimap with reciprocity, keys, Gale Boomerang item + full
curriculum (crystals, gates, carry-a-flame, ordered-seal rooms), Ironshell duo
miniboss, **Granite Warden** with all 3 phases + cinematic intro, credits stub,
victory state, checkpoints. B-item cycling with bombs + boomerang.

Built in two **sequential** sub-phases (briefs are authoritative, written
against 2C's real code — the tool-agnostic `game::puzzle` hit layer,
`BUTTON_CYCLE` seam, `skeleton::stagger` hook, `TUNIC_UNLOCKED` flag, and the
ruins far-switch preview site):

- **Phase 3A** (`WORKER_BRIEF_PHASE3A.md`) — dungeon + tool: `MapId::Dungeon`
  (one contiguous MapDef; codec 4), `game::rooms` (room rects, camera lock,
  slide transitions, tile doors/shutters), `game::items::boomerang`
  (throw/return flight, pass-through enemy hits, generic `enemies::stun` +
  skeleton stagger, `StyleVerb::BoomerangStun`, B-item slot 2, and a new
  projectile arm in `puzzle::process_hits` so overworld chimes/cranks/
  barricades retro-enable with zero special-casing), Hall of Trials shutter
  gauntlet → boomerang chest, Hall of Currents crystal curriculum +
  carry-a-flame, two ordered seal rooms, 2 small keys + boss key on clued
  paths, dungeon checkpoints 7/8 (+ dungeon boot-from-save), discovered-room
  dungeon minimap driven by the same `rooms()` table that paints the doors
  (reciprocity by construction + load assert). Sanctum Core + Guardian Arena
  ship dressed-but-dormant (sign stubs, `GRP_DNG_SANCTUM=94` locked). Catalog
  280–309, flags 100–139.
- **Phase 3B** (`WORKER_BRIEF_PHASE3B.md`, starts only after 3A's completion
  entry) — climax + victory: Ironshell duo miniboss (front armor;
  boomerang-behind / perfect-block / stun answers) in the Sanctum Core,
  Guardian Arena dressing + pre-boss checkpoint 9, Granite Warden
  (entity boss + two WindCrystal entities + PebbleCrawlers; crystal
  prime-both-within-window → gale stagger → 4 s core windows; phases at
  75/35 with orbiting crystals, arm sweep, rim crumble, 5-way fan, crystal
  swaps, fake-core flash; boss-only segmented HP bar; cinematic intro with
  name plate; `HITSTOP_BOSS_BREAK` finally used), defeat → heart container +
  Shard of Courage → credits stub → village return beat, `TUNIC_UNLOCKED`
  shop row live, Act 1 victory predicate (`SHARD_OF_COURAGE` flag) for
  Phase 4 chapter select. Catalog 310–319, flags 140–149.

**Acceptance criteria**
1. Full critical path: New Game → 3 gems → shrine → boomerang → 2 seals → Warden →
   credits, no console errors, no soft-locks (all puzzle states resettable).
2. Boomerang matters in exploration (overworld cranks/chimes retro-enabled), combat
   (stun), and puzzles (crystal routes).
3. Warden requires the crystal mechanic (no DPS cheese); all attacks telegraphed;
   phase 3 fake-core beat works.
4. Dungeon minimap exits match room topology exactly.

**Ownership:** 3A = `game::{rooms,items::boomerang,puzzle::dungeon,
ui::dungeon_map,events}` + `content::{maps::dungeon,puzzles_dungeon,
art::tiles_dungeon/props_dungeon/items}` + headroom extractions
(`world::entity_data`, `events.rs`). 3B = `game::{boss,enemies::ironshell,
ui::credits}` + `content::art::boss` + victory wiring. **Sequential only** —
3B edits arena rooms, events routing, and combat arms that 3A owns; do not
run them concurrently.

---

## Phase 4 — Meta: title / pause / save / touch / gamepad

**Goal:** Title screen, chapter select (Act 1 + locked 2–3 cards), pause/help overlay
(objective + all three binding sets), full save/checkpoint UX (continue, chapter
restart, rupee carry), touch controls skinned + mobile-verified (layout, thumb reach,
no browser gestures), gamepad fully mapped incl. menus, credits polish, R-to-skip.

One brief (`WORKER_BRIEF_PHASE4.md`, written against real post-3B code), two
**sequential** parts for one worker:

- **4A — Meta shell** (M1–M5): `GameMode::Title`, `ui::{title,pause}` (new),
  pause routing pulled out of `MinimapState::update`, chapter select with
  rupee-carry restart, New Game erase-confirm, Continue (incl. mid-dungeon
  checkpoint 7/8/9 boot), settings (`muted` in SaveGame v2 via serde default,
  `GameEvent::SetMuted`, `Audio::set_muted`), credits R-to-skip (KeyR/pad
  Back → `BUTTON_CONFIRM`), Help page = objective line + 3-column bindings +
  live input echo.
- **4B — Touch v2 + mobile + gamepad** (M6–M9, starts only after 4A commits):
  touch backend rewrite to a data-driven button table (Attack/Item/Dash/
  Interact/Cycle/Pause + floating joystick), `TouchOverlay` v2 + `menu_tap`,
  skinned `ui::touch` (Item disc doubles as B-item readout — fixes the
  450,240 HUD overlap), menu tap targets, `index.html` mobile CSS
  (dvh/overscroll/callout) + portrait-rotate scrim, gamepad hardware
  checklist + Help-page input echo as on-screen tester.

**Acceptance criteria**
1. Cold load → title → New Game/Continue/Chapter Select all function.
2. Entire game completable with gamepad alone and with touch alone (verify in
   browser with emulated touch + real device if available).
3. Pause shows current objective + bindings for all three input methods.
4. Reload mid-dungeon resumes at last checkpoint with correct flags/rupees.

**Ownership:** 4A = `game::{state,ui::{title,pause},save_data,events}` +
`content::{text,art::ui_meta,audio::sfx appends}` + one-line keyboard/gamepad/
audio engine touches. 4B = `engine::input::touch` (+ `InputState` additions),
`game::ui::touch` (new), `index.html`. Sequential — 4B skins surfaces 4A
creates; no gameplay-module edits anywhere in Phase 4.

---

## Phase 5 — Polish + Netlify (Gate C)

**Goal:** Ship it. One brief (`WORKER_BRIEF_PHASE5.md`), one worker, milestones
strictly ordered so the deploy contains everything:

- **M1 feel pass** — pay the WORKER_NOTES residual-risk debts (perfect-block
  window/readability, camp wave 2/3 spike, Warden telegraphs + fake-flash
  teach + cp9 retry, seal throw tolerance, tap-bomb vs hold-shield), tunic
  cosmetic palette swap (3B deferral), ambient leaves/embers/fountain
  particles (2B deferral, capped ≤24), F2 sprite QA sweep in motion.
- **M2 music** — `engine::audio` chiptune sequencer (2 pulse + triangle +
  noise, WebAudio lookahead scheduling from the rAF closure),
  `content::audio::music` six tracks (Title/Village/Overworld/Dungeon/Boss/
  Victory), `GameEvent::SetMusic` + `game::music_director`, SFX distinctness
  sweep under music.
- **M3 perf** — camp wave 3 / Currents / Warden P3 at ~60 (floor 55); levers:
  particle caps, animated-tile coarsening, chunk budget, ≤12 active AI law.
- **M4 validation sweep** — full New-Game→credits keyboard playthrough +
  puzzle-reset audit + reciprocity assert + touch emulation spot-check +
  save-matrix (old/absent/corrupt), screenshots archived.
- **M5 README final** (+ `docs/media/` screenshots) → **M6 Netlify prod
  deploy** to locked slug `zelda-fable-5-high-and-grok-4-5-high` (immutable
  cache headers for hashed wasm/js; `netlify login` escalation protocol) →
  **M7 GitHub push** (`gh auth` escalation; dashed repo name) → **M8 Gate C
  completion entry**.

**Acceptance criteria**
1. Full Act 1 playthrough recorded clean in a real browser (keyboard, then touch spot-checks).
2. Music + ambient audio in all areas; every prompt §12 cue present and distinct.
3. `netlify deploy --prod --dir dist` live at slug
   `zelda-fable-5-high-and-grok-4-5-high`; live URL loads and plays on desktop +
   iPhone Safari (device pass documented as owed if no hardware).
4. Git history clean; push to GitHub (escalate for credentials if needed).

**Ownership:** single worker; feel-pass edits inside `game::{player,enemies,
boss,puzzle,fx,combat::tuning}` are tuning/readability only — map geometry,
room topology, id allocations, and Phase 4 menu flows are frozen.

---

## After Gate C (deferred)

Acts 2–3 built to the contracts in GAME_DESIGN §9, reusing systems: new overworld
maps, Storm Hookshot + Tempest Sentinel, Quake Gauntlets + Molten Colossus, chapter
preloading. Planner writes `WORKER_BRIEF_ACT2.md` at that point.
