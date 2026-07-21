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

**Goal:** Triforce Shrine dungeon (GAME_DESIGN §5): room system + transitions,
dungeon minimap with reciprocity, keys, Gale Boomerang item + full curriculum
(crystals, gates, ordered-seal rooms), ironshell miniboss, **Granite Warden** with all
3 phases + cinematic intro, credits stub, victory state, checkpoints. B-item cycling
with bombs + boomerang.

Planner writes the Phase 3 briefs **after 2C lands** — they must be authored
against 2C's real code: the `game::puzzle` hit-check layer (boomerang = a new
source of the same chime/crank/plate hit events, retro-enabling overworld
one-throw solves), `BUTTON_CYCLE` B-item seam (boomerang = selected_item 2),
the skeleton `stagger` hook (boomerang stun), the `TUNIC_UNLOCKED` shop flag
(post-boss), and the ruins far-switch bonus site.

**Acceptance criteria**
1. Full critical path: New Game → 3 gems → shrine → boomerang → 2 seals → Warden →
   credits, no console errors, no soft-locks (all puzzle states resettable).
2. Boomerang matters in exploration (overworld cranks/chimes retro-enabled), combat
   (stun), and puzzles (crystal routes).
3. Warden requires the crystal mechanic (no DPS cheese); all attacks telegraphed;
   phase 3 fake-core beat works.
4. Dungeon minimap exits match room topology exactly.

**Ownership:** `game::puzzle` + dungeon maps = worker A; `game::boss` +
`game::items::boomerang` = worker B.

---

## Phase 4 — Meta: title / pause / save / touch / gamepad

**Goal:** Title screen, chapter select (Act 1 + locked 2–3 cards), pause/help overlay
(objective + all three binding sets), full save/checkpoint UX (continue, chapter
restart, rupee carry), touch controls skinned + iPhone-verified (layout, thumb reach,
no browser gestures), gamepad fully mapped incl. menus, credits polish, R-to-skip.

**Acceptance criteria**
1. Cold load → title → New Game/Continue/Chapter Select all function.
2. Entire game completable with gamepad alone and with touch alone (verify in
   browser with emulated touch + real device if available).
3. Pause shows current objective + bindings for all three input methods.
4. Reload mid-dungeon resumes at last checkpoint with correct flags/rupees.

**Ownership:** `game::ui` + `game::state` = worker A; `engine::input::touch` polish +
mobile CSS/viewport = worker B.

---

## Phase 5 — Polish + Netlify (Gate C)

**Goal:** Feel/readability pass across everything: music tracks per area (village,
overworld, dungeon, boss, victory), ambient particles/lighting (lanterns, fountains,
leaves), boss-intro cinematics polish, difficulty tuning, sprite QA pass in motion,
perf pass (chunk cache, entity caps), validation sweep (prompt §13: reachability,
exit reciprocity, minimap consistency, puzzle resets, full critical-path playthrough
in a real browser incl. touch/gamepad, screenshots), README final, **deploy**.

**Acceptance criteria**
1. Full Act 1 playthrough recorded clean in a real browser (keyboard, then touch spot-checks).
2. Music + ambient audio in all areas; every prompt §12 cue present and distinct.
3. `netlify deploy --prod --dir dist` live at slug
   `zelda-fable-5-high-and-grok-4-5-high`; live URL loads and plays on desktop +
   iPhone Safari.
4. Git history clean; push to GitHub (escalate for credentials if needed).

**Ownership:** polish split by area, not by module; deploy = single worker at the end.

---

## After Gate C (deferred)

Acts 2–3 built to the contracts in GAME_DESIGN §9, reusing systems: new overworld
maps, Storm Hookshot + Tempest Sentinel, Quake Gauntlets + Molten Colossus, chapter
preloading. Planner writes `WORKER_BRIEF_ACT2.md` at that point.
