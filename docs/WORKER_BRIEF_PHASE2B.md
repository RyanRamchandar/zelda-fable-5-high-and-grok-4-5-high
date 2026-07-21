# WORKER_BRIEF_PHASE2B.md — Act 1 Overworld Content Fill (POIs, NPCs, encounters, minimap, secrets)

You are a Grok worker. This brief is self-contained; follow it exactly. Conflict
order: `DECISIONS.md` → `ARCHITECTURE.md` → this brief → `GAME_DESIGN.md`.
You run autonomously — do not ask the human anything except for credentials.

## 0. Mission

Phase 2A shipped the 240×240 contiguous overworld skeleton: layered chunked
tiles, six region shells, doors + interior stubs, spawner, checkpoints, save
v2, banners. You make it a **place**: region-distinct props and decoration,
furnished interiors, **NPCs and signs with a dialogue box**, the **three gems
as guarded objectives** with a gated shrine door (soft critical path), tuned
**encounters from the Phase 1 roster** (slime / bat / octorok), the
**overworld minimap** (corner + pause map, fog reveal, POI icons), **8–10
telegraphed secrets** (hooks — some open now, some await bombs/boomerang), and
interaction plumbing (Interact button, chests, gem pedestals).

Not yours: gem *puzzles* (wind chimes, plate court) and the camp *wave battle*
mechanics beyond what the encounter system gives you, shop *economy UI*, new
enemy families (wisp/skeleton/raiders), bombs/boomerang. Those are Phase 2C/3
— you place their locations and flags so they slot in without map surgery.

## 1. HARD DEPENDENCY — Phase 2A must be merged first

Do not start until `docs/WORKER_NOTES.md` contains the **"Phase 2A
completion"** entry and its commits are on `main`. Read that entry first —
where it differs from this brief, **code + 2A notes win**; log drift and adapt.

Seams 2A froze for you (verify each in code before building on it):

1. `content::maps` v2 — `MapDef { ground, detail, overhang, collision, spawns,
   triggers, regions, entries }`, `flags::*` bits, painter helpers
   (`paint.rs`: path/blob/scatter/stamp), `catalog::tile_info` + reserved tile
   id ranges (you append ids in-range; document each).
2. `SpawnKind` / `TriggerKind` enums — **you append variants** (`Npc`, `Sign`,
   `Gem`, `Chest`, `Secret`, `Dialogue`); every `match` in game is exhaustive,
   the compiler walks you to each site.
3. `content::maps::build(MapId)` + `MapId` (Overworld, Arena, House(u8), Shop,
   Cave(u8)) — you may append `MapId` variants if you add interiors.
4. `game::state::switch_map` + `PlayerPersist` + fade Transition.
5. `game::world::spawner` — SpawnSlot Dormant/Alive/Dead lifecycle,
   distance activation ~480 px, sleep ~420 px, respawn after 600 ticks
   beyond 720 px.
6. `World::set_tile(layer, tx, ty, id)` + dirty-chunk invalidation — your
   secrets use this to open walls/reveal caves.
7. `SaveGame` v2 — `gems: u8` bitmask and `flags: Vec<u16>` are yours to
   define ids for (see §4 flag registry).
8. Chunk cache + animated-tile overdraw list (new animated tiles you add must
   register frames/rate in the catalog only — the pipeline handles the rest).
9. `WorldEvent::RegionEntered(u8)` + `game::ui::banner` (you may skin it).
10. Debug: F1 overlay, F2 sprite viewer, F3 map cycle, F4 entry teleport.

Phase 1 seams still in force: `WorldEvent` messaging only (no cross-module
calls), `combat::resolve_hits` damage path, `slime/bat/octorok::spawn`,
`FxKind::Toast`, `SfxId` + `spec()` appends, `ui::hud` layout consts,
`BUTTON_INTERACT = 3` (E / pad Y) is free for you.

## 2. Locked constraints

- Stack per DECISIONS.md; no new crates; `content` depends on nothing; `game`
  stays web-sys-free; art = handcrafted indexed grids only.
- Respect ownership (ARCHITECTURE §11): dialogue/minimap/banners in
  `game::ui`; interaction/chests in `game::items` or a new `game::interact`
  module (your call — log it); all text in `content/src/text.rs`; all
  placement data in `content::maps`.
