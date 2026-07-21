# DECISIONS.md — Locked Decisions (Source of Truth)

This file is the single source of truth for every contested or ambiguous decision.
**Workers never invent architecture.** If a worker hits a contradiction between code,
briefs, and this file: this file wins, then `ARCHITECTURE.md`, then the phase brief.
Update flow: only the PLANNER edits `docs/`. Workers propose changes via notes in
`docs/WORKER_NOTES.md` (append-only) and keep building.

## 1. Product

- Game: **Shard of the Triforce** — original top-down action-adventure, "Minish Cap
  cleanline" readability. Three-act campaign; **Act 1 is polished deeply first**,
  Acts 2–3 are deferred contracts (see `GAME_DESIGN.md` §9).
- Done bar: Gate C ships a deeply polished, fully playable Act 1 on Netlify.
  Acts 2–3 follow the same contracts afterward.

## 2. Stack (LOCKED)

| Concern | Decision |
|---|---|
| Language | **Rust**, stable toolchain, target `wasm32-unknown-unknown` |
| Engine | **Custom minimal engine** — no Bevy/macroquad/ggez. `wasm-bindgen` + `web-sys` + `js-sys` only |
| Rendering | **Canvas2D** onto a fixed internal canvas **480×270** (16 px tiles), integer-scaled + letterboxed to the window. Tile layers cached in pre-rendered offscreen chunk canvases |
| Loop | `requestAnimationFrame` render + **fixed 60 Hz update** with accumulator |
| ECS | **No external ECS.** Custom entity store: generational-index arena (`EntityId { index, gen }`), entities as structs with optional component fields, systems as plain functions. See `ARCHITECTURE.md` §5 |
| Map format | **Data-as-code**: Rust map-builder functions in `crates/content` paint layered tile grids at startup. No binary/Tiled assets. See `ARCHITECTURE.md` §6 |
| Build | **Trunk** (`trunk build --release`) → static `dist/`. No JS bundler, no npm for the game itself |
| Deploy | Netlify CLI from local build: `netlify deploy --prod --dir dist`. Site slug **`zelda-fable-5-high-and-grok-4-5-high`** (dashes only, never underscores) |
| Crates allowed | `wasm-bindgen`, `web-sys`, `js-sys`, `console_error_panic_hook`, `serde`, `serde_json`, `fastrand`. **Anything else requires a PLANNER decision here first** |

Why custom engine: keyboard + gamepad + touch + WebAudio are all hard requirements,
and `web-sys` exposes each as a first-class API with no engine gaps (macroquad has no
web gamepad story; Bevy WASM builds are heavy and slow to iterate). A 2D tile game at
480×270 with chunk-cached layers is comfortably 60 fps in Canvas2D, including iPhone.

## 3. Art pipeline (LOCKED)

- **Handcrafted programmatic pixel art. No AI-generated image dumps, no stock assets.**
- Sprites and tiles are authored as **indexed-color text grids** (strings of palette
  indices) in `crates/content/src/art/`, baked into an in-memory atlas canvas at
  startup. Everything is reviewable in diffs and consistent by construction.
- Palette discipline: one master palette (~48 colors) + per-room-family sub-palettes
  (overworld / dungeon / boss / secret). Defined once in `content::art::palette`.
- Base tile 16×16. Player 16×24 (16×16 collision). Enemies 16×16 or 24×24. Boss 48×48+.
- Procedural layers on top: particles, glows, shadows, water shimmer, slash arcs are
  drawn programmatically, not sprited.
- Animation QA rule: every sheet gets checked **in motion at gameplay scale** (debug
  viewer, Phase 1) before it's called done — stable footing, no frame drift.

## 4. Audio (LOCKED)

- **WebAudio synthesis, no audio files.** `engine::audio` owns one `AudioContext`
  (created/resumed on first user gesture).
- SFX = data-driven descriptors (oscillator type, envelope, pitch sweep, noise mix)
  defined in `crates/content/src/audio/sfx.rs`.
- Music = lightweight pattern sequencer (chiptune: 2 pulse + 1 triangle + noise
  channels) with per-area tracks defined as note patterns in `content::audio::music`.
- Every key state change has a distinct cue (see prompt §12 list).

## 5. Input (LOCKED, all three are hard requirements)

- Unified `InputState` (move vector + virtual buttons: Attack, Item, Dash, Interact,
  Pause, MenuNav) fed by three backends in `engine::input`:
  - **Keyboard**: WASD/arrows + J/K/L/Space/Shift/E/Enter/Esc (rebind list in GAME_DESIGN §8).
  - **Gamepad**: `navigator.getGamepads()` polled each frame; standard mapping
    (Xbox/PS): left stick + d-pad move, A/Cross attack, X/Square item, B/Circle dash,
    Y/Triangle interact, Start pause.
  - **Touch**: game-rendered virtual joystick (left) + A/B/X buttons (right) drawn on
    the canvas itself; auto-shown on touch devices; multi-touch; `touch-action: none`
    + viewport meta to kill browser gestures/zoom.

## 6. Save (LOCKED)

- `localStorage`, JSON via serde. Chapter checkpoints (act + spawn point + flags),
  rupees carry across chapter-select runs, settings (volume). Key: `shard_save_v1`.

## 7. Autonomy gates (LOCKED)

- **Gate A** — architecture + scaffold: Phase 0 done (playable stub proving render,
  loop, all three input backends, audio unlock, save roundtrip, Netlify-ready dist).
- **Gate B** — Act 1 critical path playable end-to-end (Phases 1–3): gems → shrine →
  boomerang → seals → Granite Warden defeated → credits stub.
- **Gate C** — polish + meta + deploy (Phases 4–5): title/chapter select/pause,
  touch+gamepad verified, audio/juice pass, live Netlify URL.
- Between gates workers run **without asking the human**. Escalate to human only for
  credentials/external access (e.g. `netlify login`, GitHub push auth).

## 8. Git (LOCKED)

- Single `main` branch, frequent small commits with clear messages
  (`phase0: ...`, `phase1: ...`). No PR churn. Push to GitHub at/after Gate C
  (needs human credentials — that is an allowed escalation).

## 9. Conflict resolution (LOCKED)

1. `DECISIONS.md` → 2. `ARCHITECTURE.md` → 3. current phase brief → 4. `GAME_DESIGN.md`
   flavor text.
2. Workers never restructure crates/modules or add dependencies on their own.
3. If truly blocked, write the blocker to `docs/WORKER_NOTES.md` and continue with the
   nearest non-blocking interpretation; the PLANNER reconciles at the next gate.
