# WORKER_BRIEF_PHASE2C.md — Act 1 interactivity: puzzle curriculum, bombs + shop economy, enemy depth

You are a Grok worker. This brief is self-contained; follow it exactly. Conflict
order: `DECISIONS.md` → `ARCHITECTURE.md` → this brief → `GAME_DESIGN.md`.
You run autonomously — do not ask the human anything except for credentials.

## 0. Mission

Phase 2B shipped the soft critical path (elder → 3 guarded gems → shrine seal),
minimap, secrets, and placed every 2C location as an inert stub. You replace the
interim "guarded objective" gem gates with the **real puzzle curriculum**, bring
**bombs + the B-item seam** online, open the **shop economy** (incl. heart piece
#4), make **bomb walls / barricades / the broken-bridge crank** live, and land
the **four missing Act 1 enemy families** (raider spear, raider torch, wisp,
skeleton) with a proper 3-wave camp battle.

This phase is split into two **sequential** parts. Do not interleave them:

- **Part 2C-A** — systems + puzzles + economy: `game::puzzle` tile-interaction
  layer, grove chime curriculum, ruins plate court, bombs + B-item controls,
  bomb wall + cave, barricade destructibility, shop UI + heart piece #4,
  broken-bridge crank.
- **Part 2C-B** (starts only after the 2C-A completion entry is in
  `docs/WORKER_NOTES.md`) — enemy families + camp wave battle + encounter
  re-dress + feel pass.

**Design keystone (planner decision):** the Gale Boomerang does not exist until
Phase 3 (it is *inside* the shrine the gems open), so every 2C puzzle must be
solvable with the current kit: sword slash/spin, dash, shield/perfect block,
sword beam at full hearts, and (new) bombs. Build the puzzle layer so that a
boomerang hit later satisfies the same "hit" checks — Phase 3 retro-enables
one-throw chime solves and easy crank flips for free. Do not special-case the
boomerang anywhere; it is just a future source of the same hit events.

## 1. HARD DEPENDENCY — code reality (verify each before building)

HEAD is `ea4edd9` (phase2b). Where this brief differs from code + the 2A/2B
completion notes, **code + notes win**; log drift in WORKER_NOTES and adapt.

Facts the brief is written against:

1. **No entity-vs-entity solidity exists.** `physics::move_entity` only
   resolves AABB-vs-tile (`crates/game/src/world/physics.rs`); "solid" entity
   bodies (signs, NPCs, chests) do not block movement. Therefore **all 2C
   puzzle props that must block or be walked on are tiles**, mutated via
   `World::set_tile(layer, tx, ty, id)` (dirty-chunk invalidation is automatic,
   including animated-tile re-registration). Do not build pushable-block or
   gate *entities*.
2. **Player attack surface**: `world.active_attacks: Vec<ActiveAttack>` is
   filled by `player::sword` during a swing and **cleared at the end of
   `combat::resolve_hits`**. The update order in `game/src/lib.rs` is:
   `player::update` → `interact::update` → `spawner.update` →
   `enemies::update[_no_waves]` → `integrate_non_player` →
   `combat::resolve_hits` → `items::update` → `fx::update` → banners/triggers/
   death/camera. Your `puzzle::update` must be inserted **after
   `integrate_non_player` and before `combat::resolve_hits`** so it can read
   `active_attacks` non-destructively.
3. **Projectiles**: player-owned = `EntityKind::SwordBeam` (`EntityData::Beam`,
   `from_player`) and reflected `EntityKind::OctorokRock` (`EntityData::Rock`,
   `from_player` true after perfect block). Both are entities you can position-
   test against tiles each tick.
4. **Flags**: `content::flags` (re-exported as `game::save_data::save_flags`)
   is the only registry; `has_flag(&[u16], id)` / `set_flag(&mut Vec<u16>, id)`
   in `save_data.rs`. Used ids: 1, 10–19, 30–39, 50, 60–64, 70–72, 80; group
   ids 10/30/40/41/50/60/70. You append (see §3 M1 / §4).
5. **Save**: `SaveGame` v2 (`save_data.rs`) — add new fields **only** with
   `#[serde(default)]` (the `fog` field is the precedent); version stays 2.
   `state::save_from_game(game)` builds it; `PlayerPersist` in `state.rs`
   carries player fields across `switch_map` — anything you add to the player
   (bombs etc.) must flow through **both** or it silently drops on any door.
6. **Interact**: `game/src/interact.rs` (415 lines) — `nearest_interactable`
   (28 px, front-biased), `interact_entity` dispatch, `try_shrine_seal` shows
   the door-open pattern (`set_tile` + push Door `TriggerDef` + flag +
   restore-on-load via `state::restore_shrine_door`). The Power-gem chest lock
   pattern (`PowerChestLocked` text until `GROUP_CAMP_GUARD` flag /
   `spawner::group_cleared(&game.spawner, 41)`) is your template for sealed
   gems.
7. **Spawner**: `world/spawner.rs` — `SpawnSlot` Dormant/Alive/Dead, distance
   activation 480 px, sleep 420 px, group respawn only when whole group Dead +
   far 600 ticks. `GroupCleared(u16)` fires when the last hostile of a group
   dies; `lib.rs drain_events` handles `GRP_CAMP_GUARD` → sets
   `GROUP_CAMP_GUARD` flag. Static kinds (Sign/Npc/Chest/Gem/Fountain/Dummy)
   spawn once in `Spawner::populate`; `apply_save` re-applies open/taken state.
8. **Controls** (`engine/src/input/`): `BUTTON_ATTACK=0` (J/Space, pad 0),
   `BUTTON_ITEM=1` (K, pad 2) — **currently hold-to-shield**, `BUTTON_DASH=2`
   (L/Shift, pad 1), `BUTTON_INTERACT=3` (E, pad 3), `BUTTON_PAUSE=4` (Esc,
   pad 9), `BUTTON_CONFIRM=5` (Enter), `BUTTON_COUNT=6`. Touch has joystick +
   Attack + Dash only. KeyM = minimap pulse. Buttons have `held`/`pressed`
   edges only (no released edge — derive release from your own state).
9. **Tile catalog** (`content/src/maps/catalog.rs`): ids used through 259.
   `tile_info(id)` gives sprite key / frames / anim_rate / collision flags;
   ground paints refresh collision from the catalog (`MapDef::set_ex`).
   You append ids **260–279** (2C interactives) and document each.
10. **`MapId::Cave` codec clamps to 2 caves**: `to_u8` = `20 + n.min(1)`,
    `from_u8` = `20..=21`. Your bomb cave is `Cave(2)` — extend to `n.min(2)`
    / `20..=22` and add the `build()` arm. Compiler-walked, low risk.
11. **File caps**: `game/src/lib.rs` is at 601 lines — you MUST extract before
    adding (see 2C-A M1). `interact.rs` 415, `draw_world.rs` 546,
    `state.rs` 328 — watch all of them; split before ~600.
12. **HUD consts** (`ui/hud.rs`): item slot at `ITEM_X=450, ITEM_Y=240,
    ITEM_S=20`, currently renders "—". Rupee counter at (8,26).
13. **Text**: all copy in `content/src/text.rs` (`TextId` + `text()` pages of
    ≤3 lines × ~38 chars). `TextId::ChimeSign` currently says "(puzzle
    sealed—return later)" — you rewrite it. No string literals in game code
    (HUD labels/toasts use `&'static str` consts — toasts are fine as-is).