- No file >~600 lines. Region modules will grow — split decoration into
  sibling files (`village_deco.rs`) before they blow the cap.
- Commits on `main`, prefixed `phase2b:`, small and frequent. No push, no
  deploy. Every commit: `cargo check --workspace --target
  wasm32-unknown-unknown` + `cargo clippy ... -- -D warnings` clean.
- Tuning ±30% with a WORKER_NOTES log. Blocked → WORKER_NOTES + nearest
  non-blocking interpretation.

## 3. Milestones (commit at least once each)

### M1 — Interaction seam: Interact verb, signs, dialogue box

- `content/src/text.rs`: `pub enum TextId` + `pub fn text(id: TextId) ->
  &'static [&'static str]` (pages of ≤3 lines × ~38 chars; author everything
  here, no literals in game code).
- New trigger/spawn plumbing: `SpawnKind::Sign { text: TextId }` and
  `SpawnKind::Npc { npc: NpcId }` spawn `EntityKind::{Sign, Npc}` (additive
  variants + `EntityData::{Sign(..), Npc(..)}`). Both have solid 16×16 bodies,
  no health. NPCs are stubs: 2-frame idle anim, face the player when in range,
  no pathing.
- **Interact**: when the player is within 20 px of a Sign/Npc front side and
  presses Interact (`BUTTON_INTERACT`), open the dialogue box. While open:
  world updates pause (reuse the viewer's pause pattern in `Game::update`),
  Attack/Interact advances a page, last page closes. `game::ui::dialog.rs`:
  bottom panel (rect + `Draw::text`, toast-panel art style), 2 ticks/char
  typewriter, `SfxId::TextBlip` (append) every 2 chars, input edge-guarded.
- A small "!"-style prompt marker floats above interactables in range
  (`fx` or draw inline in `draw_world::render_entity` — keep it cheap).
- `EntityKind::Chest` + `SpawnKind::Chest { flag: u16, loot: Loot }` where
  `Loot = Rupees(u32) | HeartPiece | Gem(u8)`. Interact opens: flip lid anim,
  set save flag, toast + jingle, spawn pickups (rupees fly out via existing
  pickup magnetism) or run the gem beat (M4). Opened chests persist via flag.
- HUD: rupee counter (icon + count near the item slot — hud consts exist,
  extend them; pickups already increment `pd.rupees`).

DoD: place one test sign + npc + chest in the village; full talk/open flow
with save-persisted chest state.

### M2 — Region decoration + furnished interiors (art + placement)

Art files (append tile ids in 2A's reserved ranges; register in `all_bakes`;
run the startup sprite-key assertion): `props_village.rs` (lantern 2-frame
glow, house walls/roofs/door frames, market stall, fountain basin, flower
beds), `props_wild.rs` (camp tents, palisade, bonfire 2-frame, watchtower,
ruins columns/arches/rubble, chime stand, pedestal, cracked wall, cave mouth,
birds-circling marker 2-frame), `props_interior.rs` (floorboards, rug, table,
bed, shelves, counter, pots), `npcs.rs` (elder, shopkeeper, 3 villagers, kid —
16×24, 2-frame idle, palette-swap variants welcome).

Placement per region (edit 2A's `overworld/*.rs` modules; use detail/overhang
layers; keep walkways ≥2 tiles wide):

- **Mosslight Village**: 6 houses + shop get distinct facades; lantern-lined
  paths (animated glow); plaza fountain; flower beds; 4 signs (village
  welcome, shop, "north: shrine" waypost, fountain lore); NPCs: elder (intro
  quest dialogue: "recover the three gems", sets flag QUEST_STARTED and the
  objective, see M5), shopkeeper (inside shop: "stock arrives soon" stub —
  economy is 2C), 3 villagers + kid with flavor+hint lines (each hints one
  real secret or region).
- **Interiors**: furnish all 6 houses (distinct layouts: beds/tables/shelves),
  shop (counter + shelf dressing + shopkeeper), fairy grotto `Cave(0)`
  (fountain spawn + shimmer detail tiles), cliffs cave `Cave(1)` (winding
  passage → heart-piece chest, see M6).
- **Whispering Grove**: canopy density up; glades with flower scatter; the NE
  clearing gets the **Courage Gem pedestal** dressing + 3 chime-stand props
  (inert — puzzle is 2C; sign: "the chimes answer a single gale…"); fairy-
  grotto cave mouth dressed.
- **Ashen Raider Camp**: tents, bonfires (animated), banners, watchtower,
  scattered crates/barricade props (destructibility is 2C — solid for now);
  war-chest clearing gets the **Power Gem chest** (M4).
- **Echoing Ruins**: columns/arches (overhang tops), rubble, sand drifts,
  lore tablet sign ("three gems, three virtues…"); plate-court plaza dressed
  with inert plate tiles + the **Wisdom Gem pedestal** (M4).
- **Razor Cliffs**: scrub/rock scatter, birds-circling marker above the
  cracked wall (M6 secret), summit vista dressing.
- **Triforce Shrine**: pedestal trio at the sealed door, braziers, vista
  terrace dressing, lore sign explaining the three-gem seal.

DoD: a walk through every region reads visually distinct at a glance (F2-style
screenshot set in the completion note); no walkway pinches <2 tiles; fps ~60.

### M3 — Encounters: placed Phase 1 enemies with intent

Replace 2A's placeholder spawns with authored encounters (all data in
`overworld/*.rs` SpawnDefs; tuning deltas in `combat::tuning`):

