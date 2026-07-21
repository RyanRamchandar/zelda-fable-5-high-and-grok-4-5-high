# WORKER_BRIEF_PHASE3B.md — Ironshell duo, Granite Warden, Act 1 victory (Gate B)

You are a Grok worker. This brief is self-contained; follow it exactly. Conflict
order: `DECISIONS.md` → `ARCHITECTURE.md` → this brief → `GAME_DESIGN.md`.
You run autonomously — do not ask the human anything except for credentials.

**GATE: do not start until the "Phase 3A completion" entry exists in
`docs/WORKER_NOTES.md`.** Read it first — its frozen-seams list and the real
3A code win over this brief; log drift and adapt.

## 0. Mission

Phase 3A built the Triforce Shrine: rooms + slides, Gale Boomerang, curriculum,
keys, seals, minimap — with the **Sanctum Core** and **Guardian Arena** rooms
dressed but dormant behind sign stubs. You bring the dungeon to its climax and
close Act 1 (Gate B):

- **Ironshell duo** miniboss in the Sanctum Core (front-armored pair; stun by
  boomerang-to-the-back or perfect block) guarding the boss-key beat.
- **Granite Warden**: 48×48 three-phase boss with cinematic intro, boss-only
  segmented HP bar, and the **wind-crystal prime mechanic** — the boomerang is
  the only way to expose the core (no DPS cheese), per GAME_DESIGN §6.
- **Victory flow**: heart container + Shard of Courage, credits stub,
  return-to-village beat, Act 1 victory state + flags (chapter-select unlock
  data for Phase 4), `TUNIC_UNLOCKED` shop row goes live.

This completes the full critical path: New Game → 3 gems → shrine → boomerang
→ 2 seals → Warden → credits. **Gameplay feel is priority #1**: every Warden
attack telegraphed ≥30 ticks with distinct audio, phase breaks must feel
seismic (hitstop 12, shake, shards), and losing must never cost more than the
pre-boss checkpoint.

## 1. HARD DEPENDENCY — code reality (verify each before building)

Written against post-3A main. Verify each in code + the 3A completion entry:

1. **Rooms**: `game::rooms` owns room rects, camera lock, slides, shutter
   doors; `content::maps::dungeon::rooms()` is the topology table (minimap
   reciprocity is asserted against it — if you touch arena geometry, keep the
   table and tiles in lockstep). Sanctum Core + Guardian Arena rects, their
   exits, checkpoint 8, and the boss door are frozen 3A seams.
2. **Boomerang**: `EntityKind::Boomerang`, `BoomerangData { throw_id, phase,
   ... }`, `AttackKind::Boomerang` (pass-through, 0.5 dmg, stun arm in
   `apply_attack_hit`, `enemies::stun(world, id, ticks)` generic). Boss
   crystals must key off boomerang overlap using the same `throw_id` dedupe
   discipline.
3. **Stagger/stun hooks**: `skeleton::stagger`, generic `enemies::stun`,
   `DamagedPlayer { source: Option<EntityId> }` for perfect-block attribution
   (`apply_player_damage` perfect-block branch) — Ironshell uses both routes.
4. **Spawner**: `GRP_DNG_SANCTUM = 94` slots are placed + permanently locked
   by 3A. You unlock/activate them for the miniboss via the camp-chain
   pattern (`unlock_and_activate`, `GroupCleared` in `events.rs`). The Warden
   itself is NOT a spawner slot — spawn it directly on arena entry.
5. **Hitstop / juice**: `tuning::HITSTOP_BOSS_BREAK = 12` is reserved and
   unused — it's yours. `world.hitstop`, `camera.add_shake` (≤3 px),
   `FxKind` is open to additive variants, damage numbers/flash/knockback all
   flow through `apply_attack_hit`.
6. **HP bars**: normal enemies never show HP bars (GAME_DESIGN §2); the boss
   bar is a **new bottom-of-screen segmented bar** — no existing widget;
   `ui/hud.rs` has the consts style to follow. Render order: HUD after
   world/fx (see `Game::render`).
7. **Mode/state**: `GameMode::{Play, Transition}`; dialog/shop/pause-map pause
   the world via early-return gates in `Game::update`. Your intro/victory
   cutscenes follow the same pattern (a `CutsceneState` on `Game` or on the
   boss module gating input — do NOT add web-sys, timers only).
8. **Checkpoints**: gems 2/3/4, shrine 6, Vestibule 7, antechamber 8. You add
   **pre-boss checkpoint 9** (arena antechamber / just outside the boss door)
   with a matching dungeon `EntryPoint`; `check_player_death` handles the
   respawn. GAME_DESIGN §8 lists "pre-boss" as a required checkpoint.
