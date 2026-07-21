# WORKER_BRIEF_PHASE2A.md — Act 1 Overworld Foundation (map format, chunks, camera, transitions)

You are a Grok worker. This brief is self-contained; follow it exactly. Conflict
order: `DECISIONS.md` → `ARCHITECTURE.md` → this brief → `GAME_DESIGN.md`.
You run autonomously — do not ask the human anything except for credentials.

## 0. Mission

Phase 1 shipped a 60×34 gray-box arena that plays and looks like a real game.
You build the **foundation of the ambitious Act 1 overworld**: a single
contiguous **240×240-tile** scrolling map (GAME_DESIGN §4) with layered tiles
(ground / detail / overhang), a **chunk-cached renderer** that holds 60 fps,
Minish-Cap-style camera follow, extended collision (solid / water / one-way
ledge hops), a **trigger + spawn-point system**, **map switching with fade
transitions** (overworld ⇄ interior room stubs), all six **named region
shells** with banners, and a versioned save that knows map + checkpoint.

You do **terrain, systems, and seams** — not content density. Phase 2B (a
separate worker, after you) fills the world: props, interiors, NPCs/signs,
encounters, gems, minimap, secrets. Every seam you freeze here is 2B's
contract, so precision beats cleverness.

## 1. Read first

1. `docs/DECISIONS.md` — locked stack; crates allowed; conflict order.
2. `docs/ARCHITECTURE.md` §3 (chunked rendering), §6 (map format), §11 (ownership).
3. `docs/GAME_DESIGN.md` §4 — the overworld you are building the skeleton for.
4. `docs/WORKER_NOTES.md` — Phase 1A/1B completion entries (frozen seams).
5. The actual code listed in §2. **Where this brief differs from code, code +
   WORKER_NOTES win; log drift in WORKER_NOTES and adapt.**

## 2. Current code reality (verified 2026-07-21, commit `6bf033e`)

Build on these as they exist — do not re-architect:

1. **`content::maps` (v1)** — `MapDef { width, height, ground: Vec<u16>,
   collision: Vec<bool> }` with `new / idx / in_bounds / solid_at / fill /
   rect_border / fill_rect / set`; consts `TILE_PX = 16.0`, `FLOOR = 0`,
   `WALL = 1`, `FOUNTAIN = 2`; single builder `arena()` (60×34). **You replace
   this v1 format** (see M1) — it is the one seam you may break, because you
   own every consumer and the compiler will walk you to each.
2. **`game::Game::new`** (`crates/game/src/lib.rs`, 409 lines — near the
   600-line cap, you must split, see M5) hardcodes `content::maps::arena()`,
   spawns 3 dummies + a fountain at fixed pixels, holds `WaveDirector`.
3. **`game::world::World`** — `{ arena, free, map: MapDef, camera, hitstop,
   events, tick, rng, player_id, active_attacks, hit_pairs }`; generational
   spawn/despawn/get/get_mut/iter_alive/alive_ids; `WorldEvent` enum
   (AttackHit / DamagedPlayer / Killed / FxRequest / Sfx / StyleAction /
   EnergyDenied). Frozen for you except **additive** WorldEvent variants.
4. **`game::world::physics`** — axis-separated AABB sweep vs `map.solid_at`;
   `query_aabb / aabb_overlap / circle_hits_entity`; knockback decay. You
   extend `move_entity` for water/ledges (M2) — keep signatures.
5. **`game::world::camera::Camera`** — lerp 0.15 toward target + 12 px facing
   lookahead, clamps to `(map_w_px, map_h_px)` with half-view 240×135,
   screenshake ≤3 px, `offset()` returns view top-left. You tune (M6), keep API.
6. **`game::draw_world::render_map`** — per-frame visible-tile loop, hardcoded
   `match` on WALL/FOUNTAIN/floor-checker + `is_interior_pillar` hack. **You
   rewrite this data-driven + chunked** (M2–M3).
