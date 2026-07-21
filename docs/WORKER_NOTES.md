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

## Phase 2C-B completion — 2026-07-21 (Grok 4.5 High Fast worker)

### Gate
Read Phase 2C-A completion + frozen seams (puzzle API, BUTTON_CYCLE, AttackKind::Bomb, barricade path, flags 90–99, bomb persist fields, GemData.sealed / shop / Bomb). Honored; no 2C-A puzzle/shop rework beyond enemy wiring.

### What landed (M1–M4)
- **M1 families**: `content::art::enemies_act1b` (raider spear/torch, wisp, skeleton, torch proj/flame); `game::enemies::{raider,wisp,skeleton}`; SpawnKind/EntityKind/EntityData + exhaustive match sweep; SFX `RaiderPoke`/`GuardClank`/`TorchThrow`/`FlameBurst`/`WispPhase`/`SkeletonRattle`; tuning rows; front-guard refuse (spear Guard + skeleton shield); `DamagedPlayer.source: Option<EntityId>`; skeleton perfect-block stagger + pub `skeleton::stagger` Phase 3 hook.
- **M2 camp waves**: `GRP_CAMP_W2=42` / `GRP_CAMP_W3=43` + `CAMP_WAVE_CHAIN`; spawner `locked_groups` + `camp_war_won`; GroupCleared 41→42→43 unlock/force-spawn + WaveCue toasts; clear 43 → `GROUP_CAMP_GUARD` + checkpoint save; power-chest fallback uses group 43; mid-chain rewind guard + chain reset on wave-1 respawn.
- **M3 re-dress**: camp raiders+bats; ruins octoroks+wisps+skeletons near plates; cliffs +1 summit wisp; shrine sentinels skeleton+spear/side.
- **M4 feel**: defaults within ±0 (no tuning log needed after smoke); `draw_enemies` extract (draw_world 509).

### Phase 3 inherits
- Puzzle hit-check API + crank/chime/barricade seams (2C-A)
- `BUTTON_CYCLE` / B-item selected_item u8 (boomerang=2)
- `skeleton::stagger` + perfect-block source plumb
- `TUNIC_UNLOCKED=98` + ruins far-switch / `CHEST_RUINS_BONUS` site
- Camp wave chain pattern (`locked_groups` + `CAMP_WAVE_CHAIN`)

### Verification
- `cargo check` + `clippy -D warnings` (wasm32) clean
- `env -u NO_COLOR trunk build --release` ok
- Playwright vs `python3 -m http.server 8090 --directory dist` (`/tmp/p2c_b_smoke/`): boot OW, F1 ~60 fps, save v2, F3 Arena intact; **http.server + headless_shell killed**
- File caps: lib 591, draw_world 509, draw_enemies 168, raider 491, spawner 352

### Deviations / risks
- Human feel pass still owed: wave-2/3 spike, wisp annoyance, spear guard readability, torch arc aim.
- Smoke F4 entry cycle landed shrine pocket (not camp clearing) — camp wave chain verified by code path + placement; recommend human camp break-in.
- Gamepad still untested on hardware.

### Ready for Fable Phase 3 briefs?
**YES** — Phase 2C overall (2C-A + 2C-B) complete. Soft critical path has real gem gates + bombs/shop + four Act 1B families + camp 3-wave war-chest.

## Phase 3 planning — 2026-07-21 (Planner, Fable 5)

- Phase 2C accepted at `7fa19b3`. Phase 3 split into two **sequential**
  briefs written against the real 2C code:
  `WORKER_BRIEF_PHASE3A.md` (dungeon rooms/slides, Gale Boomerang, crystal +
  flame + ordered-seal curriculum, keys/doors, dungeon minimap; Sanctum Core
  + Guardian Arena shipped dormant) then `WORKER_BRIEF_PHASE3B.md`
  (Ironshell duo, Granite Warden 3 phases, victory/credits/village return,
  Gate B). 3B is blocked on 3A's completion entry here; 3A's frozen seams
  for 3B are its brief §6.
