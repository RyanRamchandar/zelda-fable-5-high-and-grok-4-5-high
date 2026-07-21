# WORKER_BRIEF_PHASE1A.md — Player, Combat, Juice, Energy, Style

You are a Grok worker. This brief is self-contained; follow it exactly. Conflict
order: `DECISIONS.md` → `ARCHITECTURE.md` → this brief → `GAME_DESIGN.md`.
You run autonomously — do not ask the human anything except for credentials.

## 0. Mission

Make a gray-box arena that **feels GREAT to play**. Gameplay feel is priority #1
for the whole project, and this phase is where it is won. Ship the complete player
kit (movement, full sword kit, shield/perfect block, dash), the damage pipeline
with every juice item, the energy meter, the style/momentum system, a functional
plain-rect HUD, combat SFX, and — critically — the **shared `world`/event seams**
that Phase 1B (enemies + HUD art + atlas) builds on. 1B starts only after you
finish, so your seams are the contract.

**You do NOT build:** enemy families (slime/bat/octorok), the sprite atlas, any
pixel art, HUD sprite skinning, or the debug sprite viewer. Those are Phase 1B.
You build one **target dummy** entity so the combat pipeline is verifiable.

## 1. Read first

1. `docs/DECISIONS.md` — locked stack, allowed crates, git rules.
2. `docs/ARCHITECTURE.md` — module map, ownership table (§11), event rule (§5).
3. `docs/GAME_DESIGN.md` §1–3 — the exact player kit, juice checklist, style system.
4. `docs/WORKER_NOTES.md` — Phase 0 worker's environment notes (trunk `NO_COLOR`
   issue, toolchain, serve-vs-build dist warning).
5. The actual code listed in §2 below. **Trust the code over any doc drawing.**

## 2. Current scaffold reality (verified 2026-07-21, commit `f65e9f5`)

The ARCHITECTURE §1 tree is the *target* layout. What exists **today**:

- `crates/engine/src/lib.rs` — `Platform { window, canvas, draw, input, audio }`,
  `Platform::create()`, resize + audio-unlock listeners already wired.
- `crates/engine/src/canvas.rs` — `WIDTH: f64 = 480.0`, `HEIGHT: f64 = 270.0`,
  integer scale, `client_to_logical`.
- `crates/engine/src/render.rs` — `Draw` with only `clear`, `rect`, `circle`,
  `text` (all screen-space, color as `&str` CSS color). **No camera transform,
  no sprites, no atlas.** There is **no `atlas.rs`** in engine yet (1B adds it).
- `crates/engine/src/input/mod.rs` — `InputState { move_vec: (f32,f32),
  buttons: [Button; 6], touch_active, touch_overlay }`;
  `Button { held, pressed }` (no `released` edge yet); indices
  `BUTTON_ATTACK=0, BUTTON_ITEM=1, BUTTON_DASH=2, BUTTON_INTERACT=3,
  BUTTON_PAUSE=4, BUTTON_CONFIRM=5`. Keyboard: WASD/arrows, J/Space=Attack,
  K=Item, L/Shift=Dash, E=Interact, Esc=Pause, Enter=Confirm. Gamepad polled via
  `input.poll_gamepad(&window)`. Touch overlay exposes attack+dash buttons only.
- `crates/engine/src/audio.rs` — `Audio` with only `beep(freq_hz, dur_s)`
  (square osc + gain envelope). No SFX descriptor system yet.
- `crates/engine/src/time.rs` — `run_loop(&window, FnMut(steps: u32))`, fixed
  60 Hz accumulator, `MAX_FRAME_DT = 0.1`.
- `crates/engine/src/save.rs` — `save(key, json)`, `load(key) -> Option<String>`.
- `crates/game/src/lib.rs` — flat Phase 0 stub: `Game { x, y, ticks, ... }`,
  `pub enum GameEvent { Beep, Save(String) }`,
  `update(&mut self, input: &InputState) -> Vec<GameEvent>`,
  `render(&self, d: &mut Draw)` which also draws the touch overlay.
- `crates/game/src/save_data.rs` — `SaveGame { x, y }`, `SAVE_KEY = "shard_save_v1"`.
- `crates/content/src/{art,audio,maps}/mod.rs` — empty placeholder modules.
- `crates/app/src/lib.rs` — builds Platform + Game, runs loop, forwards
  `GameEvent::Beep → audio.beep`, `GameEvent::Save → engine::save::save`.
  **You will rewrite this event match** when you change `GameEvent`.