14. **SFX**: append `SfxId` variants + `spec()` rows in
    `content/src/audio/sfx.rs`; the app adapter picks them up automatically.
15. **WaveDirector** (`enemies/waves.rs`) is Arena-only (F3 debug). Do NOT
    reuse it for the camp battle — the camp battle is spawner-group-driven
    (§4 M2).

## 2. Locked constraints

- Stack per DECISIONS.md; no new crates; `content` depends on nothing; `game`
  stays web-sys-free; art = handcrafted indexed grids only.
- Ownership (ARCHITECTURE §11): puzzle state machines in `game::puzzle` (new —
  reserved for you); shop menu in `game::ui::shop`; bombs in `game::items`;
  enemy AI one file per family in `game::enemies`; all placement/text/art/
  puzzle-site data in `content`.
- Engine changes allowed **only** as specified in 2C-A M3 (input cycle button).
  Anything else engine-side: WORKER_NOTES first, then nearest non-blocking
  interpretation.
- No file >~600 lines. Commits on `main`, prefixed `phase2c:`, small and
  frequent. No push, no deploy. Every commit: `cargo check --workspace
  --target wasm32-unknown-unknown` + `cargo clippy --workspace --target
  wasm32-unknown-unknown -- -D warnings` clean (use rustup `~/.cargo/bin`
  ahead of Homebrew rustc; `env -u NO_COLOR trunk build --release`).
- Tuning ±30% with a WORKER_NOTES log. Puzzle states must be **resettable**
  (reload never soft-locks: unsolved puzzles rebuild fresh from `MapDef` +
  puzzle defs; solved ones restore open via flags).

---

# PART 2C-A — Puzzle layer, bombs, economy (worker 1)

## 3. Milestones (commit at least once each)

### M1 — `game::puzzle` seam + tiles + headroom

**Headroom first**: move `spawn_debug_shot`, `debug_assert_door_entries`, and
`player_state_label` out of `game/src/lib.rs` into a new `game/src/debug.rs`
(pub(crate) fns, call sites updated). Target lib.rs ≤ 550 before you add lines.

**New catalog tiles (260–279)** + art (new `content/src/art/props_puzzle.rs`,
registered in `all_bakes()` — startup sprite-key assertion will catch misses):

| id | const | flags | notes |
|---|---|---|---|
| 260 | `T_GATE` | SOLID | wooden/stone gate, region-neutral |
| 261 | `T_BLOCK` | SOLID | pushable stone block, distinct chiseled look |
| 262 | `T_PLATE_UP` | walk | raised pressure plate |
| 263 | `T_PLATE_DOWN` | walk | pressed plate (visibly flush) |
| 264 | `T_BARRICADE` | SOLID | lashed crates/stakes, "breakable" cracks |
| 265 | `T_CRANK` | SOLID | 2-frame crank wheel (anim only while turning is fine as a static swap) |
| 266 | `T_BRIDGE_LOWERED` | walk | plank tile visually distinct from `T_BRIDGE_H` (reuse `bridge_h` sprite + darker palette row if easier — log) |

`T_CHIME` (247) already exists as a walkable detail tile — reuse it; ring
feedback is FX + SFX, not a tile swap.