7. **Atlas** — `game::assets::bake()` decodes `content::art::all_bakes()` grids
   into a 1024×1024 `engine::atlas::AtlasBuilder` (shelf pack, 1 px gutter,
   canvas must stay DOM-attached `display:none`); `SpriteMap.get(&'static str)
   -> Option<SpriteHandle>`; `Draw::sprite(h, frame, x, y, flip_x)` +
   `sprite_scaled`. Plenty of atlas room (<25% used).
8. **`engine::render::Draw`** — `set_atlas / set_offset / clear / rect / circle
   / line / text / sprite / sprite_scaled`; internal canvas 480×270. `engine`
   is the only crate touching `web-sys`; `game` must stay web-sys-free.
9. **Input** — `BUTTON_INTERACT = 3` (E / pad Y/Triangle) exists and is unused
   by gameplay. Debug indices: `DEBUG_OVERLAY` (F1), `DEBUG_VIEWER` (F2),
   `DEBUG_ACTION` (H, only when overlay on).
10. **Save** — `game::save_data::SaveGame { x, y }`, key `shard_save_v1`,
    parse-failure falls back to `default_spawn()`. App autosaves every 60 ticks.
11. **Enemies** — `slime::spawn(world, pos)`, `bat::spawn`, `octorok::spawn`;
    `WaveDirector` is arena-only scaffolding (keeps running unconditionally
    today — you must scope it to the arena map, M5).
12. **FX/UI** — `FxKind::Toast { text: &'static str }` (max 2 stacked),
    `ui::hud` layout consts, F2 sprite viewer pauses the world.
13. Environment: rustup toolchain ahead of Homebrew rustc; `env -u NO_COLOR
    trunk build --release`; never ship `trunk serve` output.

## 3. Locked constraints

- Stack per DECISIONS.md. **No new crates.** `content` depends on nothing;
  `engine` = only web-sys; dependency direction never reversed.
- Overworld is **one contiguous 240×240 `MapDef`** (3840×3840 px ≈ 8×14
  GBA-ish screens). You may extend to at most 256×240 if the layout in §M4
  needs breathing room — log it. Do NOT shrink below 240×240; the human wants
  an ambitious map, and Phase 2B fills it.
- Interiors are **small separate `MapDef`s** with door links (ARCHITECTURE §6).
- No file >~600 lines. Pre-planned splits: `content/src/maps/overworld/`
  (one module per region), `game::state.rs`, `game::world/spawner.rs`,
  `engine::render` stays <600 by putting chunks in `engine/src/chunks.rs`.
- Art remains handcrafted indexed-color grids (DECISIONS §3), 16×16 tiles.
- Commits on `main`, prefixed `phase2a:`, small and frequent. Do not push, do
  not deploy. Every commit: `cargo check --workspace --target
  wasm32-unknown-unknown` and `cargo clippy --workspace --target
  wasm32-unknown-unknown -- -D warnings` clean.
- Tuning ±30% allowed with a WORKER_NOTES log line. Blocked → WORKER_NOTES +
  nearest non-blocking interpretation.

## 4. Milestones (commit at least once each)

### M1 — MapDef v2 + tile catalog (content crate)

Replace map format v1 in `crates/content/src/maps/`:

