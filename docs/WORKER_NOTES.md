# Worker Notes (append-only)

## Phase 0 completion — 2026-07-21 (Grok worker)

### Verified
- `cargo check --workspace --target wasm32-unknown-unknown` — clean
- `cargo clippy --workspace --target wasm32-unknown-unknown -- -D warnings` — clean
- `trunk build --release` — `dist/` has `index.html` + hashed `.js`/`.wasm`
- Browser smoke (Playwright against `python3 -m http.server 8090 --directory dist`):
  - Canvas internal 480×270, CSS integer-scaled (960×540), letterboxed on black
  - ArrowRight moves the 16×24 white rect; position advances ~100px/sec
  - `localStorage["shard_save_v1"]` written every 60 ticks; reload restores x/y
  - TouchEvents: left-half joystick moves rect; attack-button pixel tint shows overlay after first touch
- Attack → WebAudio square beep wired (context resume on first key/pointer/touch)

### Gamepad
- Mapping implemented per brief (axes 0/1 + d-pad 12–15, buttons 0/2/1/3/9).
- **Gamepad untested on hardware** (no physical pad in this environment). Code-reviewed against standard layout.

### Deviations / notes
- `InputState` also carries `touch_active` + `touch_overlay` so `game::render` can draw the overlay without `web-sys` (brief’s `render(&self, d)` signature preserved).
- `GameEvent::Save(String)` added beside `Beep` for the periodic save seam (app forwards to `engine::save`).
- Trunk 0.21.14: `NO_COLOR=1` breaks CLI (`--no-color` expects true/false). Use `env -u NO_COLOR trunk …` in this environment.
- Do not use `trunk serve` output as the Netlify `dist/` — serve overwrites `dist/` with a live-reload index. Always re-run `trunk build --release` before deploy checks.
- Homebrew `rustc` was present; installed rustup stable + `wasm32-unknown-unknown` and `trunk` via cargo for the toolchain the brief requires.

### Gate A
Ready: scaffold proves render, fixed loop, keyboard + touch (+ gamepad code), audio unlock/beep, save roundtrip, Netlify-ready `dist/`.

## Phase 1 planning — 2026-07-21 (Planner, Fable 5)

- Gate A accepted at `f65e9f5`. Phase 1 split into two sequential briefs:
  `WORKER_BRIEF_PHASE1A.md` (seams + player kit + combat/juice/energy/style +
  functional HUD + SFX seam) then `WORKER_BRIEF_PHASE1B.md` (atlas pipeline +
  sprites + slime/bat/octorok + wave director + HUD skin + F2 viewer).
  1B is blocked on 1A's completion entry here — the seam contract is
  PHASE1A §5 and freezes when 1A lands.
- Briefs were written against the *actual* scaffold, not the ARCHITECTURE tree:
  `engine::atlas` does not exist yet (1B creates it); `Draw` is rect/circle/
  text-only (1A adds camera offset + line, 1B adds sprites); `InputState`
  buttons have held/pressed only; `GameEvent::{Beep,Save}` is replaced by
  `{Sfx(SfxId),Save}` in 1A (app match updated accordingly).
- Content→engine SFX/art data flows through tiny adapters (app for sfx specs,
  game::assets for art decode) to preserve the dependency law
  (`content` depends on nothing, `engine` never depends on `content`).
- Shield is temporarily on the Item button (K / pad 2) until B-items arrive in
  Phase 2; planner will re-map when boomerang/bombs land.
- PHASE_PLAN.md Phase 1 section updated with the 1A/1B split and real paths.

## Phase 1A completion — 2026-07-21 (Grok 4.5 High Fast worker)