**Puzzle site data in content** — new `content/src/puzzles.rs` (pure data, no
logic):

```rust
pub struct ChimeGateDef { pub chime: (u32, u32), pub gate: &'static [(u32, u32)], pub open_tile: u16, pub flag: u16 }
pub struct ChimeFinaleDef { pub chimes: [(u32, u32); 3], pub window_ticks: u32, pub flag: u16 }
pub struct PlateCourtDef { pub plates: [(u32, u32); 2], pub blocks: [(u32, u32); 2], pub gate: &'static [(u32, u32)], pub open_tile: u16, pub floor_tile: u16, pub flag: u16 }
pub struct BarricadeDef { pub tiles: &'static [(u32, u32)] }   // hp is a game-side tuning const
pub struct CrankDef { pub crank: (u32, u32), pub swaps: &'static [(u32, u32, u16)], pub flag: u16 }  // swaps: (tx, ty, new_ground_tile)
pub struct BombWallDef { pub wall: (u32, u32), pub open_tile: u16, pub door: (crate::maps::MapId, u8), pub flag: u16 }
pub fn overworld() -> /* struct or tuple bundling the above for MapId::Overworld */
```

(Exact shape is yours; keep it `&'static`/const-friendly, one accessor per
MapId like `maps::build`. Coordinates in §M2/M4/M5 tables.)

**Runtime module** — `game/src/puzzle/mod.rs` (+ `chimes.rs`, `plates.rs` as
needed):

- `PuzzleState` stored on `Game` (like `spawner`), rebuilt in
  `state::switch_map` and `Game::new` from `content::puzzles` for the current
  map. Unsolved sites paint their closed state **at load time** (gates closed,
  blocks at start positions, plates up, barricades intact, bridge broken) via
  `set_tile` — the `MapDef` painters stay as 2B left them, so puzzle layout
  lives in one place (`content::puzzles`) and reload = reset.
- `puzzle::restore(world, flags)` — for each solved flag, apply the open
  state (gate tiles open, bridge lowered, bomb wall open + Door trigger,
  chime gates open). Call it right next to `state::restore_shrine_door` in
  both `switch_map` and `Game::new`.
- `puzzle::update(game)` runs every tick (insertion point per §1.2):
  - **Sword-hit vs tile**: for each `ActiveAttack` (AABB or radius), test
    overlap against registered chime / barricade / crank tile rects
    (tile rect = 16×16 at `(tx*16, ty*16)`). Dedupe per swing with
    `(swing_id, tx, ty)` (mirror `world.hit_pairs` logic locally — do not
    reuse `mark_hit`, it keys on entity index).
  - **Projectile-hit vs tile**: for alive `SwordBeam` / `OctorokRock` with
    `from_player && !hit`, test center vs the same rects; on hit mark the
    projectile `hit = true` and despawn (beams) per the existing pattern in
    `combat::resolve_hits`.
  - **Plates**: pressed if the player's feet tile or a block occupies the
    plate tile. Swap `T_PLATE_UP`/`T_PLATE_DOWN` on change (+
    `SfxId::PlateClick`). All plates of a court pressed → open gate tiles
    (`set_tile` → `open_tile`) + `SfxId::GateOpen` + small shake; any plate
    released before the court's `flag` is set → close again (fair warning
    cue). Once the court flag is set the gate stays open forever.
  - **Blocks**: player pushing into a `T_BLOCK` ground tile (velocity toward
    it, contact sustained ≥ 8 ticks) slides it 1 tile in the push direction
    if the destination is walkable (no SOLID/WATER, not another block, in
    bounds). Move = `set_tile` old→`floor_tile`, new→`T_BLOCK`, +
    `SfxId::BlockSlide` + dust `FxKind::Dust`. Blocks are never persisted —
    reload resets them (that IS the puzzle reset).
  - **Chimes**: on hit → `SfxId::ChimeRing` + `FxKind::FountainSparkle`-style
    ring FX at the chime + record tick. Gate chimes: first ring sets the gate
    flag + opens its gate tiles permanently. Finale: all 3 chimes rung within
    `window_ticks` (start 240 ≈ 4 s) → solved flag + fanfare (reuse
    `SfxId::SealOpen`) + toast "THE SEAL FADES"; a chime "expiring" out of the
    window plays `SfxId::ChimeMiss` (dull detune) so failure is legible.
  - **Barricades**: per-tile hp (const `BARRICADE_HP: i32 = 3` in
    `combat::tuning`); sword hit = 1 (finisher/spin = 2), bomb blast =
    instant. On break: `set_tile` → the region's floor tile, debris
    `KillPoof`, `SfxId::BarricadeBreak`, 30% single rupee drop
    (`pickups::spawn_one`). Not persisted — barricades rebuild on reload
    (cover, not progress).
  - **Cranks**: any player hit (sword reach won't span the river gap — in
    practice beam / reflected rock now, boomerang in Phase 3) → apply
    `swaps` (`set_tile` each), set flag, `SfxId::CrankTurn` then rumble
    (reuse `SealOpen`) + `camera.add_shake(2.5, 12)` + toast "SHORTCUT
    OPEN!".
- Events out: use existing `WorldEvent::{Sfx, FxRequest}`; flags are set via
  `game.flags` directly (puzzle::update takes `&mut Game` like
  `interact::update`); solved-state saves push through
  `state::save_from_game(game).to_json()` → `game.pending_save` (existing
  seam).

