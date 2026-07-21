# ARCHITECTURE.md — Rust/WASM Architecture

Authoritative technical structure. Workers implement inside these boundaries and do
not move code across them. Constraint: **no file may exceed ~600 lines**; split along
the module seams below before that happens.

## 1. Workspace layout

```
zelda_fable_5_high_and_grok_4_5_high/
├── Cargo.toml              # [workspace] members = crates/*
├── Trunk.toml              # trunk config (dist dir, release opts)
├── index.html              # trunk entry: canvas + minimal CSS + wasm init
├── netlify.toml            # publish = "dist", SPA-safe headers
├── docs/                   # planner-owned design docs (this folder)
├── crates/
│   ├── engine/             # platform layer: web-sys bindings, no game rules
│   │   └── src/
│   │       ├── lib.rs          # engine facade + Platform struct
│   │       ├── canvas.rs       # canvas setup, integer scaling, letterbox, resize
│   │       ├── render.rs       # draw API: sprites, tiles, rects, text, camera xform
│   │       ├── atlas.rs        # bake indexed-color grids -> offscreen atlas canvas
│   │       ├── input/
│   │       │   ├── mod.rs      # InputState (unified), edge detection
│   │       │   ├── keyboard.rs # keydown/keyup listeners
│   │       │   ├── gamepad.rs  # getGamepads() polling, standard mapping
│   │       │   └── touch.rs    # virtual joystick + buttons, multi-touch
│   │       ├── audio.rs        # AudioContext, SFX synth voices, music sequencer
│   │       ├── save.rs         # localStorage get/set JSON (serde)
│   │       └── time.rs         # fixed-step accumulator, rAF glue
│   ├── game/               # all game rules; depends on engine + content
│   │   └── src/
│   │       ├── lib.rs          # Game struct: update(dt, input) / render(draw)
│   │       ├── state.rs        # top-level GameMode enum (Title, Play, Pause, ...)
│   │       ├── world/
│   │       │   ├── mod.rs      # World: entity arena + tile map + camera
│   │       │   ├── entity.rs   # Entity struct, EntityId (generational), spawn/despawn
│   │       │   ├── camera.rs   # smooth-follow camera, soft zones, screenshake
│   │       │   └── physics.rs  # AABB vs tile grid, entity overlap queries
│   │       ├── player/         # movement, sword, combo, charge, dash, shield, beam
│   │       ├── combat/         # damage, hitstop, knockback, style meter, energy meter
│   │       ├── enemies/        # one file per family + shared ai.rs
│   │       ├── items/          # boomerang, bombs, B-item cycling, pickups
│   │       ├── puzzle/         # plates, crystals, blocks, doors, reusable state machines
│   │       ├── boss/           # granite_warden.rs (+ later bosses)
│   │       ├── ui/             # HUD, minimap, dialogs, title, pause, credits, touch skin
│   │       ├── fx/             # particles, damage numbers, slash arcs, transitions
│   │       └── save_data.rs    # SaveGame serde structs, checkpoint logic
│   ├── content/            # pure data: maps, art grids, audio patterns, dialogue
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── art/            # palette.rs, tiles.rs, player.rs, enemies.rs, ...
│   │       ├── audio/          # sfx.rs, music.rs
│   │       ├── maps/           # act1/ (one module per named region), interiors/
│   │       └── text.rs         # dialogue, signs, item descriptions
│   └── app/                # thin wasm-bindgen entry crate
│       └── src/lib.rs      # #[wasm_bindgen(start)]: build Platform + Game, run loop
└── dist/                   # trunk output (gitignored)
```

Dependency direction (enforced, never reversed):
`app → game → { engine, content }`; `content → (nothing)`; `engine → web-sys only`.
`engine` contains **zero game rules**; `game` contains **zero web-sys imports**.

## 2. Frame pipeline

1. `app` registers a rAF callback via `engine::time`.
2. Each rAF: poll gamepad, snapshot `InputState`, run 0..n fixed 60 Hz
   `game.update(&input)` steps from the accumulator, then one `game.render(&mut Draw)`.
3. `Draw` is an engine-owned command recorder over Canvas2D with camera transform;
   game code never touches `web-sys`.

## 3. Rendering

- Internal resolution **480×270**, `image-rendering: pixelated`, integer scale factor
  chosen from window size, letterboxed with black bars. Touch coordinates are mapped
  back through the same transform.
- Tile layers (ground, detail, overhang) are pre-rendered into **offscreen chunk
  canvases** (16×16 tiles = 256×256 px per chunk) at map load; per frame we blit only
  visible chunks, then draw entities y-sorted, then overhang chunks, then FX, then UI
  in screen space.
- Dirty-chunk invalidation when a tile mutates (bomb walls, quake plates).