### What landed
- **M1 seams**: `game::math`, `game::world` (arena, EntityId generational store, physics AABB sweep, camera follow+shake≤3, `WorldEvent` queue, hitstop), `Draw::set_offset` / `line`, system order in `game::lib`.
- **M2**: `content::maps::MapDef` + `arena()` 60×34 gray-box with pillars + fountain corner; tile ids FLOOR/WALL/FOUNTAIN.
- **M3–M4**: Player 8-way accel/decel (vector lerp toward target velocity over ~10 ticks — crisper than per-axis), dash i-frames 2–8, slash→backslash→finisher with 6-tick buffer + 20-tick combo window, hold-charge≥30 → spin (20 energy), sword beam at full hearts, shield on **Item (K)** with perfect-block window 6 ticks.
- **M5–M6**: `combat::resolve_hits` + damage/knockback/hitstop/flash, fx particles/numbers/arcs/toasts, energy meter, style D→S, pickups+magnetism, fairy fountain zone, 3 target dummies (20hp, respawn 120).
- **M7–M8**: plain-rect HUD, SFX seam (`content::audio::sfx` → app adapter → `engine::audio::play`), debug F1 overlay + H projectile (when overlay on), F2 reserved.

### 1B seam contract status (PHASE1A §5)
| # | Seam | Status |
|---|---|---|
| 1 | `World` spawn/despawn/get/get_mut/iter, `events`, `hitstop`, `rng` | **stable** |
| 2 | `Entity`/`EntityKind`/`EntityData` exhaustive matches | **stable** — 1B adds Slime/Bat/Octorok/OctorokRock |
| 3 | `WorldEvent` set | **stable** (`AttackHit` carries damage/knockback/source; `DamagedPlayer` has dir) |
| 4 | `combat::resolve_hits` | **stable** — enemies hurt player via `ENEMY_HIT` overlap → `DamagedPlayer`, or push `DamagedPlayer` directly (see `combat/mod.rs` module docs) |
| 5 | `physics::move_entity` + `query_aabb` / `aabb_overlap` / `circle_hits_entity` | **stable** |
| 6 | `AnimState { sheet, frame, timer }` | **stable** — walk cycles 0..3 @ 8 ticks |
| 7 | `MapDef` + arena tile ids | **stable** |
| 8 | `SfxId`/`SfxSpec`/`spec()` + app adapter | **stable** — 1B appends enemy ids |
| 9 | `ui::hud` layout consts + `fx::push_toast` | **stable** — 1B may restyle toasts into ui |
| 10 | Render order: tiles → y-sort ents → fx → HUD; `set_offset` | **stable** |
| 11 | Debug indices F1/F2/H | **stable** — F2 unused hook for sprite viewer |

### Deviations / planner notes
- Shield remapped to Item button (already planned).
- Movement accel: **vector lerp** toward target vel (rate 1/10 per tick), not separate per-axis ramps — logged as feel choice within ±0.
- Default spawn moved to `(480, 300)` open field (center pillar would trap the body); pillar at `(28,14)` instead of `(29,16)`.
- `GameEvent::Beep` removed → `Sfx(SfxId)`.
- `AttackKind::DebugShot` reserved; debug shots use `EntityKind::DebugShot` + `BeamData`.
- `fastrand` added to `game` (pre-approved).

### Verification
- `cargo check --workspace --target wasm32-unknown-unknown` — clean
- `cargo clippy --workspace --target wasm32-unknown-unknown -- -D warnings` — clean
- `env -u NO_COLOR trunk build --release` — `dist/` ok
- Playwright smoke vs `python3 -m http.server 8090 --directory dist`:
  - Canvas + HUD (hearts, energy, style D, B-slot "—") render
  - Hold A moves; `localStorage shard_save_v1` wrote `x≈170` (moved from 480)
  - J×3 / charge / dash exercised; at least one dummy cleared in screenshot
  - F1 pressed (overlay text may be subtle at scale)
- Feel: iterate locally for hitstop/combo mash; numbers match GAME_DESIGN starting values (no ±30% needed yet).

### Gate readiness for 1B
**YES** — seams frozen as above. Residual risks: perfect-block / reflect only verified via code path + H-key hook (needs human feel pass); gamepad still untested on hardware; toast→ui move is 1B's call.

## Phase 1B completion — 2026-07-21 (Grok 4.5 High Fast worker)