**New flags** (append to `content/src/flags.rs`, 90+ block):
`PUZZLE_CHIME_GATE_1 = 90`, `PUZZLE_CHIME_GATE_2 = 91`, `PUZZLE_CHIMES_DONE =
92`, `PUZZLE_PLATES_DONE = 93`, `WALL_GROVE_OPEN = 94`, `BRIDGE_LOWERED = 95`,
`SHOP_BOMB_BAG = 96`, `SHOP_BOMBS_UNLOCKED = 97`.

**New SFX**: `ChimeRing`, `ChimeMiss`, `PlateClick`, `GateOpen`, `BlockSlide`,
`BarricadeBreak`, `CrankTurn`, `BombFuse`, `BombBoom`, `BuyItem`, `ItemCycle`.

DoD: a debug-placed test court (behind F1, or just the ruins court from M2 if
you prefer to go straight there) proves hit-detection, plates, block push,
gate open/close, and reload-reset.

### M2 — Grove chime curriculum + ruins plate court (the real gem gates)

**Grove (Courage Gem)** — edits in `content/src/maps/overworld/grove.rs` +
`content::puzzles`:

- **Teaching gate 1**: on the main SW→NE corridor (near path point (50,110)):
  a 2-tile `T_GATE` across the corridor + one `T_CHIME` beside it + a sign
  (`TextId::ChimeGateSign`: "A gale — or a keen edge — wakes the chime.").
  Hit chime → gate opens forever (`PUZZLE_CHIME_GATE_1`).
- **Teaching gate 2**: on the east corridor (near (74,110), the clearing
  approach): same pattern, chime placed 2–3 tiles from the gate so the player
  learns chimes act at a distance (`PUZZLE_CHIME_GATE_2`).
- **Finale** at the NE clearing (68–80 × 108–122, pedestal (74,114), chimes
  already at (70,112), (78,112), (74,118)): keep the three chimes where 2B
  put them (~8 tiles apart — deliberately NOT coverable by one spin; the
  player must dash between them inside the window; Phase 3's boomerang
  one-throw is the retro reward). Rewrite `TextId::ChimeSign`: "Three chimes,
  one breath of wind. Let none fall silent." Until `PUZZLE_CHIMES_DONE`, the
  **gem is sealed**: interacting with the Gem entity opens
  `TextId::CourageGemSealed` ("A ward of still air surrounds the gem…")
  instead of granting — add the check in `interact::take_gem_entity`
  mirroring the Power-chest lock, keyed on
  `game.current_map == MapId::Overworld && gem id 0 && !has_flag(PUZZLE_CHIMES_DONE)`.
  Render the seal: a translucent shimmer circle over the gem while sealed
  (draw in `draw_world::render_entity`'s Gem arm using a flag lookup passed
  via… simplest: add `sealed: bool` to `GemData`, set at populate/apply_save
  from the flag, cleared live by puzzle on solve — log the seam you choose).
- Solve beat: fanfare + toast + seal shimmer bursts; gem interact now runs
  the existing `grant_gem` path unchanged (checkpoint save included).

**Ruins (Wisdom Gem)** — edits in `overworld/ruins.rs` + `content::puzzles`:

- Delete the inert `T_RUBBLE` "plate" scatter in the court
  (188..208 × 142..158 step-4 loop).
- Build the court: `T_GATE` ring around the pedestal (198,150) — a fenced
  square roughly (194,146)–(202,154) with gate tiles on the west approach;
  2 plates at ~(190,145) and ~(206,155); 2 blocks starting at ~(192,151) and
  ~(202,147) (each ≤ 6 tiles of clean pushing from its plate; verify no
  column/sand-collision blocks the lanes — adjust the court floor with
  `T_SAND` fills as needed). `floor_tile = T_SAND`.
- Sign (replace or add near (192,146), `TextId::PlateCourtSign`): "Stone
  remembers weight. Two watchers must be held down at once."
