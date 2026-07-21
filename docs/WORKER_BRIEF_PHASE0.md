# WORKER_BRIEF_PHASE0.md — Phase 0: Rust/WASM Scaffold (Gate A)

You are a Grok worker. Implement **Phase 0 only**. Do not build gameplay, art, maps,
or menus. Do not add dependencies or restructure modules beyond this brief. If
something is ambiguous: DECISIONS.md → ARCHITECTURE.md → this brief. Log any
deviation in `docs/WORKER_NOTES.md` (create it, append-only) and keep going.

## Objective

A running Rust→WASM scaffold: 480×270 letterboxed canvas, fixed 60 Hz loop, a
placeholder player rectangle movable with **keyboard, gamepad, and touch**, one
WebAudio beep on the Attack button, position saved/restored via localStorage, and a
`trunk build --release` static `dist/` ready for Netlify. **Do not deploy** (that is
Phase 5); just make `dist/` servable.

## Toolchain setup (run these; skip what's already installed)

```bash
cd /Users/ryan/Documents/workspace/vibes/zelda_2d_eval/zelda_fable_5_high_and_grok_4_5_high
rustup target add wasm32-unknown-unknown
cargo install trunk --locked        # skip if `trunk --version` works
git init 2>/dev/null; git branch -M main   # repo may already be initialized — check first
```

If `cargo install trunk` is too slow/fails, `cargo install trunk --locked --no-default-features --features rustls-tls` is the fallback. If trunk cannot be installed at all, STOP and write the blocker to `docs/WORKER_NOTES.md`.

## Workspace to create (exact layout — matches ARCHITECTURE.md §1)

```
Cargo.toml                  # [workspace], resolver = "2", members = ["crates/*"]
Trunk.toml
index.html
netlify.toml
.gitignore                  # /target, /dist, .DS_Store, .netlify
README.md
crates/app/                 # wasm-bindgen entry; index.html points here
crates/engine/
crates/game/
crates/content/             # Phase 0: lib.rs with empty modules + doc comment only
```

Root `Cargo.toml` also sets:

```toml
[profile.release]
opt-level = "s"
lto = true
```