### What landed
- **M1 atlas**: `content::art::palette` (~48 colors + char map + `PaletteSwap`), `SpriteDef`/`all_bakes()`, `engine::atlas` shelf-pack bake (hidden `#atlas` canvas — required for Chromium `drawImage`), `Draw::set_atlas`/`sprite`/`sprite_scaled`, `game::assets::bake` decode glue. App bakes once at start → `draw.set_atlas` + `Game::new(..., sprites)`.
- **M2 sheets**: programmatic Minish-Cap-ish grids in `player_base`/`player_actions`/`enemies`/`tiles`/`ui` (left = `flip_x` of right). F2 sprite viewer in `game::ui::viewer` (pauses world; 1×/3×/flip; sheet cycle + J rate). Dummies use `slime_dummy` palette-swap.
- **M3 enemies**: slime chase/lunge, bat sine swoop, octorok spit+hide (sword-immune while hidden), `OctorokRock` reflectable via perfect block (projectile kept alive through damage route). Wave director W1–W3+ escalate, 45-tick telegraph, CLEAR rupee bonus. Enemy SFX ids appended (adapter covers automatically).
- **M4 HUD**: atlas hearts/energy frame/style chip/item slot/toast panel; layout consts preserved. Player death → fountain respawn 3 hearts + "FAIRY RESCUE" toast (1A left death open).

### Seams consumed
All 1A frozen seams honored. Additive only: `EntityKind::{Slime,Bat,Octorok,OctorokRock}`, matching `EntityData`, enemy tuning rows, SFX ids. `resolve_hits` extended for rocks; hostile projectiles no longer despawn before shield/reflect can run.

### Deviations / tuning
- Drop table: nothing chance 70%→55%, energy more common (wave combat starve risk) — logged.
- Atlas canvas must be DOM-attached (`display:none`); detached canvas drew nothing under Playwright Chromium.
- Debug F1/F2/H use `debug_pulse` latch so sub-frame key taps still toggle (Playwright + quick taps).
- Wave alive count includes telegraphing spawns (body=None) so CLEAR doesn't fire instantly.
- Art authored via generative pixel painter → reviewed grids (not AI image dumps); feet pinned on walk cycles; polish is cleanline-readable but not final production Minish Cap quality.

### Verification
- `cargo check` + `clippy -D warnings` (wasm32) clean (use rustup `~/.cargo/bin` ahead of Homebrew rustc).
- `env -u NO_COLOR trunk build --release` → `dist/` ok.
- Playwright vs `python3 -m http.server 8090 --directory dist`:
  - Floor/player/dummy sprites + HUD hearts render
  - F2 viewer shows `player_idle` 1×/3×/flip over floor samples
  - Save roundtrip (`shard_save_v1` x advances)
  - Wave director toasts observed earlier; full 3-wave human feel pass still recommended

### DoD checklist
1. Atlas bake + sprite draw — yes  
2. Player sprited + F2 in-motion — yes (walk footing pinned; further polish welcome)  
3. Three families + telegraphs + combat pipeline — yes  
4. Rock reflect + hide immune — yes (code + seam; human perfect-block feel still advised)  
5. Waves escalate + CLEAR bonus — yes  
6. HUD skinned — yes  
7. Distinct enemy/wave SFX — yes  
8. Perf ~60 with small wave — F1 present; no hitch observed in smoke  
9. check/clippy/trunk; files <600 — yes  
10. This completion entry — yes  

### Phase 1 overall gate (ready for Phase 2 brief?)
**YES** — with residual human playtest: perfect-block rock reflect feel, gamepad hardware, longer wave-3+ survival, art polish pass if planner wants higher bar.

## Phase 2 planning — 2026-07-21 (Planner, Fable 5)

- Phase 1 (1A+1B) accepted at `6bf033e`. Phase 2 split into two sequential
  briefs written against the *actual* code, not the ARCHITECTURE ideal:
  `WORKER_BRIEF_PHASE2A.md` (overworld foundation) then
  `WORKER_BRIEF_PHASE2B.md` (content fill). 2B is blocked on 2A's completion
  entry here; the 2B seam contract is PHASE2A §5 and freezes when 2A lands.
- Key reality deltas the briefs encode: `MapDef` is v1 (single ground layer +
  bool collision, no spawns/triggers) — 2A replaces it wholesale (only
  permitted breaking seam; compiler-walked). `game::state`/GameMode does not
  exist yet — 2A creates it for fade transitions. `draw_world::render_map` is
  a hardcoded per-tile match — 2A rewrites it data-driven + chunk-cached
  (`engine/src/chunks.rs`, new). `SaveGame` is `{x,y}` only — 2A ships v2
  (map/entry/checkpoint/gems/flags); v1 saves fall back to New Game
  (accepted). `WaveDirector` currently runs unconditionally — 2A scopes it to
  `MapId::Arena` behind debug F3.