- Both plates weighted (blocks only — the player being a third weight is a
  shortcut they can discover, that's fine) → gate opens; taking the gem sets
  nothing extra (`PUZZLE_PLATES_DONE` is set at gate-open) — after the flag,
  gates restore open on load.
- **Phase 3 hook**: one visibly unreachable switch alcove behind a water
  channel or column line NE of the court with a chest silhouette — place a
  `T_CRANK` + fenced `CHEST_RUINS_BONUS` (new flag 20, `Loot::Rupees(50)`)
  reachable only by lowering its 1-tile gate via the crank. Beam can hit it
  at full hearts (fine — a skilled preview), boomerang trivializes it later.
  Sign: `TextId::FarSwitchSign` ("Seen, not touched. Not yet.").
- Keep the octorok/bat placements — fighting around the court while pushing
  is the intended pressure. The pedestal checkpoint (id 4) stays.

DoD: with a fresh save, Courage requires the chime finale and Wisdom requires
the plate court; both solved states persist across reload; unsolved states
fully reset on reload; the old "walk up and take it" route is impossible.

### M3 — Bombs + B-item seam (controls decision — read carefully)

**Planner control decision** (logged in DECISIONS §5 addendum): shield stays a
core always-available verb on **hold Item (K / pad 2)**. The B-item fires on a
**tap** of Item: if the button is released within 8 ticks of press AND the
selected B-item is usable, the item fires on release. Perfect-block (first 6
shield ticks) is unaffected. **Cycle** is a new engine button:

- `engine/src/input/`: add `BUTTON_CYCLE: usize = 6`, `BUTTON_COUNT = 7`.
  Keyboard `KeyQ`; gamepad buttons 4 and 5 (LB/RB); touch: none yet (touch
  item parity is Phase 4 — log it). This is the ONLY engine change you may
  make. All `[Button; BUTTON_COUNT]` arrays resize automatically; check
  touch.rs `recompute_outputs` (it zeroes the whole array — fine).
- `PlayerData` additions: `bombs: u8`, `bomb_cap: u8` (0 until unlocked; 10
  after first purchase; 20 with bag), `selected_item: u8` (0 = none, 1 =
  bombs; boomerang will be 2 in Phase 3 — keep it a plain u8 in save, an
  enum in game if you like).
- Persistence: add `bombs`, `bomb_cap`, `selected_item` to **both**
  `PlayerPersist` (`state.rs`) and `SaveGame` (`#[serde(default)]`, version
  stays 2), wired through `from_player`/`apply`, `save_from_game`, and
  `Game::new` load. Miss one and bombs vanish on the first door — test it.
- Tap detection in `player::update_shield_and_dash_intent`: before the
  `!item` branch resets `shield_ticks`, detect release: previous
  `shield_ticks` in `1..=8` && bombs selected && `bombs > 0` && state is
  Idle/DashRecovery → place bomb at the player's feet tile center, decrement
  count, `SfxId::BombFuse` start.
- **Bomb entity**: `EntityKind::Bomb` + `EntityData::BombData { fuse: u16 }`
  (start 90 ticks), no body needed (walk-through), 2-frame sprite (new art in
  `props_puzzle.rs` or `enemies.rs`-style file) with flash rate accelerating
  over the last 30 ticks. Update in a new `game/src/items/bombs.rs` called
  from `items::update`:
  - fuse 0 → explosion: `FxKind::KillPoof`-scale burst (add a bigger
    `FxKind::Explosion { pos }` if you want — fx module is open to additive
    variants), `SfxId::BombBoom`, `camera.add_shake(3.0, 10)`.
  - Damage: push an `ActiveAttack { use_radius: true, radius: 24.0, damage:
    2.0, knockback: 3.0, kind: AttackKind::Bomb, owner: bomb id, swing_id:
    fresh from player swing counter or a bomb-local counter }` — resolved
    next tick by `resolve_hits` (1-tick delay is fine). Add
    `AttackKind::Bomb`; the exhaustive match in `combat::damage`'s style-verb
    mapping gets a no-style arm (bombs award no style).
  - Player self-damage: if player center within 28 px → push
    `WorldEvent::DamagedPlayer { amount: 2, dir: away }` (respects shield/
    i-frame rules automatically).
  - Tiles: any `T_CRACKED_WALL` ground tile whose center is within 28 px →
    open it (see bomb wall below); any `T_BARRICADE` in radius → break via
    the puzzle barricade path.
- Exhaustive-match sweep: `EntityKind::Bomb` arms in `integrate_non_player`
  (no-physics group), `enemies::update_enemies`, `resolve_hits` projectile
  match (None), `reflect_projectiles_near` / `destroy_hostile_projectiles_near`
  (no), `render_entity` (sprite + flash), spawner `is_hostile_kind` (no).
- **HUD item slot** (`ui/hud.rs`): when `bomb_cap > 0`, draw bomb icon +
  count in the existing slot (`ITEM_X/Y`), replacing "—"; brief cyan flash on
  cycle (`SfxId::ItemCycle`). Selected-item = bombs is the only state for
  now; `Q` with nothing unlocked plays `SfxId::Refused`.
- **Bomb wall goes live** (grove, wall at (30,185), `T_CRACKED_WALL`):
  blast → `set_tile` ground (30,185) → `T_CAVE_MOUTH` (walkable per catalog)
  + push `TriggerDef { tx:30, ty:185, w:1, h:1, Door { target: MapId::Cave(2),
  entry: 0 } }` + set `WALL_GROVE_OPEN` + `SecretChime` + toast "SECRET!" +
  minimap `mark_discovered_secret()`. Restore on load via `puzzle::restore`.
  Keep the 2B hollow-wall interact hint (`TextId::HollowWall`) only while the
  wall is closed.
  - **`Cave(2)` bomb cave**: extend the `MapId::Cave` codec (§1.10), add
    `interiors::cave_bomb()` — small `room(12,10, exit_entry 32)` variant
    with rubble dressing + chest `CHEST_GROVE_BOMB` (flag 18, already
    reserved) with `Loot::Rupees(100)`. Add overworld `EntryPoint { id: 32,
    tx: 30, ty: 187 }`. Run the door re-entry debug assert.

DoD: buy nothing yet (M4 gives the shop) — for testing, grant bombs via a
temporary debug hook behind F1 (remove or keep behind overlay; log). Tap-K
places a bomb, hold-K still shields, perfect block still works, blast hurts
enemies + player, opens the grove wall exactly once, cave chest pays 100₹,
everything survives reload and map switches.

### M4 — Shop economy + heart piece #4

`game/src/ui/shop.rs` — `ShopState { open: bool, cursor: usize }` on
`UiState`:

- Open: `interact_entity` NPC arm — `NpcId::Shopkeeper` now opens the shop
  menu (one greeting dialog page first is fine) instead of the stub line.
- While open: world sim pauses (add `|| game.ui.shop.open` beside the dialog/
  pause-map gates in `Game::update`; render after dialog, before fade).
- Menu: panel styled like the dialog box; rows with name, price, stock note;
  cursor via `move_vec.y` edges (track previous sign locally); confirm =
  Attack or Interact press; close = Pause or Dash press. All copy via new
  `TextId`s / a small `content::text::shop_lines()` table.
- **Stock** (GAME_DESIGN §8):
  | item | price | rule |
  |---|---|---|
  | Bombs ×5 | 10₹ | first purchase sets `SHOP_BOMBS_UNLOCKED`, `bomb_cap = 10`, `selected_item = bombs`; always restockable; clamped at cap; refuse at cap ("Your pouch is full.") |
  | Bomb Bag | 100₹ | once (`SHOP_BOMB_BAG`); `bomb_cap = 20` + 5 free bombs |
  | Heart Piece | 200₹ | once (`HEART_PIECE_4` flag, id 63 — already in the registry); run the same grant path as chests: `set_flag` + `maybe_apply_heart_container` + toasts (this is the 4th piece for most players → "MAX HEART UP!") |
  | Hero's Tunic | 300₹ | visible, greyed: "After the shrine's trial." (locked until a Phase 3 boss flag — reserve `TUNIC_UNLOCKED = 98`, do not implement) |
- Buying: rupee check (refuse + `SfxId::Refused` + flash), deduct from
  `pd.rupees`, `SfxId::BuyItem`, immediate save via `pending_save`.
- Shopkeeper flavor lines update per state (stock intro / after bag / after
  heart piece). Move the shop hint sign text off "stock arrives soon".
- **Economy audit** (do the math in WORKER_NOTES): placed sources today ≈
  5 + 20 + 25 + 30 + 50 + 50 (chests/landmarks) + your new 100 (bomb cave)
  + 50 (ruins bonus, boomerang-gated) + drops (~17% of kills, S-rank bonus).
  Sinks: 10×n + 100 + 200. Target: a player who does ~6 of 10 secrets can
  afford bag + heart piece + ~10 bomb restocks without grinding. Adjust cache
  sizes ±30% and log.

### M5 — Broken-bridge crank + polish + completion

- **Crank site** (`connective.rs` + `content::puzzles`): broken bridge at
  (64..69, 93..94). Place `T_CRANK` on the north bank at ~(67,91) (solid,
  clearly visible across the gap from the south approach). Crank hit →
  `swaps` replace the 4 gap water tiles (66..67, 93..94) with
  `T_BRIDGE_LOWERED` + the two `T_BRIDGE_BROKEN` end pairs stay as ramps
  (verify the walk line grove-bank ↔ cliffs-bank actually connects; adjust
  swap list to reality). Flag `BRIDGE_LOWERED`, restore on load.
- South-bank sign (`TextId::CrankSign`): "A crank across the water. A blade's
  edge, flung true, might turn it." (Beam at full hearts or a reflected
  octorok rock does it now; boomerang later — never name the tools.)