Dependency law (never reversed): `app → game → { engine, content }`;
`content → (nothing)`; `engine → web-sys/js-sys only`. `engine` has zero game
rules; `game` has zero `web-sys` imports. Because `content` depends on nothing
and `engine` cannot depend on `content`, any content-defined data consumed by
engine goes through a tiny **adapter in `app`** (see SFX seam, §5.8).

## 3. Locked constraints

- Crates allowed: `wasm-bindgen`, `web-sys`, `js-sys`, `console_error_panic_hook`,
  `serde`, `serde_json`, `fastrand`. Add `fastrand` to `crates/game/Cargo.toml`
  for particles/drops — that is pre-approved. Nothing else.
- No file over ~600 lines; split along ARCHITECTURE module seams before that.
- Fixed 60 Hz update; all gameplay numbers are in **ticks** (60 ticks = 1 s).
- Commit small and often on `main`, messages prefixed `phase1a:`. Do not push.
  Do not deploy.
- Every commit: `cargo check --workspace --target wasm32-unknown-unknown` and
  `cargo clippy --workspace --target wasm32-unknown-unknown -- -D warnings` clean.
- Environment gotchas (from Phase 0 notes): use `env -u NO_COLOR trunk ...`;
  never treat `trunk serve` output as deployable `dist/`; toolchain is rustup
  stable + `wasm32-unknown-unknown`.
- Tuning numbers from GAME_DESIGN are starting values; you may adjust ±30% for
  feel. Log every change in `docs/WORKER_NOTES.md` (append-only).
- If blocked, write the blocker to `docs/WORKER_NOTES.md` and continue with the
  nearest non-blocking interpretation.

## 4. Milestones (commit at least once per milestone)

### M1 — Shared seams: math, world, entities, physics, camera, events

This is the contract 1B depends on. Land it first and keep it stable.

New files:

- `crates/game/src/math.rs` — `Vec2 { x: f32, y: f32 }` (add/sub/scale/len/
  normalize_or_zero), `Dir4 { Up, Down, Left, Right }` with `Dir4::from_vec`
  (dominant axis, keep previous facing when vec is zero) and `unit() -> Vec2`.
- `crates/game/src/world/mod.rs` — 
  ```rust
  pub struct World {
      pub arena: Vec<Slot>,            // Slot { gen: u32, entity: Option<Entity> }
      pub free: Vec<u32>,
      pub map: content::maps::MapDef,  // see M2
      pub camera: Camera,
      pub hitstop: u8,                 // global freeze ticks remaining
      pub events: Vec<WorldEvent>,     // internal, drained each tick
      pub tick: u64,
      pub rng: fastrand::Rng,
  }
  ```
  `spawn(&mut self, Entity) -> EntityId`, `despawn(&mut self, EntityId)`,
  `get / get_mut(EntityId) -> Option<&Entity>`, `iter_alive()`, plus an
  id-stable iteration helper for pairwise queries.
- `crates/game/src/world/entity.rs` — `EntityId { index: u32, gen: u32 }`;
  ```rust
  pub struct Entity {
      pub kind: EntityKind,
      pub pos: Vec2, pub vel: Vec2, pub facing: Dir4,
      pub body: Option<Body>,       // Body { half: Vec2, solid: bool, layer: Layer, mask: u8 }
      pub health: Option<Health>,   // Health { hp: i32, max: i32, iframes: u16, flash: u8 }
      pub knockback: Vec2,          // decaying impulse, applied by physics
      pub anim: AnimState,          // AnimState { sheet: u16, frame: u16, timer: u16 } (1B consumes)
      pub data: EntityData,         // per-kind state machine enum
  }
  pub enum EntityKind { Player, Dummy, Pickup, SwordBeam, FairyFountain,
      /* 1B adds: Slime, Bat, Octorok, OctorokRock */ }
  pub enum EntityData { None, Player(PlayerData), Dummy(DummyData),
      Pickup(PickupData), Beam(BeamData) /* 1B extends */ }
  ```
  Collision `Layer`: `PlayerBody, EnemyBody, PlayerHit, EnemyHit, Pickup` (u8 bitmask).