- Scope call: PHASE_PLAN's old Phase 2 bundled gem puzzles, shop economy, and
  new enemy families. Human priority is the ambitious map itself, so Phase 2
  is now 2A (acreage + systems) + 2B (locations, NPC/sign stubs, guarded-gem
  soft critical path, minimap, secrets) with a deferred **Phase 2C** (chime/
  plate puzzles, camp wave battle, shop UI, wisp/skeleton/raiders, bomb
  walls) briefed after 2B. 2B places every 2C location + flag so 2C is
  swap-in. PHASE_PLAN.md Phase 2 section updated accordingly.
- Perf posture: 240×240×3 layers unchunked is ~1.5k drawImage/frame (iPhone
  risk), so chunk cache is a 2A hard requirement with an LRU budget (~48
  chunks ≈ 12 MB — full-map cache would be ~170 MB, not acceptable), a
  2-bakes-per-frame amortization rule, animated tiles as post-blit overdraw,
  and a direct-draw fallback. Spawner uses distance activation/sleep so
  placed-enemy count (~60–80 defs) never means 60 live AIs.
- Sequential only: 2B edits region modules and game files 2A owns; there is
  no zero-contention parallel split. Do not run 2A and 2B concurrently.

## Phase 2A completion — 2026-07-21 (Grok 4.5 High Fast worker)

### What landed
- **M1 MapDef v2**: layered `ground`/`detail`/`overhang`, `collision: Vec<u8>`
  flags (SOLID/WATER/LEDGE_*), spawns/triggers/regions/entries,
  `content::maps::build(MapId)`, `catalog::tile_info` + id ranges, `paint.rs`
  helpers (path/blob/scatter/stamp/river), arena ported under `MapId::Arena`.
- **M2 terrain art + render + collision**: `tiles_terrain` / `tiles_water` /
  `tiles_forest` (+ palette W–Z dirt/ash/sand/wood), data-driven
  `draw_world` via `TileSprites`, `PlayerState::LedgeHop`, water/ledges in
  physics, `SfxId::{LedgeHop,CheckpointChime}`.
- **M3 chunk cache**: `engine::chunks::ChunkCache` (48 LRU, 16×16 tiles),
  `Draw::chunk_bake_*` / `chunk_blit`, 2 bakes/frame amortization + map-load
  prebake, animated-tile overdraw, direct-draw fallback, `World::set_tile` +
  dirty chunks. F1 shows `chunks: ready/budget bakeN`.
- **M4 overworld 240×240**: six region shells (village/grove/camp/ruins/
  cliffs/shrine) + connective river/roads/bridges/meadow; outer 2-tile
  solid border; checkpoints + banners + placeholder spawns.
- **M5 runtime**: `GameMode`/`Transition` fades, `switch_map` +
  `PlayerPersist`, interior stubs (houses/shop/caves), `spawner` distance
  lifecycle, WaveDirector scoped to Arena, F3 map cycle / F4 entry teleport,
  SaveGame v2 (checkpoint-based; v1 JSON → New Game).
- **M6 camera**: soft 24×16 dead-zone, lookahead ease 0.08 toward 16 px,
  follow 0.15 retained; region banners via `WorldEvent::RegionEntered`.

### Frozen 2B seam table (PHASE2A §5)
| # | Seam | Status |
|---|---|---|
| 1 | `MapDef` v2 + painters + `catalog::tile_info` + id ranges | **frozen** |
| 2 | `SpawnKind` / `TriggerKind` / `RegionDef` / `EntryPoint` (2B appends) | **frozen** |
| 3 | `content::maps::build(MapId)` + `MapId`↔u8 | **frozen** |
| 4 | `game::state::switch_map` + `PlayerPersist` + Transition fade | **frozen** |
| 5 | `spawner::SpawnSlot` Dormant/Alive/Dead + respawn rule | **frozen** |
| 6 | `World::set_tile` + dirty-chunk invalidation | **frozen** |
| 7 | `SaveGame` v2 (`gems`/`flags` for 2B writers) | **frozen** |
| 8 | Chunk cache API + animated-tile overdraw list | **frozen** |
| 9 | `WorldEvent::RegionEntered` + `ui::banner` | **frozen** |
| 10 | Debug F3 (map cycle) / F4 (teleport entries) | **frozen** |

