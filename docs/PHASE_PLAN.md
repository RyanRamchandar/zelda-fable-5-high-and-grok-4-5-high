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

**Acceptance criteria**
1. Gray-box arena: kill waves of 3 enemy families with the full verb set; combo,
   charge-spin, beam, shield, perfect block, dash all work per spec numbers.
2. Juice checklist (§2) fully present; hitstop/knockback/damage numbers/callouts.
3. Energy + style systems live and readable; fairy fountain object restores both.
4. Player sprite sheet passes in-motion QA at gameplay scale.
5. 60 fps on a mid laptop; all SFX distinct.

**Ownership:** `game::player`, `game::combat`, `game::fx` = worker A;
`game::enemies`, `game::ui` (HUD), `content::art` = worker B (if parallel).
Shared seams (`world`, events) land in worker A first.

---

## Phase 2 — Ambitious Act 1 overworld

**Goal:** The 240×240 contiguous overworld (GAME_DESIGN §4): all six named regions +
connective terrain, chunked tile rendering, smooth camera, collision, region banners,
NPCs/dialogue, shop, fairy fountains, gems as pickups with their region puzzles/fights,
overworld minimap with fog + POIs, secrets (bomb walls, caves, heart pieces), full
Act 1 enemy roster placed, interiors (houses/shop/caves) as room maps, save
checkpoints at gems.

**Acceptance criteria**
1. Walk village → all six regions with zero loading/screen transitions outdoors; 60 fps.
2. Three gems obtainable via their designed challenges; shrine door opens with all three.
3. Minimap fog-reveals; objective marker tracks gem/shrine progression.
4. Shop buy/sell works; rupee economy live; ≥8 secrets placed and telegraphed.
5. Every region visually distinct (palette/props) and every interactable legible.

**Ownership:** `content::maps` + `content::art` (tiles/props) = worker A;
`game::puzzle`, NPC/dialogue/shop in `game::ui`+`items` = worker B; minimap = B.

---

## Phase 3 — Dungeon, puzzles, boss (Gate B)

**Goal:** Triforce Shrine dungeon (GAME_DESIGN §5): room system + transitions,
dungeon minimap with reciprocity, keys, Gale Boomerang item + full curriculum
(crystals, gates, ordered-seal rooms), ironshell miniboss, **Granite Warden** with all
3 phases + cinematic intro, credits stub, victory state, checkpoints. B-item cycling
with bombs + boomerang.

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