- `crates/game/src/world/physics.rs` — AABB vs tile collision grid:
  `move_entity(world_map, entity)` integrates `vel + knockback` with axis-separated
  sweep against solid tiles (16 px tiles); knockback decays over 8 ticks
  (multiply ~0.7/tick, zero under 0.05). Overlap queries:
  `overlaps(a, b) -> bool`, `query_aabb(world, center, half, mask) -> Vec<EntityId>`.
- `crates/game/src/world/camera.rs` — `Camera { pos: Vec2, shake: Vec2,
  shake_ticks: u8, shake_mag: f32 }`. Smooth follow (lerp ~0.15/tick toward
  player + 12 px lookahead in facing dir), clamp to map bounds,
  `add_shake(mag, ticks)` (mag capped 3.0 px), per-tick decay, and
  `offset() -> Vec2` = pos − (240,135) + shake jitter (use `world.rng`).
- Events, in `world/mod.rs`:
  ```rust
  pub enum WorldEvent {
      AttackHit { target: EntityId, attack: AttackKind, dir: Vec2, pos: Vec2 },
      DamagedPlayer { amount: i32 },
      Killed { kind: EntityKind, pos: Vec2 },
      FxRequest(FxKind),               // particles/numbers/toasts, defined in fx
      Sfx(SfxId),                      // forwarded out to app
      StyleAction(StyleVerb),          // feeds style meter
      EnergyDenied,                    // meter flash + refuse sfx
  }
  ```
  Rule (ARCHITECTURE §5): peer modules never call each other; they push
  `WorldEvent`s or read `World` state. `game::lib` drains `world.events` once per
  tick after all systems run, routes them (combat → fx → outbound), and converts
  `Sfx`/`Save` into the outbound `Vec<GameEvent>` returned to `app`.
- Rewrite `crates/game/src/lib.rs`: `Game { world: World, ui: ui state, ... }`,
  keep the public surface `Game::from_storage_json(Option<String>)`,
  `update(&mut self, &InputState) -> Vec<GameEvent>`, `render(&self, &mut Draw)`.
  New `pub enum GameEvent { Sfx(SfxId), Save(String) }` (Beep is gone — update
  `crates/app/src/lib.rs` match: `Sfx(id)` → adapter in §5.8, `Save` unchanged).
  Fixed system order per tick:
  1. tick timers (iframes, flash, anim);
  2. if `world.hitstop > 0`: decrement, run `fx::update` only, skip 3–7;
  3. `player::update`; 4. `enemies::update` (stub); 5. `physics` integration;
  6. `combat::resolve_hits`; 7. `items::pickups::update`; 8. `fx::update`;
  9. `camera::update`; 10. drain events → route → return outbound.
- `crates/game/src/enemies/mod.rs` — **stub only**: `pub fn update(world: &mut World, input: &InputState) {}`
  called in slot 4. 1B fills it. Do not add enemy logic.
- Save: keep `SaveGame { x, y }` roundtrip working (player pos), 60-tick interval
  as today. Do not expand the save schema this phase.

Render seam (needed for camera): extend `engine/src/render.rs` `Draw` with
world-space variants that take a camera offset, or (preferred, simplest) add
`Draw::set_offset(dx: f32, dy: f32)` applied inside `rect/circle/text`, plus
`Draw::line(x1,y1,x2,y2,w,color)` for slash arcs and debug. Game calls
`set_offset(-cam.x, -cam.y)` for world drawing and `set_offset(0,0)` for UI/HUD.
Keep `Draw` free of game rules. Render order: floor tiles (flat rects from map),
entities **y-sorted by `pos.y`**, fx, then screen-space HUD + touch overlay
(keep the existing touch overlay drawing).

### M2 — Arena map (gray-box)

- `crates/content/src/maps/mod.rs` — plain-data `MapDef`:
  ```rust
  pub struct MapDef {
      pub width: u32, pub height: u32,          // in 16px tiles
      pub ground: Vec<u16>,                     // tile ids, 0 = plain floor (1B skins)
      pub collision: Vec<bool>,                 // true = solid
  }
  ```
  `content` imports nothing — keep it pure data + builder fns (`fill`, `rect_border`).