9. **Save/flags**: additive `#[serde(default)]` only; prefer flags. 3A froze
   flags 100–139; **you own 140–149**. `TUNIC_UNLOCKED = 98` is reserved and
   the shop row is greyed with "After the shrine's trial." — you set the flag
   and wire the shop row live (`ui/shop.rs` stock table + rupee sink 300₹ +
   a cosmetic palette-swap on the player sprite is OPTIONAL — if cheap via
   the existing `PaletteSwap` bake path do it, else tunic = +1 max heart? NO:
   keep it cosmetic-or-nothing, log the choice; do not add defense stats).
10. **Credits**: none exist. `TextId` pages + a simple auto-scroll screen is
    yours; Phase 4 polishes it (R-to-skip is Phase 4 per PHASE_PLAN — ship a
    skip-on-Attack-hold as the stub behavior and log it).
11. **File caps**: post-3A sizes unknown — check `wc -l` first; expect
    `lib.rs`/`events.rs`/`draw_world.rs` near their limits. `game::boss` is
    a fresh module (ARCHITECTURE reserves it); keep `granite_warden.rs` under
    600 by splitting phase logic / rendering if needed.
12. **Catalog/art**: 3A used tiles 280–309; **310–319 are yours** (arena rim
    crumble tiles, core-glow dressing). Boss sprites: 48×48 body (multi-frame
    strips), crystal entities 16×16, pebble crawler 16×16 — new
    `content/src/art/boss.rs`, registered in `all_bakes()`; palette swap rows
    for the crystal blue/amber states if you reuse strips.
13. Toolchain: rustup `~/.cargo/bin` first; `env -u NO_COLOR trunk build
    --release`; kill smoke servers/browsers after every run.

## 2. Locked constraints

- Stack per DECISIONS.md; no new crates; no engine changes at all; `content`
  depends on nothing; `game` stays web-sys-free; art = handcrafted grids.