- Polish sweep: chime/plate/gate/crank FX readable at gameplay scale; bomb
  flash legible; shop navigable with pad; a full keyboard run of: New Game →
  elder → chime gates → chime finale → Courage → camp chest (still the 2B
  guard-group gate — 2C-B upgrades it) → plate court → Wisdom → shrine.
- WORKER_NOTES completion entry: landed/deviations/tuning/economy math,
  screenshots, **explicit statement that the 2C-A seams below are frozen**.

## 2C-A Definition of Done

1. Courage Gem requires the chime curriculum (2 taught gates + 3-in-window
   finale); Wisdom Gem requires the plate court; both reset cleanly on reload
   when unsolved and restore open when solved. No path lets you take a gem
   without its puzzle. Power Gem unchanged (2C-B's job).
2. Bombs: buyable, cap rules, tap-K place / hold-K shield unchanged
   (perfect block verified), blast damages enemies + player + barricades +
   cracked wall; grove bomb cave open-once with 100₹ chest; state survives
   reload + door transitions.
3. Shop: full menu with 4 rows, purchases persist, heart piece #4 →
   max-heart beat fires with 4 pieces.
4. Crank lowers the broken bridge permanently; shortcut walkable both ways.
5. Barricade tiles breakable by sword (3 hits) and bombs (instant) with
   debris/SFX; several placed in camp as cover previews (final arena
   dressing is 2C-B).
6. `cargo check` + `clippy -D warnings` (wasm32) clean; `env -u NO_COLOR
   trunk build --release` ok; no file >~600 lines; F3 Arena unaffected.
7. Frozen seams for 2C-B: `game::puzzle` API + `content::puzzles` shapes,
   `BUTTON_CYCLE`, `AttackKind::Bomb`, barricade break path, flags 90–98,
   `SaveGame`/`PlayerPersist` bomb fields.

---

# PART 2C-B — Enemy families + camp wave battle (worker 2, after 2C-A lands)

Do not start until the **"Phase 2C-A completion"** entry exists in
WORKER_NOTES. Read it first; code + that entry win over this section.

## 4. Milestones

### M1 — Four enemy families (art + AI + SFX)

Art: new `content/src/art/enemies_act1b.rs` (SpriteDef pattern from
`enemies.rs`: horizontal frame strips, `F` outline discipline, palette chars
from `palette.rs`), registered in `all_bakes()`. Bodies 16×16; sprites may be
16×24 drawn with the player's `-8.0` y-offset trick. Tuning consts appended to
`combat::tuning`. Each family: `SpawnKind` + `EntityKind` + `EntityData`
variants (exhaustive-match sweep: spawner `spawn_kind` + `is_hostile_kind`,
`enemies::update_enemies`, `integrate_non_player`, `resolve_hits`,
`render_entity`, `Entity::is_enemy`), one file per family in
`game/src/enemies/`, spawn telegraph via the existing `SPAWN_TELEGRAPH`
shimmer pattern.

- **Raider Spear** (`raider.rs`, camp): HP 4, contact 1. Approach to ~26 px →
  25-tick poke telegraph (spear glint + `SfxId::RaiderPoke` windup) → thrust
  (spawn a 1-tick `ENEMY_HIT`-layer hitbox 20 px forward or push
  `DamagedPlayer` on overlap, damage 2 = one heart) → 30-tick **guard**
  stance: front sword hits refused (clank spark + `SfxId::SkeletonClank`
  reuse or own `GuardClank`) — flank, dash-through, or wait. Walks around
  bonfires when idle (small patrol drift).
- **Raider Torch** (`raider.rs`): HP 3, keeps 60–110 px. 30-tick overhead
  telegraph → lobs `EntityKind::TorchProj` in an arc (linear travel + fake-z
  sine like the ledge hop; not blockable-reflectable — it's an arc, mark it
  hostile non-reflect in the projectile matches) → on landing: 2×2-ish ground
  flame zone for 90 ticks (damaging `DamagedPlayer 1` on overlap every 30
  ticks, flame FX) — area denial that pushes the player around the camp.
- **Wisp** (`wisp.rs`, ruins): HP 2, contact 1 ("burn": brief orange flash on
  the player). Cycle: 90 ticks visible (drifts toward player at 0.5) →
  40-tick fade telegraph → 60 ticks phased (25% alpha, `health.iframes`
  held high so it is untargetable, no contact damage) → fade in near the
  player (≤ 60 px, never inside a 12-px ring). `SfxId::WispPhase` on both
  transitions. Only damageable while visible — teaches timing.
- **Skeleton** (`skeleton.rs`, ruins): HP 4, contact 1. Shield up while
  walking (front sword hits refused with clank — reuse the octorok-hide
  refusal pattern in `apply_attack_hit`, keyed on facing vs hit direction);
  probe attack: 20-tick telegraph → short lunge poke (damage 2). Vulnerable:
  (a) from behind, (b) 40-tick stagger after its poke recovery, (c)
  **perfect-blocking its poke staggers it 60 ticks** (hook: in
  `apply_player_damage`'s perfect-block branch, if the damage source entity
  is a skeleton in poke state → set its stagger; you'll need the attacker id
  plumbed — extend `DamagedPlayer` with `source: Option<EntityId>` (additive,
  default None at existing call sites)). Boomerang stun comes free in Phase 3
  via the same stagger entry point — leave a `pub fn stagger(world, id)`.
- SFX appends: `RaiderPoke`, `GuardClank`, `TorchThrow`, `FlameBurst`,
  `WispPhase`, `SkeletonRattle` (+ reuse `EnemyHurt`/`Kill`).
- F2 viewer: verify each sheet in motion (footing pinned, silhouettes
  distinct at 100% zoom).

### M2 — Camp 3-wave war-chest battle (Power Gem upgrade)

Spawner-group wave chain (NOT the arena WaveDirector):

- Flags/groups: keep `GRP_CAMP_GUARD = 41` as **wave 1** (re-author its 6
  slots as 3 raider spears + 2 bats + 1 torch). Add `GRP_CAMP_W2 = 42`
  (2 spears + 2 torches) and `GRP_CAMP_W3 = 43` (3 spears + 1 torch + 1
  skeleton "veteran") — spawn positions ring the war-chest clearing
  (198..208 × 55..65).
- Spawner extension (frozen-seam-respecting, additive): a `locked_groups`
  set on `Spawner`, initialized from a const chain table in `content::flags`
  (`pub const CAMP_WAVE_CHAIN: [(u16, u16); 2] = [(41, 42), (42, 43)];`).
  Locked groups never activate by distance. On `WorldEvent::GroupCleared(g)`
  (handled in `lib.rs drain_events` where `GRP_CAMP_GUARD` is already
  matched): unlock the chained group, force-activate its slots (spawn with
  the 45-tick telegraph), `SfxId::WaveCue` + toast "WAVE 2!" / "WAVE 3!".
  `GroupCleared(43)` → set `GROUP_CAMP_GUARD` flag + "GUARDS CLEARED" toast +
  checkpoint save (the chest unlock flag stays the same, so the existing
  power-chest check in `interact.rs` needs only its `group_cleared(..., 41)`
  fallback updated to the flag-or-43 rule).
- If `GROUP_CAMP_GUARD` is already set at map load: 41 behaves as a normal
  respawning camp group; 42/43 stay permanently locked (never respawn — the
  war is won).
- Arena dressing: barricade tiles (2C-A) seal the two gaps into the clearing
  — the player breaks in (sword or bomb), which triggers wave 1's activation
  naturally by distance; barricades inside the ring give cover from torch
  arcs. Death mid-chain: uncleared waves stay locked, cleared ones stay
  cleared until the normal group respawn rule rewinds 41 (acceptable: the
  chain restarts only via 41's respawn — verify no state where 42 is alive
  and 41 respawns; guard with the locked set).
- Torch flames must not burn NPCs/signs (there are none in camp — assert).

### M3 — Encounter re-dress across regions

Re-author SpawnDefs (placement files only; totals stay ~60–80):

- **Camp**: `GRP_CAMP` slimes/bats → mostly raider spears + torches around
  the three bonfires (keep 2–3 bats as harassers). The camp finally reads
  "raider camp".
- **Ruins**: keep octorok lanes; add 3–4 wisps drifting the colonnades and
  2 skeletons patrolling near the plate court (pressure while pushing
  blocks) — `GRP_RUINS`.
- **Cliffs**: unchanged roster (octorok showcase) + 1 wisp near the summit
  for spice.
- **Grove / meadow / shrine approach**: grove unchanged; shrine sentinels →
  1 skeleton + 1 raider spear per side (tougher soft gate, still runnable).
- Combinational rule from GAME_DESIGN §7: every pocket mixes ranged + melee
  + harasser; never raise HP to add difficulty.
- Safe zones hold: village, interiors, 12-tile radius of checkpoints/doors.

### M4 — Feel/difficulty pass + completion

- Full keyboard playthrough: New Game → elder → chime curriculum → Courage →
  camp break-in → 3 waves → Power → plate court (+ wisp/skeleton pressure) →
  Wisdom → shrine opens → lobby. Bombs bought and used on the grove wall
  somewhere in the middle. Reload tests at: mid-chime-window, mid-wave-2,
  blocks half-pushed, post-crank.
- Perf: F1 in the camp during wave 3 (worst case: ~8 AI + flames + FX) —
  hold ~60. Enemy count budget per pocket ≤ 12 active.
- Tune contact/poke damage, wave sizes, wisp phase timings ±30%, log.
- WORKER_NOTES completion entry incl. what Phase 3 inherits (puzzle seam,
  stagger hook, B-item cycle, tunic flag, ruins boomerang-bonus site).

## 2C-B Definition of Done

1. Four families alive with distinct, telegraphed behaviors; F2-verified art;
   distinct SFX; skeleton perfect-block stagger works.
2. Camp battle: barricade break-in → 3 escalating waves → chest unlock →
   Power Gem; chain is reload-safe and never soft-locks; won-war state
   persists.
3. Region encounter identities match §M3; safe zones hold; ≤12 active AI;
   ~60 fps in wave 3 (F1 evidence).
4. Full critical path (now with all three real gem gates) clean end-to-end;
   no console errors.
5. check/clippy/trunk clean; files <~600 lines; completion entry written.

---

## 5. File ownership

**2C-A**: `game/src/puzzle/*` (new), `game/src/items/bombs.rs` (new),
`game/src/ui/shop.rs` (new), `game/src/debug.rs` (new, extraction),
`content/src/puzzles.rs` (new), `content/src/art/props_puzzle.rs` (new),
catalog ids 260–279, flags 90–98 + `CHEST_RUINS_BONUS = 20`, `TextId`/text
appends, `SfxId` appends, `MapId::Cave` codec + `interiors::cave_bomb`,
placement edits in `grove.rs`/`ruins.rs`/`connective.rs`/`camp.rs`
(barricades only), `engine::input` BUTTON_CYCLE **only**, additive fields on
`PlayerData`/`PlayerPersist`/`SaveGame`/`GemData`, `AttackKind::Bomb` +
match arms, HUD item-slot skin.

**2C-B**: `game/src/enemies/{raider,wisp,skeleton}.rs` (new),
`content/src/art/enemies_act1b.rs` (new), spawner `locked_groups` +
chain handling, `DamagedPlayer.source` plumb, SpawnDef re-authoring in
`camp.rs`/`ruins.rs`/`cliffs.rs`/`shrine.rs`, tuning appends, SFX appends.

**Neither may restructure**: chunk/render pipeline, camera, physics beyond
specified, Phase 1 player/combat internals beyond specified hooks, minimap,
`WaveDirector`, docs (append-only WORKER_NOTES excepted).

## 6. Verification protocol

- Per milestone: wasm32 check + clippy, `env -u NO_COLOR trunk build
  --release`, then a targeted Playwright smoke vs `python3 -m http.server
  8090 --directory dist` (scripted key sequences; screenshots; localStorage
  assertions for new flags/fields). Kill the server + headless browser after
  every run (2A/2B precedent).
- Puzzle-specific asserts: chime window fail-then-succeed; plate court gate
  re-closes on weight removal pre-flag; block pushed into a corner →
  reload resets; bomb wall opens exactly once; crank persists.
- Save-shape check: version 2 JSON with `bombs`/`bomb_cap`/`selected_item`
  defaults parses old 2B saves without wiping them.
- Human feel notes at completion: chime window generosity, block push
  latency, bomb tap-vs-shield comfort, wave-2/3 spike, wisp annoyance level.