- `crates/content/src/maps/arena.rs` — `pub fn arena() -> MapDef`: 60×34 tiles
  (960×544 px, ~2×2 screens so the camera has room), solid border walls, a few
  interior pillar blocks (2×2 solids) for circling around, an open combat field
  center, and a fountain corner. Export tile-id constants (FLOOR=0, WALL=1,
  FOUNTAIN=2) so 1B can map ids → sprites without changing the map.
- Render: floor = flat dark color, walls = lighter rects, fountain = colored
  circle. Legible, not pretty — 1B replaces visuals only.

### M3 — Movement feel (GAME_DESIGN §1 numbers)

`crates/game/src/player/mod.rs` (+ `player/movement.rs` if it grows):

- 8-way movement, top speed 1.5 px/tick (90 px/s); 10-tick accel ramp to top
  speed and 10-tick decel to zero (per-axis or vector lerp — pick what feels
  crisper, log choice). 4-dir facing from dominant input axis; velocity stays
  8-way. Facing does not change during a swing.
- Dust particles (via `FxRequest`) on direction snap (facing change while moving)
  and on dash start.
- Dash: 4.5 px/tick for 10 ticks + 6-tick recovery (no steering during dash;
  recovery allows buffered slash — dash is cancellable into slash from tick 6).
  I-frames ticks 2–8. Costs 25 energy; refused (flash + `EnergyDenied`) if
  insufficient. Dash through the dummy must not deal damage (dash is movement,
  not an attack — but it awards `StyleVerb::DashThrough` if you pass through an
  enemy body's AABB during i-frames).
- Player collision body 16×16 (bottom-aligned; sprite later is 16×24). Spawn at
  arena center; camera snaps to player on load.

### M4 — Sword kit (GAME_DESIGN §1)

`crates/game/src/player/sword.rs` — a single explicit state machine
(`Idle, Swing{stage, tick}, Charging{tick}, Spin{tick}, DashState..., Recovery`)
in `PlayerData`. Numbers (starting values):

- **Quick slash**: 12-tick swing, 90° arc in facing dir. Hitbox: AABB 20×20
  centered 14 px ahead of player center in facing dir, active ticks 3–9.
  Damage 1. A 6-tick **input buffer**: attack pressed during a swing's last
  6 ticks queues the next stage.
- **3-hit combo**: slash → backslash → **lunging finisher** (1.5× damage,
  +60% knockback, small 8 px forward hop over 4 ticks). Combo drops if the next
  buffered press doesn't arrive within 20 ticks of a swing ending. Finisher
  triggers the "COMBO FINISH!" toast + `StyleVerb::Finisher`.
- **Hold-to-charge**: hold attack ≥30 ticks → charged (wind-up shimmer fx while
  holding, distinct ready cue at 30). Release → **full spin**: 360° hitbox
  (28 px radius circle approximated as 4 overlapping AABBs or radial distance
  check), 2× damage, big knockback, drains 20 energy on release (if energy
  < 20 at release, do a normal slash + `EnergyDenied`). Releasing before 30
  ticks = normal slash.
- **Sword beam**: on quick slash while at full hearts, also spawn a
  `SwordBeam` projectile: 3 px/tick in facing dir, 0.75× damage, dies at
  8 tiles (128 px) or first hit. Small spawn cue + distinct sfx.
- **Shield**: hold Item button (K / gamepad 2 — Phase 1 has no B-items yet, so
  Item = shield this phase; note this remap in WORKER_NOTES for the planner):
  front-90° block while held, movement slowed to 0.6 px/tick. Blocked hit:
  costs 5 energy, zero damage, small self-pushback, spark fx.
  **Perfect block**: hit lands within first 6 ticks of shield raise → refund
  +10 energy, "PERFECT BLOCK" toast, `StyleVerb::PerfectBlock`, distinct sfx,
  reflects projectiles (flip beam/rock velocity; matters for 1B octorok rocks —
  make reflection generic on projectile kinds). The dummy can't attack, so
  verify blocks with a debug projectile (H key, §4-M8).

All numbers live in `crates/game/src/combat/tuning.rs` as `pub const`s — one
place, grouped, commented. Player module reads them; nothing else defines numbers.

### M5 — Damage pipeline + juice (GAME_DESIGN §2 — non-negotiable checklist)

`crates/game/src/combat/mod.rs` + `damage.rs`:

- Attacks register hitboxes → `combat::resolve_hits` finds overlapping hurtboxes
  (layer/mask), dedupes per-swing (one hit per target per swing id), then:
  damage, hp floor 0 → `Killed`; set target flash-white 4 ticks; knockback
  impulse scaled by attack (quick 2.0, finisher 3.2, spin 4.0 px/tick initial,
  8-tick decay); hitstop: 3 ticks (normal hit), 5 (finisher/charge spin),
  12 reserved for boss phase break (const now); `FxRequest` impact sparks +
  damage number; `Sfx(HitEnemy)`; `StyleAction(verb)`.
- Damage numbers: `crates/game/src/fx/numbers.rs` — small, spawn at hit pos,
  pop up 6 px then fade over ~40 ticks; finisher/spin numbers gold, others white.
- Particles: `crates/game/src/fx/particles.rs` — pooled (cap ~256), kinds:
  dust, impact spark, kill poof, charge shimmer, fountain sparkle. Simple
  pos/vel/life/color rects, 1–3 px.
- Slash arc fx: `crates/game/src/fx/mod.rs` draws a fading 90° arc (3–4 line
  segments or circle-slice) in facing dir for the swing's active ticks; full
  circle for spin.
- Toasts (temporary home): `crates/game/src/fx/toasts.rs` — queue of
  `{text, ticks}` drawn top-center screen-space with `Draw::text`, pop-in +
  fade, max 2 stacked. **1B will move/restyle toast rendering into `game::ui`**
  — keep the queue API (`push_toast`) in one place and document it.
- Screenshake: `camera.add_shake(...)` — ≤3 px, only on finisher, spin hit,
  player damaged, kill.
- Kill flow: `Killed` → poof fx + drop spawn (see M6 pickups) + sfx.
- **Target dummy**: `EntityKind::Dummy`, 20 hp, stationary, takes damage/
  knockback/flash like an enemy, respawns at its post 120 ticks after death.
  Place 3 in the arena. 1B keeps them (useful forever) but may gate behind debug.

### M6 — Energy, style, pickups, fountain

- `crates/game/src/combat/energy.rs` — 100-unit meter. Spends: dash 25, spin 20,
  block 5. Refunds: perfect block +10. Regen 12/s (0.2/tick) after 30 ticks
  without a spend. `EnergyDenied` → meter flash 10 ticks + dull refuse sfx.
- `crates/game/src/combat/style.rs` — rank D→C→B→A→S. Points per
  `StyleVerb` (Slash, Finisher, ChargeSpin, DashThrough, PerfectBlock — leave
  a `BoomerangStun` variant for later): each verb has a per-verb repeat-decay
  (recent repeats award less; refresh window ~180 ticks). No-damage streak
  trickle (+1/120 ticks while in combat). Decay slowly out of combat
  (~-1/90 ticks after 300 ticks with no verb). Taking damage drops exactly one
  rank. Thresholds as consts (e.g. 0/20/50/90/140 — tune). Effects (economy
  only): B+ → energy regen ×1.5; A+ → dash cost 20; S → kills also drop +1
  rupee + small energy orb. Rank change → chip pulse + short cue + toast.
- `crates/game/src/items/mod.rs` + `items/pickups.rs` — pickup entities:
  rupee (green diamond rect, value 1), heart (red, +1 half-heart), energy orb
  (cyan, +15). Magnetism: within 24 px of player, accelerate toward them;
  collect on overlap; distinct pickup sfx each; despawn after 600 ticks with
  blink in last 120. Drop table on `Killed` (dummy: 70% nothing, else 1 rupee /
  heart at low player hp / energy orb — simple weighted roll with `world.rng`).
- Fairy fountain: `EntityKind::FairyFountain` zone at the fountain corner —
  standing inside refills energy (+2/tick) and hearts (+1 half-heart/30 ticks),
  sparkle particles, soft chime while healing. Satisfies acceptance "fountain
  object restores both".
- Player health: 3 hearts = 6 half-heart units. Player damage path exists
  (`DamagedPlayer`): iframes 60 ticks + flicker, knockback, rank drop, shake,
  hurt sfx. Testable via debug key (M8) until 1B enemies land.

### M7 — HUD (functional, plain rects) + SFX set

- `crates/game/src/ui/mod.rs` + `ui/hud.rs` — screen-space, drawn last:
  hearts top-left (rects: full/half/empty); energy = left-edge vertical bar
  (frame + fill + low/denied flash); style chip next to it (rank letter via
  `Draw::text` + pulse on change); B-item slot box bottom-right showing "—"
  (empty until Phase 2). **This HUD is deliberately ugly**; 1B restyles it with
  atlas sprites — keep layout constants in one block, and keep all HUD state
  reads read-only from `World`/combat state.
- SFX seam (three pieces, respects dependency law):
  1. `engine/src/audio.rs`: add
     ```rust
     pub struct SfxParams { pub osc: OscKind /* Square, Triangle, Saw, Sine, Noise */,
         pub freq_start: f32, pub freq_end: f32, pub attack_s: f32, pub decay_s: f32,
         pub gain: f32, pub noise_mix: f32 }
     pub fn play(&mut self, p: &SfxParams)
     ```
     Noise via `AudioBufferSourceNode` with a pre-generated random buffer (add
     `AudioBuffer`, `AudioBufferSourceNode` web-sys features). Keep `beep` or
     fold it in.
  2. `crates/content/src/audio/sfx.rs`: `pub enum SfxId { Slash1, Slash2,
     Finisher, ChargeReady, SpinRelease, Beam, Dash, Refused, ShieldBlock,
     PerfectBlock, HitEnemy, HurtPlayer, Kill, PickupRupee, PickupHeart,
     PickupEnergy, RankUp, RankDown, FountainChime /* 1B adds enemy cues */ }`
     plus `pub struct SfxSpec { /* same fields as SfxParams, plain data */ }`
     and `pub fn spec(id: SfxId) -> SfxSpec`. `content` stays dependency-free.
  3. `crates/app/src/lib.rs`: on `GameEvent::Sfx(id)`, adapt
     `content::audio::sfx::spec(id)` field-by-field into `engine::audio::SfxParams`
     and call `audio.play`. (This mirror is planner-approved; keep it tiny.)
  `SfxId` is re-exported through `game` so `world.events` can carry it.
  Every verb above gets a **distinct** cue — same synth family per category
  (slashes = quick saw sweeps, pickups = sine blips, denials = low dull square).
- Rate-limit: max ~8 sfx per tick, drop extras.

### M8 — Debug hooks + verification pass

- Extend `engine/src/input`: add `pub debug: [Button; 3]` to `InputState`
  (keyboard-only): `DEBUG_OVERLAY=0` (F1), `DEBUG_VIEWER=1` (F2, wired but unused
  — 1B's sprite viewer hook), `DEBUG_ACTION=2` (H). Wire through
  `SharedInput`/keyboard.rs/snapshot/end_frame exactly like normal buttons.
- F1 overlay: fps estimate, entity count, player pos/state, energy, style points,
  hitstop, particle count (screen-space `Draw::text`).
- H key: fires a slow debug projectile at the player from 60 px ahead of facing
  (tests shield, perfect block, reflect, player damage, iframes, rank drop) —
  only when F1 overlay is on.
- Then run the full verification protocol (§7) and the feel checklist (§8),
  fix, tune, commit.

## 5. File ownership (you create/own) and the 1B contract

You own this phase: `crates/game/src/{lib.rs, math.rs, world/*, player/*,
combat/*, fx/*, items/*, enemies/mod.rs (stub), ui/* (functional draft)}`,
`crates/content/src/maps/*`, `crates/content/src/audio/sfx.rs`,
`crates/engine/src/render.rs` (offset/line additions),
`crates/engine/src/audio.rs` (SfxParams/play), `crates/engine/src/input/*`
(debug buttons only), `crates/app/src/lib.rs` (event routing/adapter).

**Contract handed to 1B (do not break after landing; changes require a
WORKER_NOTES entry):**
1. `World` API: spawn/despawn/get/get_mut/iter, `events` push, `hitstop`, `rng`.
2. `Entity`/`EntityKind`/`EntityData` — 1B adds variants (Slime, Bat, Octorok,
   OctorokRock) and match arms; design your matches to fail loudly on new
   variants (exhaustive matches, no `_` catch-alls in combat/fx/physics).
3. `WorldEvent` set incl. `AttackHit`, `Killed`, `FxRequest`, `Sfx`, `StyleAction`.
4. `combat::resolve_hits` semantics: enemies deal contact/projectile damage to
   the player by pushing hit events / using hit layers — document exactly how
   in the module docs (1B follows it).
5. `physics::move_entity` + query helpers.
6. `AnimState` fields (1B's atlas rendering reads these; you set frame/timer
   generically — e.g. walk cycles frames 0..3 at 8 ticks/frame — even though
   you render rects).
7. `content::maps::MapDef` + arena tile-id constants.
8. `SfxId`/`SfxSpec`/`spec()` table + app adapter (1B appends enemy ids).
9. `ui::hud` layout constants + `push_toast` API.
10. Render order incl. y-sort and `Draw::set_offset` convention.
11. Debug button indices (F2 reserved for 1B's sprite viewer).

At the end, append a "Phase 1A completion" section to `docs/WORKER_NOTES.md`:
what landed, exact seam signatures if they differ from this brief, tuning
deltas, and anything 1B must know.

## 6. Definition of Done

1. All M1–M8 items implemented per spec (or with logged ±30% tuning deltas).
2. Gray-box arena playable: move, dash (i-frames + cost), full 3-hit combo with
   buffering, charge spin, sword beam at full hearts, shield + perfect block
   (verified via debug projectile), dummy targets take damage/knockback/flash/
   die/respawn, pickups drop + magnetize + collect, fountain restores.
3. Juice checklist (GAME_DESIGN §2) fully present: hitstop 3/5, knockback with
   8-tick decay, slash arcs, impact sparks, 4-tick white flash, damage numbers
   (gold crits), ≤3 px screenshake, kill poof + drops with magnetism, toasts
   ("COMBO FINISH!", "PERFECT BLOCK", rank shifts). No HP bars on dummies.
4. Energy + style live and readable on the HUD; refusal feedback (flash + dull
   sfx) works; style effects (regen ×1.5 at B+, dash 20 at A+, S drops) work.
5. Every combat verb has a distinct SFX; audio unlocks on first gesture.
6. `cargo check` + `clippy -D warnings` clean on wasm target;
   `env -u NO_COLOR trunk build --release` emits a working `dist/`.
7. 60 fps with F1 overlay showing ~60; no per-tick allocations in hot loops
   beyond event vecs (reuse buffers where easy).
8. No file >600 lines. Save roundtrip still works. Enemies stub + F2 hook in
   place for 1B. WORKER_NOTES completion entry written. All commits on `main`
   prefixed `phase1a:`, not pushed.

## 7. Verification protocol (do all of it)

1. `cargo check --workspace --target wasm32-unknown-unknown`
2. `cargo clippy --workspace --target wasm32-unknown-unknown -- -D warnings`
3. `env -u NO_COLOR trunk build --release`; serve `dist/` via
   `python3 -m http.server 8090 --directory dist`.
4. Browser smoke (Playwright like Phase 0, or manual + screenshots):
   - move all 8 directions; watch accel/decel ramps; dust on turns
   - J×3 → combo finisher toast + gold number + hop; drop combo via 20-tick gap
   - hold J 30+ ticks → shimmer → release → spin hits all dummies around you
   - dash: costs energy, i-frames through debug projectile, slash-cancel
   - empty energy → dash/spin refused with flash + dull sfx; regen after 0.5 s
   - F1+H projectile: shield block (−5 energy), perfect block (+10, toast,
     reflect), unshielded hit (hearts drop, iframes flicker, rank drops one)
   - kill dummies → poof, drops, magnetism, pickups tick meters up
   - fountain refills hearts + energy with sparkles + chime
   - style climbs D→S with varied verbs; repeat-spam of one verb climbs slower
   - reload page → position persists (save roundtrip)
5. Touch: joystick + attack/dash buttons still work (Phase 0 overlay untouched).
6. Keep a fps eye during a spin hitting 3 dummies + ~100 particles.

## 8. Feel checklist (subjective bar — iterate until true)

- Stopping and turning feel instant but not twitchy (ramps ~10 ticks).
- Buffered combo means mashing J never eats an input.
- Hitstop makes every hit feel weighty but never laggy (3 ticks is short!).
- Knockback separates crowds; finisher visibly launches.
- Charge release feels earned: shimmer → ready cue → screen-clearing spin.
- Refused actions never feel like bugs (flash + sfx read as "no energy").
- The dummy fight loop (approach → combo → dash out → re-engage) is fun for
  60+ seconds even though nothing fights back. That is the bar.

If any item is false, tune (±30%, log it) before calling the phase done.