```rust
// maps/mod.rs
pub const TILE_PX: f32 = 16.0;

pub mod flags {
    pub const SOLID: u8   = 1 << 0; // blocks all solid bodies
    pub const WATER: u8   = 1 << 1; // blocks walkers; flyers/projectiles pass (also SOLID for now — no swimming in Act 1)
    pub const LEDGE_N: u8 = 1 << 2; // one-way hop when moving north
    pub const LEDGE_S: u8 = 1 << 3;
    pub const LEDGE_E: u8 = 1 << 4;
    pub const LEDGE_W: u8 = 1 << 5;
}

pub struct MapDef {
    pub width: u32,
    pub height: u32,
    pub ground: Vec<u16>,     // always drawn
    pub detail: Vec<u16>,     // 0 = empty; drawn over ground, below entities
    pub overhang: Vec<u16>,   // 0 = empty; drawn ABOVE entities (tree canopy, arch tops)
    pub collision: Vec<u8>,   // flags:: bits per tile
    pub spawns: Vec<SpawnDef>,
    pub triggers: Vec<TriggerDef>,
    pub regions: Vec<RegionDef>,
    pub entries: Vec<EntryPoint>, // named spawn points (door landings, checkpoints, new-game)
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MapId { Overworld, Arena, House(u8), Shop, Cave(u8) } // 2B adds more variants

pub fn build(id: MapId) -> MapDef;   // single loader entry point

pub struct EntryPoint { pub id: u8, pub tx: u32, pub ty: u32 } // id 0 = default/new-game

pub struct SpawnDef { pub tx: u32, pub ty: u32, pub kind: SpawnKind, pub group: u16 }
pub enum SpawnKind { Slime, Bat, Octorok, FairyFountain, Dummy }
// 2B appends Npc(..), Sign(..), Gem(..), Chest(..) etc. Keep the enum non-exhaustive in spirit:
// every `match` in game must have explicit arms (compiler walks 2B to each site).

pub struct TriggerDef { pub tx: u32, pub ty: u32, pub w: u32, pub h: u32, pub kind: TriggerKind }
pub enum TriggerKind {
    Door { target: MapId, entry: u8 },
    Banner { region: u8 },        // index into MapDef.regions
    Checkpoint { id: u8 },        // save-point id; also an EntryPoint with the same id
}
// 2B appends Secret{..}, Dialogue{..}. Same additive rule.

pub struct RegionDef { pub name: &'static str, pub rect: (u32, u32, u32, u32) }
```