## 4. Atlas / art bake

- `content::art` exports `const` sprite definitions: `(w, h, frames, &str grid)` where
  each char indexes the master palette (`.` = transparent).
- `engine::atlas` bakes all definitions into one offscreen canvas at startup and
  returns `SpriteHandle`s (atlas rects). Rendering is pure `drawImage` from atlas.
- Palette swap variants (room families, enemy tints) are baked as extra atlas rows.

## 5. Entity model (custom, no external ECS)

```rust
pub struct EntityId { index: u32, gen: u32 }        // generational, safe across despawn
pub struct Entity {
    pub kind: EntityKind,        // enum: Player, Slime, Octorok, Boomerang, Chest, ...
    pub pos: Vec2, pub vel: Vec2, pub facing: Dir4,
    pub body: Option<Body>,      // AABB size, collision layer/mask
    pub health: Option<Health>,
    pub anim: AnimState,
    pub data: EntityData,        // enum with per-kind state machines (AI, timers)
}
pub struct World { arena: Vec<Slot>, free: Vec<u32>, /* tilemap, camera, flags */ }
```

- Systems are plain functions in owner modules: `player::update`, `enemies::update`,
  `combat::resolve_hits`, `fx::update`, called in a fixed order from `game::lib`.
- Cross-cutting communication via an `EventQueue` (`Vec<GameEvent>`) drained once per
  tick — damage dealt, switch toggled, sfx requests, secret found. **No direct calls
  between peer feature modules**; they talk through events or `World` state.

## 6. Map format

- A map = `MapDef { width, height, layers: [ground, detail, overhang], collision,
  spawns: Vec<SpawnDef>, triggers: Vec<TriggerDef>, ambient: RoomFamily }`.
- Authored in `content::maps` as builder code: helper painters
  (`fill`, `rect`, `path`, `scatter`, `stamp(prefab)`) compose regions. Named Act 1
  regions are separate modules **stitched into one contiguous overworld grid** (see
  GAME_DESIGN §4) — the overworld is a single large `MapDef` (~240×240 tiles);
  interiors/dungeon rooms are small separate `MapDef`s with door links.
- Trigger types: door, dialogue, chest, region-name banner, checkpoint, secret, cutscene.

## 7. Input bridge

- `engine::input` merges keyboard/gamepad/touch into one `InputState { move_vec: Vec2,
  buttons: [ButtonState; N] }` with pressed/held/released edges computed per tick.
- Touch backend also exposes its overlay geometry so `game::ui` can skin the joystick
  and buttons in the game's art style; visibility auto-toggles on first touch event.

## 8. Audio bridge

- `game` emits `GameEvent::Sfx(SfxId)` / `SetMusic(TrackId)`; `app` forwards them to
  `engine::audio`. Audio unlock: first pointer/key/touch event resumes the context.
- Synth voices and the pattern sequencer live in `engine::audio`; which notes/patterns
  to play live in `content::audio`. Same platform/data split as art.

## 9. Save

- `game::save_data::SaveGame` (serde) ⇄ `engine::save` (localStorage string I/O).
  Versioned key `shard_save_v1`; unknown/corrupt payloads fall back to New Game.

## 10. Build & deploy

- Dev: `trunk serve` (auto-reload). Release: `trunk build --release` → `dist/`.
- `netlify.toml`: `publish = "dist"`, plus `Cache-Control: no-cache` on `index.html`
  and long cache on hashed wasm/js.
- Deploy: `netlify deploy --prod --dir dist` to site `zelda-fable-5-high-and-grok-4-5-high`.
- CI-friendliness: `cargo check --workspace --target wasm32-unknown-unknown` and
  `cargo clippy --workspace` must pass at every commit.

## 11. Ownership boundaries (anti-megafile / anti-split-brain)

| Area | Owner module | Others may |
|---|---|---|
| web-sys / DOM / canvas / audio / storage | `engine` only | never import web-sys |
| Game loop order, GameMode transitions | `game::lib` + `game::state` | request via events |
| Player verbs & feel constants | `game::player` (+ `combat::tuning.rs` for numbers) | read only |
| Damage/knockback/hitstop/energy/style | `game::combat` | emit AttackHit events |
| Enemy AI | `game::enemies` (one file per family) | spawn via MapDef |
| Puzzle state machines | `game::puzzle` | trigger via events |
| HUD/menus/minimap/touch skin | `game::ui` | expose read-only state |
| Particles/numbers/shake | `game::fx` | emit FxRequest events |
| All maps, art grids, audio patterns, text | `content` | consume via handles/ids |

Rule of thumb: if a change needs edits in two owner modules, the interface is an
event or a `World` field — never a direct cross-module function call.
