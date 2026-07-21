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