- **South meadow / roads**: sparse singles (2–3 slimes, 1 bat) — safe-ish
  spine roads, danger off-road.
- **Whispering Grove**: bats at maze corners (ambush over corridors), slime
  pockets in dead-ends guarding rupee scatter; ~10–14 total.
- **Ashen Raider Camp**: densest melee pocket — 3 clusters of 3–4 slimes+bats
  around bonfires; the war-chest clearing spawns a 6-pack guard `group` that
  must be cleared to open the chest (see M4); ~14–18 total.
- **Echoing Ruins**: octorok lanes firing down colonnades (cover via columns),
  bats between arches; ~10–14.
- **Razor Cliffs**: octoroks on upper bands lobbing at switchbacks below
  (their rocks already outrange the player's reach — the *route* is the
  counterplay), slimes on landings; ~10–14.
- **Shrine approach**: 2 tough sentinel groups framing the road (the "soft
  gate" — skilled players can run past, fighting is the honest path).
- Village + interiors + a 12-tile radius around every checkpoint and door:
  **zero hostile spawns**.

Group support in `spawner`: `SpawnDef.group` (exists) gains meaning — a
`group_cleared(world, group) -> bool` helper (all slots of the group Dead) and
a rule that grouped slots only respawn when the whole group is Dormant-
eligible, so "clear the camp guard" is a stable state. Emit
`WorldEvent::GroupCleared(u16)` (additive) when the last member dies.

Total placed enemies ~60–80 SpawnDefs; the activation/sleep radii keep alive
counts small — verify ≤12 active AI in the worst pocket (F1 count).

DoD: each region's encounter identity matches the list; camp guard group
clears and stays cleared until respawn rule; checkpoint/door safe zones hold.

### M4 — The three gems + shrine soft gate (critical path)

Gems are **guarded objectives now, puzzle-gated later** (2C swaps the guard
for the real mechanism without moving anything):

- `SpawnKind::Gem { id: 0|1|2 }` → `EntityKind::Gem` on a pedestal/chest per
  region: **Courage** on the grove pedestal, **Power** in the camp war-chest
  (locked until its guard `group` is cleared — listens for `GroupCleared`),
  **Wisdom** on the ruins plate-court pedestal (ringed by the ruins' densest
  octorok+bat placement).
- Gem pickup beat: interact (pedestal) or open (chest) → hold-up flash +
  "COURAGE GEM" toast + `SfxId::GemGet` (append, distinct fanfare-ish sweep) +
  set `SaveGame.gems` bit + **checkpoint save** (GAME_DESIGN §8: checkpoint at
  each gem) + objective update (M5).
- **Shrine door**: sealed-door tiles + a `TriggerKind::Secret`-style check —
  interact with <3 gems → dialogue ("the seal holds… N of 3 gems"); with 3 →
  door-open moment: `World::set_tile` swaps the door tiles open (dirty-chunk
  seam proves itself here), rumble shake, `SfxId::SealOpen` (append), toast.
  Beyond it: a small antechamber Door → `MapId::ShrineLobby` (new interior
  stub: "the dungeon lies beyond" sign) — Phase 3 replaces the stub with the
  real dungeon entrance. Persist door state via flag DOOR_SHRINE_OPEN.
- Objective flag registry: define `pub mod save_flags` in `game::save_data`
  with documented u16 constants (QUEST_STARTED, CHEST_* per chest,
  SECRET_* per secret, DOOR_SHRINE_OPEN, HEART_PIECE_* …). Single registry,
  no magic numbers at call sites.

DoD: full soft critical path playable with Phase 1 verbs only: village elder →
grove gem → camp fight+chest gem → ruins gem → shrine door opens → lobby stub.
Reload at any point resumes correctly (gems/flags/checkpoint).

### M5 — Overworld minimap: fog, POIs, objective

`game::ui::minimap.rs` (+ `MinimapState` on `UiState`):

- **Fog data**: overworld-only grid at 4×4-tile resolution (60×60 cells).
  Reveal cells within a 10-cell radius of the player each 8 ticks. Serialize
  into `SaveGame` as a compact bitset (`Vec<u32>`, 113 words — extend save,
  version stays 2, note it in WORKER_NOTES; parse-failure → New Game rule
  already covers migration).
- **Corner minimap** (toggleable M key + shows by default): 64×64 px panel,
  top-right (don't collide with hud consts — hearts are top-left, item slot
  bottom-right). Rendering: revealed cells → 1 px per cell colored by a
  terrain-class lookup (build a 60×60 `Vec<u8>` class map once at map load
  from the tile catalog: water/forest/cliff/path/structure/grass). Unrevealed
  = dark. Player = blinking white dot; camera-view rectangle outline.
  Cache the terrain pixels into a pre-rendered chunk? No — 3.6k rects at 1 px
  is too many `fill_rect`s; instead draw runs (consecutive same-color cells in
  a row = one rect). Budget ≤ ~400 rects; if over, halve resolution (log it).
- **Pause map** (Enter/`BUTTON_PAUSE`): full-screen 180×180 px version of the
  same data (3 px/cell), region names lettered on revealed regions, POI icons,
  legend, "resume" hint. Gameplay pauses (same pause pattern as dialog).
- **POI icons** (6×6 sprites in `props_*` art): shop, fairy fountain(s),
  shrine, gem sites (until claimed → then checkmark), discovered secrets.
  Shop/shrine/fountain icons appear once their cell is revealed; gem icons
  always visible from quest start (they are the objective); secret icons only
  after discovery.
- **Objective marker**: `UiState.objective: Option<(tx, ty, TextId)>` driven
  by quest flags (elder → nearest unclaimed gem, 3 gems → shrine). Star icon
  on both maps + edge-of-minimap arrow when off-panel. Update on GemGet /
  QUEST_STARTED / door-open events.

DoD: fog reveals as you explore and survives reload; both maps legible at
gameplay scale (screenshots); objective marker tracks the M4 progression.

### M6 — Secrets (8–10, telegraphed) + heart pieces + polish sweep

Every secret gets: a **telegraph** (visual tell or NPC/sign hint), a save
flag, a minimap icon on discovery, and `SfxId::SecretChime` (append; the
classic "found it" cue). Implement `TriggerKind::Secret { flag: u16 }` firing
once on entry.

Ship exactly these (locations may shift ±10 tiles; log):
1. Grove bomb-wall cave (rupee cache) — cracked-wall art + kid NPC hint;
   **inert until bombs (2C/3)**: interacting says "it sounds hollow…"; flag
   reserved.
2. Cliffs hidden cave → **heart piece #1** chest (`Cave(1)`, exists from 2A)
   — birds-circling telegraph above the cracked entrance; this one is OPEN
   (walk-in behind a waterfall-ish cliff notch — no tool needed).
3. Ruins collapsed cellar: pushable-looking rubble tile you can walk behind
   (perspective trick) → rupee cache + **heart piece #2**.
4. River island reachable by hopping LEDGE tiles from the broken-bridge bank
   → rupee cache; villager hints "the river hides a path when you look down".
5. Village: pay-attention secret — walk behind the shop (gap in the hedge) →
   chest with 20₹.
6. Grove: lone discolored tree in a dead-end (distinct trunk tile) →
   walk-through → glade with fairy fountain #2 + **heart piece #3**.
7. Camp: watchtower back ladder tile → top → chest (50₹) + vantage dialogue.
8. Cliffs summit vista: interact at the marked stone → "the shrine watches
   over the valley" + 30₹ scenic reward (teaches interact-on-landmarks).
9. Meadow: ring of flowers; stand in center 3 s → fairy + energy refill +
   flag (hinted by kid NPC).
10. Shrine terrace: brazier pair — interact both within 5 s → chest 50₹
    (foreshadows the boss crystal pairing; sign hints "twin flames answer
    together").

Heart pieces: 3 placed (4th comes from the shop in 2C) — `Loot::HeartPiece`,
collect 4 → +1 max heart (store count in save flags; HUD already renders
`max/2` hearts — verify odd-count rendering).

Polish sweep to close the phase: banner skin (panel art instead of bare
rect), drifting-leaf particles in grove/village (`fx` ambient emitter, cap
~20), bonfire ember particles in camp, region-appropriate palette-swap checks,
and a full-map walkthrough fixing any collision/readability bugs found.

## 4. File ownership

Yours: `content/src/text.rs`, `content/src/art/{props_village,props_wild,
props_interior,npcs}.rs`, placement edits inside `content/src/maps/overworld/
*.rs` + `interiors.rs` (+ `ShrineLobby`), appended enum variants + their match
arms across `game`, `game/src/ui/{dialog,minimap}.rs` + banner skin,
`game/src/interact.rs` (or items/), spawner group logic, `save_data` flag
registry + fog bitset, SFX id appends.

Not yours (do not restructure): 2A's chunk/render/physics/state/camera code,
Phase 1 combat/player/fx internals, `engine` (except nothing — you should not
need engine changes at all; if you think you do, WORKER_NOTES first).

## 5. Definition of Done

1. Critical path: New Game → elder quest → 3 gems (grove pedestal, camp
   guarded chest, ruins pedestal) → shrine door opens → lobby stub; playable
   with keyboard alone, no console errors, reload-safe at every step.
2. All six regions decorated and visually distinct; all interiors furnished;
   ≥6 NPCs/signs-with-personality in the village + ≥4 wayfinding/lore signs
   in the wild; dialogue box with typewriter + blip.
3. Encounters per M3 identity table; camp guard group gate works; safe zones
   hold; ≤12 active AI worst case; ~60 fps sustained (F1 evidence).
4. Minimap: corner + pause, fog reveal persisted in save, POI icons, working
   objective marker through the whole M4 progression.
5. All 10 secrets in with telegraphs + flags + chime; 3 heart pieces
   collectible; 4-piece → max-heart logic verified (via debug flag set).
6. Rupee counter on HUD; chests persist; `SaveGame` roundtrips gems, flags,
   fog, checkpoint (manual localStorage inspection + reload test).
7. `cargo check` + `clippy -D warnings` (wasm32) clean; `env -u NO_COLOR
   trunk build --release` works; no file >~600 lines.
8. WORKER_NOTES completion entry: landed/deviations/tuning log, screenshot
   list, flag registry summary, and explicit statement of what is stubbed
   for 2C/3 (chimes, plates, barricades, shop economy, bomb walls, broken-
   bridge crank).

## 6. Verification protocol

- Per milestone: wasm32 check + clippy, then a targeted Playwright smoke vs
  `python3 -m http.server 8090 --directory dist` (boot, walk a scripted path,
  screenshot; interact-key flows for dialog/chest; localStorage assertions
  for flags/fog).
- Full pre-completion run: scripted keyboard playthrough of the M4 critical
  path with screenshots at each gem + the shrine opening; a reload mid-run
  proving resume; F1 perf capture in the camp (densest pocket).
- Manual feel pass: dialogue pacing, minimap legibility at 2× scale, secret
  telegraphs actually readable, encounter difficulty (log ±30% tunings).