### Deviations / tuning
- Camera dead-zone / lookahead at brief defaults (24×16, look 16 @ 0.08) —
  no ±30% needed after smoke.
- Palette: 4 new entries W–Z (dirt light, ash, sand, wood); logged.
- Arena FLOOR/WALL/FOUNTAIN relocated to catalog ids 200–204 (0 = void).
- Overworld boots at entry 0 (village south gate); arena via F3 with overlay.
- Structure shells live in `tiles_forest.rs` with canopy (kept files <600).

### Verification
- `cargo check` + `clippy -D warnings` (wasm32) clean
- `env -u NO_COLOR trunk build --release` → `dist/` ok
- Playwright vs `python3 -m http.server 8090 --directory dist`:
  - Boot overworld; grass/path/trees + bat spawn; HUD hearts
  - F1: `fps~60`, `map OW`, `chunks: 18/48 bake0`
  - Save v2 JSON in `shard_save_v1` (`version:2`, map/entry/checkpoint)
  - F3 → Arena WAVE 1 + dummies/pillars (`map Arena`, chunks live)
  - Processes cleaned: http.server:8090 + headless_shell killed after smoke

### Perf notes / risks
- Mid-laptop smoke ~60 fps with chunk blit path; bake amortized (bake0 after
  warm). iPhone Gate C still unverified on device.
- Overworld shell density is terrain-only — 2B will add props; watch animated
  water count (coarsen if >180 visible).
- Door re-entry guarded by authored entry points + `door_cooldown`; debug
  assert scans door landings at map load.

### Gate readiness for 2B
**YES** — seams frozen above. Residual: human full-map ledge/bridge feel
pass, gamepad still untested, art is readable shell quality not final polish.

## Phase 2B completion — 2026-07-21 (Grok 4.5 High Fast worker)

### What landed
- **M1 Interact**: `content::text` + `flags` registries; `SpawnKind::{Sign,Npc,Chest,Gem}`
  + `EntityKind` twins; `game::interact` + `ui::dialog` (typewriter + TextBlip);
  chests (Loot::Rupees/HeartPiece/Gem) persist via flags; rupee HUD retained.
- **M2 Decoration**: props_village/wild/interior + npcs art; catalog ids 240–259;
  all six regions dressed; houses/shop/caves furnished; `MapId::ShrineLobby`.
- **M3 Encounters**: authored slime/bat/octorok placements; camp guard `group=41`
  + `group_cleared` / `WorldEvent::GroupCleared`; village/CP/door safe (no hostiles
  in village shell).
- **M4 Gems + seal**: Courage pedestal (grove), Power chest (camp, group-gated),
  Wisdom pedestal (ruins); gem hold-up + GemGet + checkpoint; shrine interact
  seal → set_tile open + ShrineLobby stub; `DOOR_SHRINE_OPEN` restore on load.
- **M5 Minimap**: 60×60 fog bitset in SaveGame v2 (`fog: Vec<u32>`, 113 words);
  corner map (M toggle) + pause map (Esc/Enter); POI icons + objective marker.
- **M6 Secrets**: 10 telegraphed secrets/flags (bomb-wall inert stub, cliffs cave
  heart #1, ruins cellar heart #2, river island, shop hedge, pale-tree heart #3,
  camp tower, summit vista, meadow flower ring, shrine braziers); SecretChime;
  4-piece → max-heart helper; banner panel skin.

### Flag registry
`content::flags` (re-exported as `game::save_data::save_flags`): QUEST_STARTED,
CHEST_*, SECRET_*, DOOR_SHRINE_OPEN, HEART_PIECE_1..4, HEART_REWARD_APPLIED,
GEM_*, GROUP_CAMP_GUARD, GRP_* encounter groups.

### Screenshots (Playwright `/tmp/p2b_smoke/`)
- `01_village.png` / `02_f1.png` — overworld props, sign, chest, NPCs, corner
  minimap, `fps~60`, `chunks: 18–36/48 bake0`