Keep/extend painter helpers on `MapDef` (they are your authoring API and 2B's):
`fill`, `set(x, y, layer, tile)` (layer enum Ground/Detail/Overhang),
`set_flags(x, y, u8)`, `fill_rect`, `rect_border`, plus new ones in
`maps/paint.rs`: `path(points, width, tile)` (walkable road), `blob(cx, cy, r,
tile, seed)` (organic patches, deterministic via a tiny LCG — content has no
fastrand), `scatter(rect, tile, density, seed)`, `hline/vline`,
`stamp(prefab)` where a prefab is a small `&[&str]` char-grid mapped through a
legend `&[(char, TileSet)]` — used for bridges, house footprints, cliff
switchbacks. Deterministic: same seed → same map every load.

**Tile catalog** in `maps/catalog.rs` — the data-driven contract that kills
`draw_world`'s hardcoded match:

```rust
pub struct TileInfo {
    pub sprite: &'static str, // key into game::assets::SpriteMap
    pub frames: u16,          // >1 = auto-animated
    pub anim_rate: u16,       // ticks per frame
    pub flags: u8,            // default collision flags painted with this tile
}
pub fn tile_info(id: u16) -> &'static TileInfo;
pub const T_VOID: u16 = 0; // never drawn (detail/overhang empty)
```

Assign stable u16 ids grouped by family with gaps for 2B (document ranges in
catalog.rs): 1–39 terrain (grass A/B, path, dirt, sand), 40–79 water (deep,
shallow, shore edges N/S/E/W + corners, animated 2-frame shimmer), 80–119
cliffs (face, top, edge set, stairs), 120–159 forest (tree trunk, canopy
pieces), 160–199 structures (bridge H/V, broken bridge, fence, shrine stone),
200–239 village/interior shells, 240+ reserved 2B. Painters normally set
`collision` from `tile_info(id).flags` automatically (with an override arg for
special cases).

Arena: port `arena()` onto v2 (ground-only + flags) under `MapId::Arena` so
Phase 1 combat testing stays alive. Keep FLOOR/WALL/FOUNTAIN ids stable within
the new catalog (0 is reserved → move them; update `draw_world` consumers).

DoD for M1: workspace compiles with v2 everywhere (`solid_at` reads SOLID bit;
`flags_at(tx, ty) -> u8` added), arena unchanged in play.

### M2 — Terrain tile art + data-driven map rendering + collision v2

**Art** (`content/src/art/`): author the core terrain set as indexed grids —
grass A/B + flower/pebble details, dirt path (+edges), water deep/shallow +
8-piece shore + 2-frame shimmer, cliff top/face/edge set + stairs, tree trunk
(16×16) + 2×2-tile canopy (4 grids), wooden bridge H/V + broken end, fence,
shrine-stone. Reuse the existing palette ramps (grass 4–7, stone g–k, water
u–z indices); add at most ~6 new palette entries (there is charset headroom;
log additions). New files `tiles_terrain.rs`, `tiles_water.rs`,
`tiles_forest.rs` (keep each <600 lines); register in `art::all_bakes()`.
Every tile sprite name matches `catalog.rs` exactly — add a debug assertion in
`game::assets::bake` that every `tile_info().sprite` resolves (fail loud at
startup, listing missing keys).

**Rendering** (`game::draw_world`): rewrite `render_map` to walk the three
layers through `tile_info` (sprite key → SpriteMap lookup cached in a
`Vec<Option<SpriteHandle>>` indexed by tile id, built once at Game::new — no
per-frame HashMap hits). Draw order per frame: ground+detail (chunked, M3) →
y-sorted entities → overhang (chunked) → fx → HUD. Animated tiles (water,
fountain): see M3 for how they interact with chunks.

**Collision** (`game::world::physics`): `move_entity` consults `flags_at`:
- SOLID: as today.
- WATER: solid for solid bodies (no swimming in Act 1). Bats/projectiles
  already bypass tile collision (non-solid bodies / fly paths) — verify, don't
  break.
- LEDGE_*: player-only one-way hop. When the player's sweep is blocked by a
  ledge tile whose direction matches the movement (e.g. moving south into
  LEDGE_S), start a short scripted hop: lock input, animate the player in an
  arc to the first non-solid tile beyond (usually 2 tiles = 32 px), 18 ticks,
  dust poof + a new `SfxId::LedgeHop`. Implement as a new
  `PlayerState::LedgeHop { tick, from, to }` variant (additive — compiler
  walks you to every match). Enemies treat ledges as SOLID.

DoD for M2: a small test strip of every terrain family visible in the arena or
a temp corner of the overworld; walk on path, blocked by water/cliff, hop a
ledge southward but not back.

### M3 — Chunk cache renderer (engine) — the 60 fps backbone

240×240 × 3 layers is ~1,500 visible drawImage calls/frame unchunked — risky
on iPhone (Gate C target). Implement ARCHITECTURE §3 chunk caching:

New file `crates/engine/src/chunks.rs` (wire into `engine::lib`):

```rust
pub const CHUNK_TILES: u32 = 16;           // 16×16 tiles = 256×256 px per chunk
pub struct ChunkKey { pub layer: u8, pub cx: u32, pub cy: u32 } // layer 0 = ground+detail composited, 1 = overhang

pub struct ChunkCache { /* offscreen canvases + LRU bookkeeping, all web-sys here */ }
impl ChunkCache {
    pub fn new(max_chunks: usize) -> Result<Self, String>; // budget 48 chunks ≈ 12 MB
    pub fn ready(&self, key: ChunkKey) -> bool;
    pub fn invalidate(&mut self, key: ChunkKey);
    pub fn begin_frame(&mut self, frame: u64);              // LRU stamp
}
```

Integration through `Draw` (game never sees web-sys):
- `Draw::chunk_bake_begin(&mut self, cache: &mut ChunkCache, key) -> bool` —
  targets that chunk's canvas; while active, `Draw::sprite/rect` draw in
  chunk-local coordinates (game passes `world_px - chunk_origin_px`).
  Returns false (no-op) if the cache is full and nothing evictable this frame.
- `Draw::chunk_bake_end(&mut self, cache)` — restore main canvas target.
- `Draw::chunk_blit(&self, cache: &ChunkCache, key, dst_x, dst_y)` — one
  drawImage of the 256×256 chunk.

Game side (`draw_world`): each frame compute visible chunk range (≤ 3×3 per
layer), `begin_frame`, bake any non-ready visible chunk (cap **2 bakes per
frame** after the initial load burst; on map load, prebake the 3×3 around the
player synchronously), then blit. Ground+detail composite into layer 0 (detail
drawn over ground inside the same bake); overhang is layer 1 and blits after
entities.

**Animated tiles** (water shimmer, fountain): do NOT rebake chunks per frame.
Bake the static frame-0 into the chunk; keep a per-map list of animated tile
positions (built at load from the catalog) and overdraw only those tiles
(sprite frame = `(world.tick / anim_rate) % frames`) after the layer-0 blit.
Overworld water is scattered but bounded — if visible animated tiles exceed
~180, coarsen the shimmer (every other tile animates; log it).

**Fallback**: if `ChunkCache::new` fails (canvas creation), keep the direct
per-tile path (M2 version) behind a flag. Never crash into a black screen.

Dirty support: `World` gets `pub fn set_tile(&mut self, layer, tx, ty, id)`
which updates MapDef + collision flags and pushes the affected `ChunkKey` into
a `dirty_chunks: Vec<ChunkKey>` that `draw_world` drains to
`cache.invalidate`. (Bomb walls in Phase 3 and 2B secrets depend on this seam.)

DoD for M3: F1 overlay shows `chunks: <ready>/<budget>` and bake count/frame;
full-speed run across the map shows no hitches >1 frame and steady ~60 fps.

### M4 — The 240×240 Act 1 overworld terrain (all six region shells)

`content/src/maps/overworld/mod.rs` builds the single MapDef and calls one
module per region — each paints its own tile rect and registers its regions /
entries / triggers / spawns. **Files: `village.rs`, `grove.rs`, `camp.rs`,
`ruins.rs`, `cliffs.rs`, `shrine.rs`, `connective.rs`** (river, roads,
bridges, south meadow). Keep each <600 lines by leaning on paint.rs helpers
and prefab stamps.

Normative layout (tile coords; refine shapes freely, keep compass relations,
sizes, and connection points — 2B builds on these):

```
        (0,0) ────────────────────────────────► x=240
        │  Razor Cliffs        Triforce Shrine (96..144, 4..32, gated door ~ (120,30))
        │  (40..160, 16..92)   ▲ approach only via cliffs switchbacks
        │       │ river source (~96,28)
        │       ▼                Ashen Raider Camp (164..232, 36..104)
        │  Whispering Grove      │
        │  (8..84, 96..200)      ▼
        │       │            Echoing Ruins (156..232, 110..184)
        │       ▼                │
        │   Mosslight Village (92..152, 148..212)
        │       │
        │   South Meadow / new-game fields (84..232, 196..236)
        ▼ y=240
```

Requirements per region shell (terrain + collision + entries only; 2B decorates):

- **Connective**: river from (≈96,28) flowing south then bending east to exit
  the map edge near (238,~150); 4–6 tiles wide; **2 intact bridges** (village
  north ≈(120,142), ruins approach ≈(178,148)) + **1 broken bridge** with gap
  (grove→cliffs shortcut ≈(66,94)) — broken side solid, crank hook is 2B/3.
  Dirt roads: village↔each region, winding not straight. South meadow: open
  grass with tree clusters, the New Game entry (`entry 0` ≈ (118,206) village
  south gate) and gentle geometry.
- **Mosslight Village**: cleared plaza, path grid, 6 house footprints + shop
  footprint + fountain pocket as **stamped solid shells with Door triggers**
  (targets `House(0..=5)`, `Shop`; interiors stubbed in M5). Village fence /
  hedge border with 3 gates (south, west→grove, north→bridge).
- **Whispering Grove**: dense tree-wall maze (trunk tiles solid on ground,
  canopy on overhang so the player walks under fringes); at least 3 distinct
  loops/dead-ends and a NE clearing reserved for the Courage Gem (2B); one
  cave-mouth Door (target `Cave(0)` = fairy grotto stub) near (76,100).
- **Ashen Raider Camp**: palisade ring with 2 openings, bonfire clearings
  (dirt), watchtower footprint (solid 2×2); inner war-chest clearing reserved
  (2B). Terrain reads "scorched": dirt/ash ground variant.
- **Echoing Ruins**: broken column grid (1×1 solid stubs), collapsed arch
  prefabs (overhang tops), sand/cracked-stone ground, plaza reserved for the
  plate court (2B).
- **Razor Cliffs**: 2–3 elevation bands built from cliff-face/edge tiles;
  switchback paths with stairs tiles; ≥3 LEDGE_S hop points shortcutting the
  way down; summit pocket + cave-mouth Door (target `Cave(1)` stub) for the
  heart-piece cave (2B); river source pool.
- **Triforce Shrine**: stone approach, vista terrace, shrine facade with a
  **sealed door tile** (solid; 2B/3 wires the 3-gem gate) + `Checkpoint` and
  `Banner` triggers.

Every region: one `RegionDef` (name per GAME_DESIGN §4), one `Banner` trigger
covering its entrances, ≥1 `Checkpoint` (village fountain, grove clearing,
camp gate, ruins plaza, cliffs base, shrine terrace), and 6–12 placeholder
`SpawnDef`s (slime/bat/octorok mix appropriate to the region) so M5's spawner
is testable — 2B rebalances all encounters.

Also in M4: **map-edge behavior** — the outer 2 tiles of the whole overworld
are solid (cliff/tree/water border), no walking off the world.

DoD for M4: you can walk from the south meadow to all six regions and back
with no loading, tile seams look intentional (shores have edges, cliffs have
faces, paths have edges), and collision never traps the player (sweep the
whole map with the debug teleport from M5).

### M5 — Map runtime: switching, transitions, spawner, save v2, arena scoping

Split `game::lib` before it grows (it is at 409 lines):

- **`game::state.rs`** — `pub enum GameMode { Play, Transition(Transition) }`
  (Title/Pause arrive in Phase 4; keep the enum, additive later).
  `Transition { kind: Fade, t: u8, target: MapId, entry: u8 }`: 16 ticks fade
  out (full-screen black rect alpha ramp via `Draw::rect` after everything
  else), swap map at the midpoint, 16 ticks fade in. Input ignored during
  transitions. World is rebuilt, not mutated:

```rust
// game::state
pub fn switch_map(game: &mut Game, target: MapId, entry: u8) {
    // 1. extract PlayerPersist from the current world
    // 2. World::new(content::maps::build(target), entry_pos)
    // 3. re-apply PlayerPersist; spawner::populate(&mut world); camera.snap_to
}
pub struct PlayerPersist { hearts, max_hearts, energy, rupees, style_points, style_rank /* + gems/flags via SaveGame */ }
```

- **Door triggers**: each tick (Play mode), test the player's feet tile
  against `map.triggers`. Door fires on overlap → start Transition. Re-entry
  guard: the destination `EntryPoint` must place the player **outside** the
  return-door's rect (author maps accordingly; assert in a debug check that
  scans every Door's target entry at startup).
- **Interior stubs**: `content/src/maps/interiors.rs` — one generic 12×10
  house room (walls, floor, exit Door back to `Overworld` at the correct
  village entry), used for `House(0..=5)`; a 16×12 `Shop` shell; two 14×12
  cave shells `Cave(0)`/`Cave(1)` (grotto gets the fountain spawn). Bare but
  navigable — 2B furnishes.
- **`game::world::spawner.rs`** — owns `Vec<SpawnSlot { def, state }]`
  populated from `MapDef.spawns` at map load.
  - Activation: every 16 ticks, spawn defs whose distance to the player is
    < 480 px and state == Dormant (uses existing `slime::spawn` etc. — map
    `SpawnKind` → those calls; `FairyFountain`/`Dummy` spawn immediately at load).
  - Sleep: enemy AI update skips entities farther than 420 px from the player
    (add the distance check in `enemies::update`'s dispatch loop — cheap and
    contained).
  - Death: mark slot Dead; respawn to Dormant when the player has been > 720 px
    away for 600 ticks (re-entry pressure without farm-in-place).
- **Arena scoping**: `WaveDirector` runs only when `current_map == MapId::Arena`.
  Game starts on `MapId::Overworld` at entry 0. Keep the arena reachable via a
  debug: with F1 overlay on, `H` still fires the debug shot; add **F3
  (`DEBUG_MAP`, new debug index in `engine::input`)** cycling
  Overworld→Arena→Overworld with the normal transition (also your fastest
  regression check that Phase 1 combat still feels right).
- **Debug teleport**: with overlay on, F3+held-direction? No — keep simple:
  F4 (`DEBUG_TELEPORT`) cycles the player through all `EntryPoint`s of the
  current map. Essential for the M4 collision sweep and for 2B.
- **Save v2** (`game::save_data`):

```rust
pub struct SaveGame {
    pub version: u32,            // = 2
    pub map: u8, pub entry: u8,  // MapId encoded ↔ u8 helpers in content::maps
    pub checkpoint: u8,          // last Checkpoint id touched (overworld entry)
    pub hearts: i32, pub max_hearts: i32, pub rupees: u32,
    pub gems: u8,                // bitmask, written by 2B+; carried now
    pub flags: Vec<u16>,         // generic world flags (secrets, doors); 2B defines ids
}
```

  Autosave keeps the 60-tick cadence but now stores checkpoint-based position
  (map + entry), not raw x/y — respawn/death and reload both go to the last
  checkpoint (replaces the hardcoded `(72,88)` fountain respawn; fairy-rescue
  toast stays). Old v1 JSON fails parse → default New Game (accepted, note it).
  Checkpoint trigger: on overlap, if `checkpoint != id`, save immediately +
  toast "CHECKPOINT" + `SfxId::CheckpointChime` (new).

DoD for M5: enter/exit every stub interior with clean fades; die in the grove
→ respawn at grove checkpoint with rupees intact; reload mid-map → resume at
last checkpoint; arena via F3 still runs waves; overworld runs zero waves.

### M6 — Camera feel + perf pass + seam freeze

- Camera (keep `Camera` API): add a **soft dead-zone** — target moves the
  camera only when outside a 24×16 px box around the camera center; lookahead
  eases toward `facing.unit() * 16` at rate 0.08/tick instead of snapping
  12 px; keep 0.15 follow lerp and the existing map clamps and shake. Result:
  standing still and small strafes don't swim, turning leads smoothly
  (Minish-Cap read). Numbers are starting points — tune ±30% and log.
- Region banner (minimal, 2B skins): on `Banner` trigger for a region not
  bannered in the last 1800 ticks, draw the region name centered near the top
  (`Draw::text` + a dark rect underlay), fade in 20 / hold 90 / fade out 30
  ticks. Owned by `game::ui` (new `banner.rs`), driven by a
  `WorldEvent::RegionEntered(u8)` (additive variant).
- Perf: with F1 overlay, run the full map perimeter + through the village;
  entity count stays bounded (spawner sleep working), fps ~60, chunk bakes
  amortized. If iPhone-class throughput is in doubt, verify total
  drawImage/frame ≤ ~350 (9 chunk blits + entities + animated tiles + HUD).
- Freeze the 2B seam contract (see §5) in your WORKER_NOTES completion entry.

## 5. File ownership + the 2B seam contract

You create/own in 2A:
- `content/src/maps/` — `mod.rs` (v2 types), `catalog.rs`, `paint.rs`,
  `arena.rs` (ported), `overworld/*.rs` (7 modules), `interiors.rs` (stubs).
- `content/src/art/tiles_terrain.rs`, `tiles_water.rs`, `tiles_forest.rs`
  (+ palette additions, `all_bakes` registrations).
- `engine/src/chunks.rs` + `Draw` chunk methods + `DEBUG_MAP`/`DEBUG_TELEPORT`
  input indices.
- `game/src/state.rs`, `game/src/world/spawner.rs`, rewritten
  `game/src/draw_world.rs`, extended `physics.rs`, `camera.rs` tune,
  `save_data.rs` v2, `game/src/ui/banner.rs`, `lib.rs` rewiring.

Seams you must freeze for 2B (their brief assumes exactly this):
1. `MapDef` v2 fields + painter helpers + `catalog::tile_info` + tile id
   ranges (2B appends ids in the reserved ranges and new art files).
2. `SpawnKind` / `TriggerKind` / `RegionDef` / `EntryPoint` — 2B appends enum
   variants (Npc, Sign, Gem, Chest, Secret, Dialogue) and handles every match.
3. `content::maps::build(MapId)` single loader + `MapId` u8 encoding.
4. `game::state::switch_map` + `PlayerPersist` + Transition fade.
5. `spawner::SpawnSlot` lifecycle (Dormant/Alive/Dead + respawn rule).
6. `World::set_tile` + dirty-chunk invalidation.
7. `SaveGame` v2 fields (`gems`, `flags` are 2B's to write).
8. Chunk cache API + the animated-tile overdraw list.
9. `WorldEvent::RegionEntered` + banner UI hook.
10. Debug F3 (map cycle) / F4 (teleport entries).

## 6. Definition of Done

1. Game boots into the **240×240 overworld** at the village south gate; walk
   to all six named regions and back, zero loading, zero screen transitions
   outdoors, ~60 fps on a mid laptop (F1 numbers in the completion note).
2. Chunked rendering live (F1 shows chunk stats); water/fountain animate;
   no visible chunk seams or stale chunks after `set_tile` (test via debug).
3. Collision v2: water and cliffs block, paths guide, ≥3 working one-way
   ledge hops in Razor Cliffs; a full-map walk sweep finds no traps or
   walk-through walls.
4. All six regions have shells, name banners on entry, checkpoints, and the
   normative connection topology (river + 2 bridges + broken bridge, roads).
5. Doors: 6 houses + shop + 2 caves enter/exit with fades; entry points never
   re-trigger their own door.
6. Save v2: death → last checkpoint; reload → last checkpoint; rupees/hearts
   persist; arena (F3) still plays Phase 1 waves; overworld has
   distance-activated placeholder encounters that sleep when far.
7. `cargo check` + `clippy -D warnings` (wasm32) clean; `env -u NO_COLOR trunk
   build --release` emits a working dist; no file >~600 lines.
8. WORKER_NOTES completion entry: what landed, deviations, tuning logs, perf
   numbers, and the **frozen 2B seam table** (§5 statuses).

## 7. Verification protocol

- After every milestone: `cargo check --workspace --target
  wasm32-unknown-unknown && cargo clippy --workspace --target
  wasm32-unknown-unknown -- -D warnings`.
- Playwright smoke against `python3 -m http.server 8090 --directory dist`
  (pattern in WORKER_NOTES Phase 1 entries): boot to overworld, hold ArrowUp
  ~10 s and screenshot (terrain + banner), F1 screenshot (fps/chunk stats),
  walk into a house door and screenshot the interior, localStorage roundtrip.
- Manual feel pass in a real browser: camera dead-zone (no swim when idle),
  ledge hop, river/bridge reads, full perimeter run watching the F1 fps.