Dependencies (add with `cargo add` to get current versions; do not hand-pin guesses):
- `engine`: `wasm-bindgen`, `js-sys`, `console_error_panic_hook`, and `web-sys` with
  features: `Window, Document, HtmlCanvasElement, CanvasRenderingContext2d, Element,
  CssStyleDeclaration, KeyboardEvent, TouchEvent, TouchList, Touch, Gamepad,
  GamepadButton, Navigator, Storage, AudioContext, AudioContextState, OscillatorNode,
  OscillatorType, GainNode, AudioParam, AudioDestinationNode, Performance, DomRect`
  (add more features as compile errors demand — that's normal).
- `game`: `serde` (derive), `serde_json`. **No web-sys in `game`, ever.**
- `app`: `wasm-bindgen` + path deps on `engine`, `game`, `content`.
- `content`: no deps.

## index.html (trunk entry)

- `<link data-trunk rel="rust" data-wasm-opt="z" href="crates/app/Cargo.toml" />`
- One `<canvas id="game">`; CSS: black page background, canvas centered with
  `image-rendering: pixelated`, `touch-action: none` on canvas and body.
- `<meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=1, user-scalable=no, viewport-fit=cover">`
- Title: "Shard of the Triforce".

`Trunk.toml`: `[build] target = "index.html", dist = "dist"`.
`netlify.toml`:

```toml
[build]
  publish = "dist"
[[headers]]
  for = "/index.html"
  [headers.values]
    Cache-Control = "no-cache"
```

## Module skeletons to implement (keep each file small; stubs where noted)

### crates/engine (platform layer — the real work of Phase 0)

- `canvas.rs`: grab `#game`, set internal size 480×270, compute integer scale from
  `window.innerWidth/Height`, set CSS size (letterbox via centering), re-run on
  `resize`. Expose logical→client coordinate mapping for touch.
- `time.rs`: rAF loop via `Closure` self-rescheduling; fixed-step accumulator
  (dt = 1/60 s, clamp frame delta to 0.1 s); calls `update()` 0..n times then `render()`.
- `render.rs`: thin `Draw` struct over `CanvasRenderingContext2d`: `clear(color)`,
  `rect(x, y, w, h, color)`, `text(s, x, y, color)` (canvas `fillText` is fine for
  Phase 0). No camera yet.
- `input/mod.rs`: `InputState { move_vec: (f32, f32), buttons: [Button; 6] }` for
  Attack, Item, Dash, Interact, Pause, Confirm; each `Button { held, pressed }` with
  edges computed once per update tick.
- `input/keyboard.rs`: keydown/keyup listeners on window (prevent default for game
  keys). WASD + arrows = move; J/Space=Attack, K=Item, L/Shift=Dash, E=Interact,
  Esc/Enter map to Pause/Confirm.
- `input/gamepad.rs`: poll `navigator.getGamepads()` every frame; standard mapping:
  axes 0/1 + d-pad (buttons 12–15) = move (0.25 deadzone); button 0=Attack, 2=Item,
  1=Dash, 3=Interact, 9=Pause.
- `input/touch.rs`: touchstart/move/end on canvas. Left half = floating joystick
  (anchor at first touch, 24 px logical max radius → move_vec). Right half: two
  circular buttons (Attack, Dash) at fixed logical positions (~430,230) and
  (~400,250), radius 18 px. Track multi-touch by identifier. Expose
  `overlay_geometry()` so render can draw translucent circles; only after first
  touch event sets a `touch_active` flag.
- `audio.rs`: lazy `AudioContext`; `resume()` on first key/pointer/touch;
  `beep(freq_hz, dur_s)` = oscillator (square) → gain envelope (attack 5 ms, decay to
  0) → destination.
- `save.rs`: `fn save(key: &str, json: &str)` / `fn load(key: &str) -> Option<String>`
  over `window.localStorage`.
- `lib.rs`: `Platform` struct wiring all of the above; `console_error_panic_hook::set_once()`.

### crates/game

- `lib.rs`: `Game { x: f32, y: f32 }` (spawn 240,135). `update(&mut self, input:
  &InputState)`: normalize move_vec, move 1.5 px/tick, clamp to bounds; on
  Attack.pressed → return an event asking for a beep (define a tiny
  `enum GameEvent { Beep }` + `Vec<GameEvent>` return — this seam becomes the real
  event queue later). Every 60 ticks, serialize `{x, y}` via serde_json for saving.
- `render(&self, d: &mut Draw)`: clear dark green `#1a3b2a`, draw 16×24 white rect at
  (x, y), draw "Shard of the Triforce — Phase 0" text top-left, draw touch overlay
  circles if `touch_active`.
- `save_data.rs`: `#[derive(Serialize, Deserialize)] struct SaveGame { x: f32, y: f32 }`,
  key `"shard_save_v1"`. Load at startup; corrupt/missing → default spawn.

### crates/app

- `lib.rs`: `#[wasm_bindgen(start)] fn start()`: build Platform, load save, build
  Game, start loop. Loop body: poll gamepad → snapshot InputState → fixed updates
  (forward `GameEvent::Beep` to `engine::audio`, periodic save via `engine::save`) →
  render.

### crates/content

- `lib.rs` with `pub mod art; pub mod audio; pub mod maps;` as empty stub modules and
  a doc comment pointing at ARCHITECTURE.md §1. Nothing else.

## README.md

Project name, one-line pitch, link to `docs/`, then exactly:

```
## Run
rustup target add wasm32-unknown-unknown
cargo install trunk --locked
trunk serve            # dev, http://localhost:8080
trunk build --release  # static output in dist/
## Checks
cargo check --workspace --target wasm32-unknown-unknown
cargo clippy --workspace --target wasm32-unknown-unknown
```

## Verification you must actually perform

1. `cargo check --workspace --target wasm32-unknown-unknown` — clean.
2. `cargo clippy --workspace --target wasm32-unknown-unknown` — no warnings you
   introduced.
3. `trunk build --release` — `dist/` contains index.html + hashed `.wasm`/`.js`.
4. `trunk serve` and verify **in a real browser** (headless/browser tool ok):
   canvas letterboxes on window resize; arrow keys move the rect smoothly; Attack
   beeps; reload restores position. Emulate touch (device-mode) and verify the
   joystick moves the rect and overlay circles render. If no physical gamepad is
   available, code-review the mapping against the standard layout and note
   "gamepad untested on hardware" in WORKER_NOTES.md.
5. Serve `dist/` statically (`python3 -m http.server -d dist 8081`) and confirm it
   loads — proves Netlify-readiness.

## Git

Commit granularity: (1) `phase0: workspace + trunk scaffold`, (2) `phase0: engine
platform layer`, (3) `phase0: game stub + save`, (4) `phase0: readme + netlify
config`. Do **not** push anywhere.

## Definition of Done

- All 5 verification steps pass.
- Layout exactly matches ARCHITECTURE.md §1 (empty dirs may be omitted).
- No gameplay beyond the moving rect + beep. No TODO-stubs in engine code paths.
- No dependency outside the DECISIONS.md §2 allowlist.
- `docs/WORKER_NOTES.md` exists with a short completion report (what was verified,
  any deviations, gamepad-hardware status).