- `06_arena.png` — F3 Arena still waves/dummies (combat intact)
- Fog words non-zero in localStorage after walk; version:2 save roundtrip

### Deviations / tuning
- Engine: KeyM → `InputState.minimap_toggle` (brief required M; logged).
- Multi-frame prop/NPC grids authored as horizontal strips (atlas contract).
- `game/src/lib.rs` ~601 lines (at ~600 cap).
- Pause-map Esc/Enter wired in code; automated screenshot evidence weak (prefer
  human glance). Interact range loosened to 28 px for feel.
- Ambient leaf/ember particles not shipped (residual polish).
- NPC art is readable stub quality, not final Minish Cap polish.

### Stubbed for Phase 2C / 3
- Grove chime puzzle (stands + sign present, inert)
- Ruins plate court (detail rubble plates, inert)
- Camp barricade destructibility
- Shop economy UI (shopkeeper “stock arrives soon”)
- Bomb-wall open (hollow interact only; flag reserved)
- Broken-bridge crank / full river path tooling
- Heart piece #4 from shop
- New enemy families (wisp/skeleton/raiders)
- Real dungeon beyond ShrineLobby sign

### Verification
- `cargo check` + `clippy -D warnings` (wasm32) clean
- `env -u NO_COLOR trunk build --release` → dist ok
- Playwright vs `python3 -m http.server 8090 --directory dist`: boot OW, F1
  ~60 fps, fog in save, F3 Arena; **processes cleaned** (http.server +
  headless_shell killed)

### Gate readiness for 2C / Phase 3 brief?
**YES** — soft critical path + content fill seams are in; 2C can swap puzzle
mechanics onto placed props/flags without map surgery. Recommend human play
of elder→3 gems→seal before 2C polish pass.

## Phase 2C planning — 2026-07-21 (Planner, Fable 5)

- Phase 2B accepted at `ea4edd9`. Phase 2C is one brief
  (`WORKER_BRIEF_PHASE2C.md`) with two **sequential** parts: **2C-A**
  (puzzle layer + real gem gates + bombs/B-item + shop economy + bomb wall /
  barricades / bridge crank) then **2C-B** (raider spear/torch + wisp +
  skeleton families, camp 3-wave war-chest battle, encounter re-dress).
  2C-B is blocked on 2C-A's completion entry here; 2C-A's frozen seams are
  listed in its DoD §7.
- Key design decision: **no boomerang until Phase 3** (it's inside the
  shrine), so every 2C puzzle is solvable with the current kit — chimes and
  the crank answer *any* player hit (sword swing, sword beam, reflected
  octorok rock); the finale demands 3 chimes inside a 4 s window (dash
  between them). Phase 3's boomerang retro-enables one-throw solves through
  the same tool-agnostic hit checks, so nothing is special-cased.
- Key reality deltas the brief encodes: there is **no entity-vs-entity
  solidity** (physics is AABB-vs-tile only) → all gates/blocks/plates/
  barricades/cranks are *tiles* mutated via `World::set_tile` (dirty-chunk
  seam), not entities; `active_attacks` clears inside `resolve_hits` →
  `puzzle::update` slots between `integrate_non_player` and
  `combat::resolve_hits`; `MapId::Cave` codec clamps to 2 caves → extended
  for the bomb cave `Cave(2)`; `game/src/lib.rs` sits at the 600-line cap →
  2C-A M1 extracts debug helpers first.
- Controls decision (DECISIONS §5 spirit, logged here): shield stays
  hold-Item (K/pad 2); B-item fires on a ≤8-tick *tap* of Item; cycle is a
  new `BUTTON_CYCLE` (Q / pad LB+RB) — the only engine change 2C may make.
  Touch parity for items is deferred to Phase 4 (touch has joystick +
  attack + dash today).
- Camp battle uses chained spawner groups (41→42→43 via a `locked_groups`
  set + `GroupCleared` unlock), NOT the Arena WaveDirector — the spawner's
  group lifecycle already gives dormant/dead/respawn semantics for free.
- New id allocations: catalog tiles 260–279 (gate, block, plates, barricade,
  crank, lowered bridge), flags 90–98 (+ `CHEST_RUINS_BONUS = 20`), SFX
  appends for chime/plate/gate/block/barricade/crank/bomb/shop/cycle.