- Ownership: boss logic in `game/src/boss/` (new: `mod.rs`,
  `granite_warden.rs`, split further as needed); Ironshell in
  `game/src/enemies/ironshell.rs` (it's an enemy family, not a boss);
  victory/credits UI in `game/src/ui/credits.rs`; boss art in
  `content/src/art/boss.rs`.
- **Tool gate is non-negotiable**: the Warden's core is exposable ONLY via
  the crystal mechanic. Sword/bomb/beam damage while the core is hidden = 0
  (clank + spark + refusal cue, like the skeleton shield). No damage path may
  bypass it.
- No file >~600 lines. Commits on `main`, prefixed `phase3b:`, small and
  frequent. No push, no deploy. Every commit: wasm32 `cargo check` + `clippy
  -- -D warnings` clean; `trunk build --release` at milestones.
- Tuning ±30% with WORKER_NOTES log. Death during the boss → checkpoint 9
  with the arena fully reset (no partial boss HP persistence).

## 3. Milestones (commit at least once each)

### M1 — Ironshell duo (Sanctum Core miniboss)

`game/src/enemies/ironshell.rs` + art (24×24, heavy shell silhouette, in
`enemies_act1b.rs`-style or `boss.rs` — your call, log):

- **Family**: `SpawnKind::Ironshell` + `EntityKind::Ironshell` + data +
  exhaustive-match sweep (spawner, updates, integrate, resolve, render,
  is_enemy). HP ~8 each. Slow advance (~0.4), a lunging shell-bash
  (30-tick telegraph, `GuardClank`-family windup, damage 2), brief recovery.
- **Front armor**: sword/beam/boomerang from the front → refused (clank
  spark, no damage, small self-knockback for the player's swing rhythm).
  Vulnerable when: (a) hit from **behind** (boomerang's curved return makes
  this natural — the taught answer), (b) staggered by **perfect-blocking**
  its bash (`source` attribution → stagger, like skeletons), (c) generic
  `enemies::stun` (boomerang front hits do 0 damage but a BACK hit both
  damages and stuns — the two-crystal boss literacy starts here). Stagger
  ~75 ticks, shell-open sprite state, `SfxId::IronshellCrack` on real hits.
- **The duo fight**: entering Sanctum Core (boss key NOT required here —
  verify 3A's gating; the boss door is what the key opens) slams shutters,
  unlock+activate `GRP_DNG_SANCTUM` (2 Ironshells placed by 3A; if 3A placed
  none, add the 2 SpawnDefs). Two at once forces positioning: bait one's
  bash while circling the other. `GroupCleared(94)` → shutters open, the
  boss-key chest area is accessible (or trivially walkable if 3A left it
  open — reconcile with 3A's geometry, log), toast "THE CORE QUIETS",
  checkpoint save.
- Bombs: blast damages them from any side (2C precedent — bombs are earned
  ordnance) but does not stun; keep HP high enough that bombs alone are
  wasteful, not a cheese (log the math).
- SFX appends: `IronshellBash`, `IronshellCrack` (+ reuse clank/rattle).

DoD: duo fight is winnable with boomerang-behind or perfect-block play,
refuses frontal grinding, never soft-locks (shutters reopen on clear; death
resets the group via the locked/respawn rules — verify with a deliberate
death).

### M2 — Guardian Arena, cinematic intro, boss scaffolding

- **Arena prep**: Guardian Arena room (3A shell) gets its final dressing:
  ~13×11-tile open floor, rim tiles (310+) that can visually crumble in
  phase 3, two **crystal perches** at opposite edges (E/W), sealed-behind-you
  shutter on entry. **Pre-boss checkpoint 9** + `EntryPoint` in the
  antechamber outside the boss door.
- **Boss module scaffold** (`game/src/boss/mod.rs` + `granite_warden.rs`):
  `BossState` hung off `Game` (None outside the arena). The Warden is an
  entity (`EntityKind::GraniteWarden`, 48×48 sprite, 40×40 body, no
  knockback — massive) plus **two crystal entities**
  (`EntityKind::WindCrystal`, 16×16, invulnerable to damage, positions
  driven by the boss module) and transient `EntityKind::PebbleCrawler`
  minions (phase 2+; 16×16, HP 2, chase-and-lunge — reuse slime logic
  shape). Exhaustive-match sweep for all three.
- **Cinematic intro** (first entry only, flag `WARDEN_INTRO_SEEN = 140`):
  shutter slams → input locked → camera pushes to the Warden as it assembles
  from rubble (3–4 keyframe sprites, dust FX, rising rumble) → **name plate**
  "GRANITE WARDEN" (letterboxed bars + text, ~90 ticks) → camera returns,
  boss bar slides in, fight starts. Implement as a tick-scripted sequence in
  the boss module gating input (dialog-pause pattern). Re-entries after
  death: skip to a 30-tick short version (bar slide + roar).
- **Boss HP bar** (`ui/hud.rs`-style consts, render from boss module or a
  small `ui::boss_bar`): bottom-center segmented bar — 3 segments with
  notches at 75% and 35% (phase gates), fills drain within the current
  segment; boss name label. Only visible while the fight is live.
- **Core rule plumbing**: the Warden takes damage ONLY while
  `core_exposed` (see M3); otherwise all `AttackHit`s on it are refused
  (clank + zero) — implement in `apply_attack_hit` via a boss-module query
  or a `WardenData.core_exposed` check (keep combat's exhaustive style —
  log where the check lives).

DoD: walking through the boss door plays the intro once, spawns the Warden +
crystals + bar; the Warden is invincible and harmless (no attacks yet is
fine mid-milestone); death outside → checkpoint 9.

### M3 — The three phases (the fight)

All numbers start from GAME_DESIGN §6; tune ±30% and log. Every attack:
≥30-tick telegraph, distinct audio, readable wind-up sprite.

**Crystal mechanic (all phases)**: each `WindCrystal` is **primed** by a
boomerang overlap (throw or return leg; `throw_id` dedupe so one pass primes
once) — primed = lit sprite + hum + a draining 300-tick (5 s) timer shown as
a small ring. **Both primed concurrently** → GALE: wind FX sweeps the arena,
the Warden staggers (`HITSTOP_BOSS_BREAK`-adjacent feel: brief slow, shake),
**core exposed 240 ticks (4 s)** — core = glowing chest sprite state; sword
window (boomerang does its 0.5 pass-through too, but sword is the payoff).
Priming cues: `CrystalBlue/Amber` reuse or own `CrystalPrime` +
`GaleStagger` SFX. Unprimed-timer-expiry: soft fizzle cue (teaches pairing).
One or two throws can prime both (a through-line exists at fight start —
author perch positions so a single brave cross-arena throw can clip both).

- **Phase 1 (100→75%)**: slow **slam** (jump + crash → expanding shockwave
  ring the player dashes through or outruns; ring = radial `ENEMY_HIT`
  pulse, damage 2) and **rock-throw fan** (3 rocks, arcing, `OctorokRock`
  reuse or own kind, damage 1). Crystals static at the E/W perches. 3
  exposure windows ≈ one segment (core hits do ~segment/3 per window with
  decent play). Segment empties → **phase break**: `HITSTOP_BOSS_BREAK`,
  shake 3, armor shards FX, roar, bar notch locks.
- **Phase 2 (75→35%)**: crystals **orbit slowly** (drift along the rim,
  ~0.3 px/tick — leading the throw becomes the skill). Slam now also spawns
  **2 PebbleCrawlers** (they die to anything; they exist to punish pure
  kiting). New **sweeping arm**: half-arena horizontal sweep, telegraphed by
  a ground shadow line — the dodge is dash-through i-frames (GAME_DESIGN:
  "jump ↔ no"). Prime pairing window tightens to 210 ticks (3.5 s).
- **Phase 3 (35→0%)**: **arena rim crumbles** — outer ring tiles swap to
  crumble/pit tiles (SOLID edge, floor shrinks ~1–2 tiles all around;
  `set_tile` + debris FX + sustained rumble). Rock fan is 5-way; crystals
  **swap perch positions after each stagger** (teleport with a wind-poof —
  re-read before each pair). **Fake core flash**: 60–90 ticks after a
  stagger ends, the chest flashes WITHOUT a gale (no crystal prime) — hits
  during a fake flash clank (refusal cue) and the Warden counter-slams
  sooner: teaches "wait for the gale," per design. Keep at most one fake per
  exposure cycle; make the real gale visually unmistakable (screen-wide wind
  streaks vs a local glint).
- **Defeat**: final segment empties → collapse sequence: input locked,
  crumble-apart keyframes, white-out flash, `SfxId::WardenFall`, crystals
  shatter, silence beat → victory flow (M4). Set `WARDEN_DEFEATED = 141`.
- Player-death handling: full arena reset (boss despawned, `BossState`
  cleared, rim tiles restored — repaint from MapDef on `switch_map`, which
  the respawn already does), checkpoint 9 respawn, short intro variant.
- New SFX: `WardenSlam`, `WardenSweep`, `WardenRoar`, `RockFan`,
  `CrystalPrime`, `GaleStagger`, `CoreHit`, `PhaseBreak`, `FakeFlash`,
  `WardenFall`, `PebbleSkitter` (+ specs). Distinct per attack — the audio
  telegraph requirement is acceptance-critical.

DoD: full fight beatable with intended play (~4–8 min skilled), unbeatable
by DPS cheese (verified: whale on the closed core for 60 s → zero damage),
all telegraphs ≥30 ticks with unique audio, phase 3 fake-core beat observably
teaches waiting, death → clean reset.

### M4 — Victory flow, rewards, credits stub, Act 1 state

- **Reward beat** (in-arena, post-collapse): **Heart Container** pickup
  (max_hearts += 2, full heal — direct grant, not the 4-piece path; flag
  `WARDEN_HEART = 142`) with hold-up toast "HEART CONTAINER", then the
  **Shard of Courage** rises: item-get dialog
  (`TextId::ShardOfCourage`: the Act 1 macguffin, ties to the elder's
  quest), flag `SHARD_OF_COURAGE = 143`, `GemGet`-class fanfare.
- **Credits stub** (`game/src/ui/credits.rs` + `TextId::CreditsPages` or a
  `&'static [&'static str]` table in content): full-screen dark panel,
  auto-scroll (~0.4 px/tick) of title, "ACT 1 CLEAR", roles (Planner /
  Workers / Engine: handcrafted), "Acts 2–3 await…". Hold Attack ≥60 ticks
  skips (Phase 4 does R-to-skip polish). Runs as a `GameMode`-adjacent
  overlay pausing the world (dialog pattern).
- **Return-to-village beat**: credits end → fade transition to Overworld at
  the **village fountain** (new or existing entry; checkpoint back to a
  village id), toast "ACT 1 COMPLETE", elder gets a post-victory line
  (`TextId::ElderVictory` — new arm in `interact.rs` keyed on
  `SHARD_OF_COURAGE`), shrine door stays open, dungeon stays replayable
  (Warden re-fightable from the short intro if the player returns —
  `WARDEN_DEFEATED` skips the rewards on re-kill, log it).
- **Tunic goes live**: on `WARDEN_DEFEATED`, set `TUNIC_UNLOCKED = 98`;
  shop row purchasable (300₹, once, flag `TUNIC_BOUGHT = 144`); effect =
  cosmetic palette swap on the player sprite via the existing `PaletteSwap`
  bake (bake a `player_tunic` variant row; `render_entity` picks the key by
  flag) — if the swap-per-sheet plumbing exceeds ~an hour of work, ship the
  purchase + a "worn with pride" dialog and log the cosmetic as Phase 5
  polish. No stat changes.
- **Act 1 victory state** (Phase 4 contract): a single readable predicate —
  `SHARD_OF_COURAGE` in flags = Act 1 complete. Note it in WORKER_NOTES for
  the Phase 4 chapter-select worker. Save immediately at every beat
  (heart, shard, return).
- Full honest keyboard run for Gate B: New Game → elder → 3 gems → shrine →
  Trials → boomerang → Currents → seals → boss key → Ironshell duo → Warden
  (die at least once mid-phase-2 to verify checkpoint 9 + reset) → victory →
  credits → village. No console errors, no soft-locks. Screenshot the
  critical beats to `/tmp/p3b_smoke/`.
- WORKER_NOTES completion entry: landed/deviations/tuning (esp. boss
  numbers), Gate B readiness statement, what Phase 4 inherits (victory
  predicate, credits stub, checkpoint map, tunic state).

## 4. Definition of Done (3B / Gate B)

1. Ironshell duo: front-armored pair, boomerang-behind / perfect-block /
   stun answers all work; group-driven, reset-safe, no frontal-grind path.
2. Granite Warden: 3 phases per GAME_DESIGN §6 — crystal prime/pair → gale →
   4 s core windows; 75/35 breaks with heavy juice; orbiting crystals +
   pebbles + arm sweep (dash-through dodge) in P2; rim crumble + 5-way fan +
   crystal swaps + fake-core-flash in P3; core exposable ONLY via crystals.
3. Every boss/miniboss attack telegraphed ≥30 ticks with distinct audio;
   boss-only segmented HP bar; cinematic intro with name plate (short
   variant on retry).
4. Death during boss → pre-boss checkpoint 9, arena fully reset.
5. Victory: heart container + Shard of Courage → credits stub (skippable) →
   village return with post-victory elder + open world; tunic purchasable;
   all of it save-persistent and re-entrant.
6. Full critical path (acceptance #1 of PHASE_PLAN Phase 3) verified in a
   real browser end-to-end; boomerang demonstrably matters in exploration,
   combat, puzzles, AND the boss.
7. check/clippy/trunk clean; no file >~600; overworld/F3/dungeon 3A content
   unbroken; WORKER_NOTES completion entry written (Gate B).

## 5. File ownership (3B)

**Creates**: `game/src/boss/{mod,granite_warden}.rs` (+ splits),
`game/src/enemies/ironshell.rs`, `game/src/ui/credits.rs` (+
`ui/boss_bar.rs` if separate), `content/src/art/boss.rs`.

**Edits (additive)**: catalog 310–319, flags 140–149 (+ set 98), SFX
appends, TextId appends (`ShardOfCourage`, `ElderVictory`, credits copy),
`EntityKind::{Ironshell, GraniteWarden, WindCrystal, PebbleCrawler}` + data
+ match sweeps, `apply_attack_hit` core/armor refusal arms,
`GRP_DNG_SANCTUM` unlock wiring in `events.rs`, checkpoint 9 +
`EntryPoint`, arena room dressing in `content::maps::dungeon` (within 3A's
frozen rects), shop stock row (tunic), `interact.rs` elder arm, village
return entry, `lib.rs`/`state.rs` wiring for boss/credits modes.

**Must NOT touch**: `engine/*`, room/slide system internals, boomerang
flight/stun internals, puzzle APIs (overworld + dungeon), minimap/dungeon-map
internals beyond adding the arena/boss-door glyphs if trivial, shop internals
beyond the stock table, docs other than WORKER_NOTES.

## 6. Verification protocol

- Per milestone: wasm32 check + clippy, `env -u NO_COLOR trunk build
  --release`, Playwright smoke vs `python3 -m http.server 8090 --directory
  dist` (screenshots to `/tmp/p3b_smoke/`; localStorage flag assertions).
  Kill server + headless browser after every run.
- Boss-specific: DPS-cheese negative test (closed core, 60 s of attacks →
  HP unchanged); fake-flash negative test (hits during fake → zero + clank);
  pairing-timeout test (prime one, wait, prime other → no gale); death-reset
  test each phase; re-entry short intro test.
- Feel evidence: F1 fps during phase 3 (crumble + pebbles + FX worst case)
  ~60; human notes on telegraph readability and window generosity.
- Gate B statement in WORKER_NOTES: explicit "critical path clean" with the
  run's screenshot list.