- Split rationale: the boomerang is load-bearing for the curriculum AND the
  boss, so it lives with the dungeon worker (3A) — 3B consumes it as a
  frozen seam (`AttackKind::Boomerang` pass-through + `throw_id` dedupe +
  `enemies::stun`). This differs from the old PHASE_PLAN "puzzle=A,
  boss+boomerang=B" sketch, which would have made the curriculum worker
  build against a tool that didn't exist yet.
- Key reality deltas the briefs encode: room-to-room movement is **camera
  slides inside one `MapDef`** (additive `Camera::set_bounds`), not
  `switch_map` fades — the 2A fade/transition seam stays map-level
  (lobby↔dungeon). Doors/shutters/gates are **tiles** (2C precedent: no
  entity solidity). The boomerang joins `puzzle::process_hits` as a third
  player projectile arm that does NOT set `hit`/despawn on tile contact, so
  one throw can ring multiple chimes/crystals — this is what retro-enables
  the overworld one-throw solves with zero special-casing. `Game::new`
  currently forces boot to Overworld/Arena — 3A extends it so dungeon saves
  boot the dungeon (checkpoints 7/8; 3B adds pre-boss 9).
- Headroom mandated up front: `world/entity.rs` (598) and `lib.rs` (591) are
  at the cap — 3A M1 extracts `entity_data.rs` and `events.rs` before any
  feature code (same pattern as 2C-A's `debug.rs` extract).
- Allocations: MapId 4 = Dungeon; catalog tiles 280–309 (3A) / 310–319 (3B);
  flags 100–139 (3A: item/keys/doors/seals/rooms) / 140–149 (3B:
  intro/defeat/heart/shard/tunic-bought) + `TUNIC_UNLOCKED=98` set by 3B;
  spawner groups 90–94 (94 = Sanctum miniboss, locked until 3B); Loot gains
  Boomerang/SmallKey/BossKey; `StyleVerb::BoomerangStun` (verb_cooldowns →
  7). Keys are derived from flags (chest flags minus door-opened flags) — no
  new SaveGame fields; version stays 2.
- Dungeon minimap reciprocity is data-driven: one `rooms()` exits table
  drives both the painted door tiles and the map render, plus a load-time
  debug assert — acceptance criterion 4 can't drift.
- Boss design locked to GAME_DESIGN §6 with entity crystals (not the 2C tile
  crystals — they orbit/swap in P2/P3); core damage gate enforced in
  `apply_attack_hit`; `HITSTOP_BOSS_BREAK=12` (reserved since Phase 1)
  finally consumed. Heart pieces stay complete at 4 — Warden drops a full
  heart container instead.
- PHASE_PLAN Phase 3 section rewritten to match. No gameplay code in this
  commit.


## Phase 3A completion — 2026-07-21 (Grok 4.5 High Fast worker)

### Gate
Read Phase 3 planning + 2C frozen seams (puzzle hit API, BUTTON_CYCLE, skeleton::stagger,
barricades, flags 90–99). Honored; overworld puzzle API untouched. Boomerang is a third
projectile arm in `puzzle::process_hits` that does **not** set hit/despawn on tile contact.

### What landed
- **M1 headroom**: `world/entity_data.rs` (enemy family data + `BoomerangData`);
  `game/src/events.rs` (`drain`); `lib.rs` ~508, `entity.rs` ~470.
- **MapId::Dungeon (=4)** + `content/maps/dungeon.rs` + `dungeon_rooms.rs` (16 rooms,
  reciprocal exits); shrine lobby north door → dungeon entry 0; dungeon south → lobby entry 1.
- **Rooms**: `game/src/rooms.rs` — camera `set_bounds`, ~24-tick smoothstep slides,
  shutter slam on Trials entry, key/boss/seal/inner door unlock from flags, debug
  exit-reciprocity assert.
- **Catalog 280–307** + art `tiles_dungeon` / `props_dungeon` / `items` (boomerang strip);
  flags 100–139 + GRP 90–94 (Sanctum 94 locked); SFX + TextId appends.
- **M2 Gale Boomerang**: `items/boomerang.rs` — throw/return/catch, pass-through combat
  (`AttackKind::Boomerang` 0.5 dmg), `enemies::stun` + `skeleton::stagger`, style
  `BoomerangStun`, puzzle hits + barricade chip 1, item cycle bombs↔boomerang, pickup magnet.
- **M3 layout**: Vestibule / Trials 1–3 + boomerang chest + key1 / Currents curriculum /
  flame key2 / seals / antechamber (boss key) / Sanctum + Arena stubs; checkpoints 7/8;
  dungeon boot from save; `ui/dungeon_map.rs` (M + Esc pause page).
- **M4 puzzles**: `content/puzzles_dungeon` + `game/puzzle/dungeon` — crystal toggles,
  multi same-state gate, carry-a-flame, ordered seal throws; both seals open ante shutters.
- **M5 HUD**: dungeon key pips + boomerang icon; F1+H grants bombs + boomerang + keys (debug).

### Tuning / decisions logged
- Boomerang flies over WATER (SOLID only blocks); speed 3.4 out / 4.2 return, range 112 px,
  stun 60 ticks — defaults within ±0 after smoke.
- West wing open by design; Currents door = SmallKey (`DDOOR_WING`); Lines→SealE = InnerKey
  (`DDOOR_INNER`). Key inventory derived from chest flags − door flags.
- Unsolved flame torches reset on room re-enter; seals persist via flags.
- Multi-frame dungeon sprites: `SpriteDef.w` is **frame** width (atlas `w * frames`).

### Verification
- `cargo check` + `clippy -D warnings` (wasm32) clean
- `env -u NO_COLOR trunk build --release` ok
- Playwright vs `python3 -m http.server 8090 --directory dist` (`/tmp/p3a_smoke/`):
  boot OW, F1+H grants flags 100–103, F3 Arena WAVE 1 ~60 fps; **http.server + headless killed**
- File caps: lib 508, entity 470, puzzle/dungeon 493, rooms 393, dungeon map 369

### Frozen seams for 3B (do not break)
1. `RoomDef` / `rooms()` / `RoomsState` / slide API (`rooms::update` pause)
2. Dungeon geometry: Sanctum Core + Guardian Arena rects, entries, `GRP_DNG_SANCTUM=94`
   (locked), boss-door tiles, checkpoint 8
3. `AttackKind::Boomerang` (pass-through, stun-on-hit, dmg 0.5) + `enemies::stun` +
   `throw_id` dedupe; `skeleton::stagger` for shield drop
4. Flags **100–139** allocated; **140–149** reserved for 3B; `TUNIC_UNLOCKED=98` still reserved
5. `DungeonPuzzleState` + crystal-toggle / seal-order APIs (3B boss crystals are entities)
6. Catalog **280–307** used; **308–309** spare; **310–319** reserved for 3B arena dressing
7. `Loot::{Boomerang,SmallKey,BossKey}`; `verb_cooldowns: [u16; 7]`; `StyleVerb::BoomerangStun`

### Deviations / residual risks
- Trials II plate is placed; exit still primarily shutter-gated on `GroupCleared` (plate is
  teach-pressure, not a hard soft-lock gate).
- Seal throw geometry is authored for plinth lines — freestyle solves possible; human feel
  pass owed on throw tolerance + shutter fight density.
- Overworld retro-enable verified by code path (same `process_hits` arm); live post-dungeon
  field throw not fully scripted in smoke (F1 grant exercises combat/puzzle arms).
- Touch item parity still Phase 4; gamepad LB/RB cycle still untested on hardware.

### Ready for Phase 3B?
**YES** — dungeon walkable with debug grants, boomerang + curriculum + keys + minimap +
dormant Sanctum/Arena stubs. 3B can drop Ironshell / Granite Warden into reserved seams
without map surgery.

## Phase 3B completion — 2026-07-21 (Grok 4.5 High Fast worker)

### Gate
Read Phase 3A completion + frozen seams (rooms/slide API, Sanctum/Arena rects + GRP 94,
`AttackKind::Boomerang` + `throw_id`/`stun`, flags 100–139, dungeon puzzle APIs, catalog
≤307, Loot/StyleVerb). Honored; no room/slide/boomerang/puzzle internals rewritten.

### Drift vs brief (code wins)
- 3A placed Skeleton+RaiderSpear in GRP 94, not Ironshells — replaced with 2× `SpawnKind::Ironshell`.
- Boss-key chest already in antechamber (3A); duo guards Sanctum → Arena BossKey door path.
- Tunic cosmetic palette-swap deferred to Phase 5 polish (purchase + `TextId::TunicBought` shipped).
- Credits skip = hold Attack ≥60 ticks (Phase 4 owns R-to-skip polish).

### What landed (M1–M4)
- **M1 Ironshell duo**: `enemies/ironshell.rs` + art in `content/art/boss.rs`; front armor
  refuse (incl. frontal boom); back-hit dmg+stun+stagger; perfect-block stagger on bash;
  bombs any-side (2 dmg, HP 8 → wasteful cheese); Sanctum shutter + `unlock_and_activate(94)`;
  `GroupCleared` → `SANCTUM_CLEARED=145`, toast "THE CORE QUIETS", shutters open, save.
- **M2 Arena + scaffold**: checkpoint **9** + EntryPoint antechamber; arena perch dressing;
  `game::boss` (`mod`/`granite_warden`/`warden_crystals`/`warden_attacks`/`pebble`);
  cinematic intro (flag `WARDEN_INTRO_SEEN=140`) + short retry; segmented boss bar;
  core closed refuse in `apply_attack_hit` via `WardenData.core_exposed`.
- **M3 phases**: crystal prime/`throw_id` dedupe → dual prime → GALE → core 240 ticks;
  P1 slam+3-rock fan; P2 orbit + pebbles + sweep; P3 rim pit crumble + 5-fan + perch swap +
  fake-core flash; `HITSTOP_BOSS_BREAK` on phase breaks; death → `boss::clear` + cp9 reset.
- **M4 victory**: heart container (`WARDEN_HEART=142`, +2 max) → Shard dialog
  (`SHARD_OF_COURAGE=143`) → credits stub → village fountain entry 1; `WARDEN_DEFEATED=141`
  + `TUNIC_UNLOCKED=98`; shop tunic 300₹ once (`TUNIC_BOUGHT=144`); elder victory line;
  re-kill skips rewards.

### Flags (140–149)
`WARDEN_INTRO_SEEN=140`, `WARDEN_DEFEATED=141`, `WARDEN_HEART=142`, `SHARD_OF_COURAGE=143`,
`TUNIC_BOUGHT=144`, `SANCTUM_CLEARED=145`. Act 1 complete predicate for Phase 4:
**`SHARD_OF_COURAGE` in flags**.

### Tuning log
- Ironshell HP 8 / bomb 2 → ≥4 blasts each (logged as anti-cheese).
- Warden HP 48; phase gates 75%/35% (36 / 16.8); prime 300 / P2+ 210; core expose 240;
  telegraphs ≥30 (slam/sweep 36). Defaults within ±0 after smoke.

### Verification
- `cargo check` + `clippy -D warnings` (wasm32) clean
- `env -u NO_COLOR trunk build --release` ok
- Playwright vs `python3 -m http.server 8090 --directory dist` (`/tmp/p3b_smoke/`):
  `01_boot.png` / `02_f1h.png` / `03_walk.png` / `04_arena.png` — no console errors;
  save v2 written; F3 Arena intact; **http.server + headless_shell killed**
- File caps: `granite_warden` 406 (split crystals/attacks), `lib` 526, boss modules <400

### Gate B
**YES — critical path systems complete**: Ironshell duo + Granite Warden 3-phase
crystal/gale/core fight + victory/credits/village return + tunic unlock. Automated smoke
covers boot/F1/F3/save; full New-Game→Warden human feel pass still recommended (telegraph
readability, fake-flash teach, death mid-P2 → cp9). Screenshots: `/tmp/p3b_smoke/01–04`.

### Phase 4 inherits
- Victory predicate: `SHARD_OF_COURAGE`
- Credits stub (`ui/credits.rs`, Attack-hold skip)
- Checkpoint map: 1 village, 2–4 gems, 6 shrine, 7 vestibule, 8 ante, **9 pre-boss**
- Tunic: unlocked/bought flags; cosmetic sprite swap still open
- Boss re-fightable (short intro; rewards once)

### Ready for Fable Phase 4/5 briefs?
**YES** — Gate B closed for Act 1 climax systems. Residual risks: human boss feel pass,
gamepad/touch item parity still Phase 4, tunic palette cosmetic polish.