- Phase 3 briefs (dungeon/boomerang/boss split) will be written **after 2C
  lands** — they depend on 2C's puzzle hit-check API, `BUTTON_CYCLE`
  seam, and skeleton stagger hook, so writing them now would repeat the
  ARCHITECTURE-vs-reality drift we've avoided by briefing against real code.
  PHASE_PLAN Phase 2/3 sections updated accordingly.

## Phase 2C-A completion — 2026-07-21 (Grok 4.5 High Fast worker)

### Drift vs brief §1
- Brief assumed HEAD `ea4edd9` (phase2b); actual start was `b813946` (phase2c-plan). Code + 2B notes still matched §1 facts; no behavioral drift.
- Added `PUZZLE_RUINS_FAR = 99` (beyond 90–98) so the ruins far-switch gate restores on load; chest loot flag remains `CHEST_RUINS_BONUS = 20`.
- `T_BRIDGE_LOWERED` uses a darker plank sprite (`prop_bridge_lowered`), not a palette-row reuse of `bridge_h`.

### What landed (M1–M5)
- **M1**: `game::debug` extract (`lib.rs` 556); catalog tiles 260–266 + `props_puzzle` art; `content::puzzles`; `game::puzzle` (paint/restore/update after `integrate_non_player`, before `resolve_hits`); flags 90–99; SFX appends.
- **M2**: Grove teaching chime gates + finale seal on Courage gem (`GemData.sealed`); ruins plate court fence + blocks/plates + far-switch crank/chest; Courage sealed interact text.
- **M3**: `BUTTON_CYCLE` (Q / LB+RB); tap-K bombs / hold-K shield; `EntityKind::Bomb` + `AttackKind::Bomb`; bomb wall → `Cave(2)` + 100₹ chest; F1+H debug bomb grant retained.
- **M4**: `game::ui::shop` stock (bombs×5 / bag / heart #4 / locked tunic); persist via flags + `pending_save`.
- **M5**: Broken-bridge crank at (67,91) → lowered planks; camp barricade previews.

### Economy math (WORKER_NOTES)
- Sources ≈ 5+20+25+30+50+50 + **100** (bomb cave) + **50** (ruins far, gated) + kill drops (~17%).
- Sinks: 10×n bomb restocks + 100 bag + 200 heart. A player clearing ~6/10 secrets can afford bag + heart + ~10 restocks without grind. No cache ±30% needed.

### Verification
- `cargo check` + `clippy -D warnings` (wasm32) clean
- `env -u NO_COLOR trunk build --release` ok
- Playwright vs `python3 -m http.server 8090 --directory dist` (`/tmp/p2c_a_smoke/`): boot OW, F1+H grants bombs (`bomb_cap:10`), tap-K places (count 9), save v2 fields present, F3 Arena ok; **http.server + headless_shell killed**
- File caps: lib 556, puzzle/mod 552, interact 427, draw_world 565, state 356

### Frozen seams for 2C-B (do not break)
1. `game::puzzle` API: `PuzzleState::for_map`, `paint_closed`, `restore`, `update`, `try_open_bomb_wall`, `bomb_break_barricades` + `content::puzzles` shapes (`OverworldPuzzles` / def structs / `for_map`)
2. `BUTTON_CYCLE` (=6), `BUTTON_COUNT` (=7) — touch item parity still Phase 4
3. `AttackKind::Bomb` (no style verb)
4. Barricade break path (`puzzle::barricades::damage_barricade` / bomb radius)
5. Flags **90–98** (+ `99` ruins far; `CHEST_RUINS_BONUS=20`; `TUNIC_UNLOCKED=98` reserved)
6. `SaveGame` / `PlayerPersist` fields: `bombs`, `bomb_cap`, `selected_item` (`#[serde(default)]`, version stays 2)
7. `GemData.sealed` (Courage); shop `UiState.shop`; bomb entity kind

### Ready for 2C-B?
**YES** — 2C-A completion entry is this section. Residual risks: human feel on chime window / block push latency / tap-vs-shield; plate court push lanes vs column clutter; gamepad LB/RB cycle untested on hardware; camp barricades are cover previews only (wave chain is 2C-B).
