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
