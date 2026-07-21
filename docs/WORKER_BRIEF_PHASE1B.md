# WORKER_BRIEF_PHASE1B.md — Enemies, HUD Skin, Art Atlas Pipeline

You are a Grok worker. This brief is self-contained; follow it exactly. Conflict
order: `DECISIONS.md` → `ARCHITECTURE.md` → this brief → `GAME_DESIGN.md`.
You run autonomously — do not ask the human anything except for credentials.

## 0. Mission

Phase 1A shipped a gray-box arena that feels great: full player kit, damage
pipeline, juice, energy/style, functional rect HUD, SFX. You make it **look and
fight** like a real game: the programmatic pixel-art **atlas pipeline** (indexed
color grids → offscreen atlas canvas), real sprites for player/enemies/tiles/HUD,
**three enemy families** (slime, bat, octorok), enemy SFX, the **debug sprite
viewer**, and the HUD skinned in the game's art style. Result = full Phase 1
acceptance: kill waves of 3 enemy families with the full verb set at 60 fps.

## 1. HARD DEPENDENCY — Phase 1A must be merged first

Do not start until `docs/WORKER_NOTES.md` contains the **"Phase 1A completion"**
entry and its commits are on `main`. Read that entry first — where it differs
from this brief's assumptions, **the code + 1A notes win**; note any drift in
WORKER_NOTES and adapt.

Seams 1A guarantees you (verify each in code before building on it):

1. `game::world::World` — spawn/despawn/get/get_mut/iter_alive, `events: Vec<WorldEvent>`,
   `hitstop`, `rng` (fastrand), `camera`, `map`.
2. `game::world::entity` — `Entity { kind, pos, vel, facing, body, health,
   knockback, anim, data }`, `EntityId`, `EntityKind`, `EntityData`, collision
   `Layer` bitmask (PlayerBody/EnemyBody/PlayerHit/EnemyHit/Pickup). You **add**
   variants: `EntityKind::{Slime, Bat, Octorok, OctorokRock}` and
   `EntityData::{Slime(..), Bat(..), Octorok(..), Rock(..)}`. 1A's matches are
   exhaustive on purpose — the compiler will walk you to every site to handle.
3. `WorldEvent` — `AttackHit`, `DamagedPlayer`, `Killed`, `FxRequest`, `Sfx(SfxId)`,
   `StyleAction`, `EnergyDenied`. Enemies communicate **only** via these events
   and `World` state; never call `player`/`combat`/`fx` functions directly
   (ARCHITECTURE §5/§11).
4. `combat::resolve_hits` — hit layers/masks + player-damage semantics documented
   in `game::combat` module docs. Enemy contact/projectile damage goes through
   this pipeline; you get knockback/iframes/hitstop/rank-drop for free.
5. `physics::move_entity` + `query_aabb` — use for all enemy movement/collision.
6. `AnimState { sheet, frame, timer }` — 1A already advances generic frame
   timers; your atlas rendering maps `(kind, anim)` → atlas rects.
7. `content::maps::MapDef` + arena tile-id constants (FLOOR/WALL/FOUNTAIN).
8. `content::audio::sfx` — `SfxId` enum + `spec()` table; **append** enemy ids.
9. `ui::hud` layout constants + `push_toast` — restyle rendering, keep the API.
10. Render order: floor → y-sorted entities → fx → screen-space UI;
    `Draw::set_offset` camera convention.
11. Debug buttons: F1 overlay exists; **F2 (`DEBUG_VIEWER=1`) is reserved for
    your sprite viewer**; H debug projectile exists.
12. Combat tuning consts live in `game::combat::tuning` — put enemy damage/hp/
    speed numbers there too (one place for all numbers).

## 2. Locked constraints

- Stack per `DECISIONS.md`: no new crates (fastrand already in `game` from 1A);
  `engine` = only crate touching `web-sys`; `content` depends on nothing;
  `game` has zero `web-sys`.
- **Art is handcrafted programmatic pixel art** (DECISIONS §3): indexed-color
  text grids in `crates/content/src/art/`; no AI image dumps, no binary assets,
  no base64. Everything diff-reviewable.
- Base tile 16×16; player 16×24 (16×16 collision, bottom-aligned); these enemies
  16×16. Master palette ~48 colors defined once.
- No file >~600 lines — art grid files split by subject (`player.rs`,
  `enemies.rs`, `tiles.rs`, `ui.rs`).
- Commits on `main`, prefixed `phase1b:`, small and frequent. Do not push, do
  not deploy. Every commit: `cargo check --workspace --target
  wasm32-unknown-unknown` + `cargo clippy ... -- -D warnings` clean.
- Environment: `env -u NO_COLOR trunk ...`; never ship `trunk serve`'s dist.
- Tuning ±30% allowed with a WORKER_NOTES log line. Blocked → WORKER_NOTES +
  nearest non-blocking interpretation.

## 3. Milestones (commit at least once each)

### M1 — Palette + atlas pipeline

- `crates/content/src/art/palette.rs` — master palette: ~48 `pub const` RGBA
  entries with a stable index → char mapping. Charset: `.` = transparent, then
  `0-9`, `a-z`, `A-Z` as needed. Organize in commented ramps (greens for grass/
  tunic, grays for stone, skin, reds, blues/teals, golds). One source of truth;
  export `pub fn rgba(idx: u8) -> [u8; 4]` and the char↔index helpers.
- Sprite definition type in `crates/content/src/art/mod.rs`:
  ```rust
  pub struct SpriteDef {
      pub name: &'static str,
      pub w: u32, pub h: u32,        // frame size in px
      pub frames: u32,               // laid out horizontally in `grid`
      pub grid: &'static str,        // h rows of w*frames chars (whitespace-tolerant)
  }
  pub fn all_sprites() -> Vec<&'static SpriteDef>   // registry for baking + viewer
  ```
- `crates/engine/src/atlas.rs` (new; add `pub mod atlas;` to engine lib) —
  bakes at startup:
  - Create an offscreen canvas (`OffscreenCanvas` if available, else a detached
    `document.create_element("canvas")` — detached canvas is simplest and fine).
  - API kept engine-pure (no content dep):
    ```rust
    pub struct AtlasBuilder { /* canvas + ctx + cursor */ }
    impl AtlasBuilder {
        pub fn new(w: u32, h: u32) -> Result<Self, String>;
        // pixels: RGBA rows for one horizontal frame strip
        pub fn add_strip(&mut self, frame_w: u32, frame_h: u32, frames: u32,
                         pixels: &[u8]) -> SpriteHandle;
        pub fn finish(self) -> Atlas;
    }
    #[derive(Clone, Copy)] pub struct SpriteHandle { pub x: u32, pub y: u32,
        pub frame_w: u32, pub frame_h: u32, pub frames: u32 }
    ```
    Blit pixels via `ImageData` + `put_image_data` (add the `ImageData` web-sys
    feature; `OffscreenCanvas`/`OffscreenCanvasRenderingContext2d` features only
    if you use them). Shelf-pack rows; 1024×1024 is plenty.
  - `Draw` gains sprite drawing: store `Option<Atlas>` (or canvas ref) inside
    `Draw` via `pub fn set_atlas(&mut self, atlas: Atlas)`, plus
    ```rust
    pub fn sprite(&self, h: SpriteHandle, frame: u32, x: f32, y: f32, flip_x: bool)
    ```
    using `draw_image_with_html_canvas_element_and_sw_and_sh_and_dx_and_dy_and_dw_and_dh`
    (respect the existing `set_offset` camera convention; disable smoothing once:
    `ctx.set_image_smoothing_enabled(false)`).
  - Decode grids → RGBA in `game` (or `app`) glue: `content` supplies grids +
    palette, `engine` consumes raw pixels — dependency law intact. Suggested:
    `game::assets.rs` module that iterates `content::art::all_sprites()`,
    decodes chars → palette RGBA, feeds `AtlasBuilder`, and returns a
    `SpriteMap { player: SpriteHandle, slime: SpriteHandle, ... }` stored in
    `Game`. Bake once in `Game::new`-time via a `&mut Draw`/builder passed from
    `app` (pick the cleanest wiring; document it in WORKER_NOTES).
- Palette-swap variants (enemy tints / room families): bake extra strips by
  remapping ramp indices at decode time — add
  `pub struct PaletteSwap { pub from: &'static [u8], pub to: &'static [u8] }`
  in content and apply during decode. Use it for at least one variant (e.g.
  red-tint angry slime when at low hp, or the octorok projectile) to prove the
  mechanism for Phase 2.

### M2 — Sprite sheets (content) + debug viewer

Author in `crates/content/src/art/` as indexed grids. Required sheets:

- `player.rs` — 16×24 frames: idle×4 dirs (1 frame each is OK to start, 2 for
  breathing better), walk×4 dirs×4 frames, slash×4 dirs×3 frames, backslash×4
  dirs×3, lunge×4 dirs×2, spin×8 frames (or 4 + rotation illusion), dash×4 dirs
  ×2, shield×4 dirs×1, hurt×1, charge-glow overlay. Left/right may be mirrored
  via `flip_x` — author right-facing only (log this).
  Style: Minish-Cap-like readability — green tunic, big head silhouette, dark
  outline, 2-px feet contact. Feet stay pinned across walk frames (no drift).
- `enemies.rs` — slime (idle squish×2, hop×2, lunge windup×1, splat death uses
  kill-poof fx), bat (fly×3, swoop×1), octorok (idle×2, hide×2, spit×2),
  octorok rock 8×8×2 (spin).
- `tiles.rs` — 16×16: floor (2 variants, subtle checker/noise), wall
  (top + face, so walls read with fake height), fountain (2-frame shimmer),
  pillar block. Map 1A's tile-id constants (FLOOR/WALL/FOUNTAIN) → these; keep
  ids unchanged. Replace 1A's flat-rect floor/wall rendering with sprite tiles
  drawn via the same camera offset (still per-tile `drawImage` this phase —
  chunk caching is Phase 2; the arena is small).
- `ui.rs` — heart (full/half/empty, 8×8), energy bar frame + fill caps, style
  rank chip backdrop, B-item slot frame, small toast panel 9-slice (or 3-slice).
- **Debug sprite viewer** (Phase 1 acceptance #4 depends on it): F2 toggles a
  mode that pauses world updates, shows one sheet at a time at 1× and 3× zoom
  **playing its animation at gameplay timing**, name + frame index via
  `Draw::text`; arrows/move to cycle sheets, attack to cycle anim rates.
  Use it to QA: stable footing, no frame drift, no stray pixels, silhouette
  readable on both floor variants. Fix art until it passes **in motion**.
- Wire player + entity rendering to the atlas: map `(EntityKind,
  facing/state, AnimState.frame)` → `(SpriteHandle, frame, flip_x)` in one
  rendering match (e.g. `game::render_entities`). Y-sort unchanged. Keep the
  1A dummies rendering as a gray palette-swap of the slime sheet (they stay
  useful; fine to keep visible in the arena).

### M3 — Enemy families (`crates/game/src/enemies/`)

Fill 1A's `enemies::update(world, input)` stub: per-family files + `ai.rs` for
shared helpers (line-of-sight, distance-to-player, wander steering, spawn-safe
placement). All numbers in `combat::tuning`. Enemies use `physics::move_entity`
(collide with walls), deal damage **only** through the 1A combat pipeline
(contact/projectile hit layers), flash/knockback/die exactly like the dummy did.
No HP bars — danger reads through animation + sound (GAME_DESIGN §2).

- `slime.rs` — chase + lunge. States: Idle (squish anim, wander ~0.3 px/tick),
  Chase (player within 96 px, LOS: 0.7 px/tick hop-move — move in 12-tick hops
  with 8-tick rests so it reads as hopping), LungeWindup (within 28 px: 20-tick
  telegraph squash + cue sfx), Lunge (4 px/tick, 8 ticks, contact = 1 half-heart),
  Recover (20 ticks, punishable). HP 3. Contact damage only during Lunge and
  Chase overlap.
- `bat.rs` — sine swoop harasser. Perches/hovers out of reach (slow drift,
  ignores walls while flying is fine this phase — body mask excludes solid
  tiles), every ~180 ticks picks a swoop: telegraph flutter 20 ticks + squeak,
  then swoop through the player's position on a sine-offset path (2.2 px/tick,
  contact 1 half-heart), then climbs back. HP 2. Erratic ±sine on hover so it's
  annoying to hit — rewards spin and dash-through.
- `octorok.rs` — ranged lob + hide. Cycle: Idle/aim (turns to face player) →
  Spit (telegraph 20 ticks, then fires `OctorokRock` at 2.5 px/tick toward the
  player's position, straight line this phase) → Hide (ducks 60 ticks,
  **immune to sword while hidden** — flash denied-style feedback if hit, no
  damage) → pop up, repeat ~150-tick cycle. HP 3. Rock: projectile entity,
  1 half-heart on player contact, dies on walls/shield; **must be reflectable
  by perfect block** (1A made reflection generic — flip velocity + change its
  hit layer so a reflected rock damages enemies for 1).
- Spawn/wave director — `enemies/waves.rs`, arena-only scaffolding (Phase 2
  replaces with MapDef spawns): Wave 1 = 3 slimes; Wave 2 = 2 slimes + 2 bats;
  Wave 3 = 2 octoroks + 2 slimes + 1 bat; then repeat with +1 enemy (cap ~10
  alive). Spawn at arena-edge spawn points with a 45-tick telegraph (ground
  shimmer fx + cue) so nothing pops in on top of the player. "WAVE N" toast.
  Clearing a wave: brief lull (120 ticks) + rupee bonus scaling with style rank.
- Enemy SFX: append to `content::audio::sfx` — slime squish/lunge, bat squeak/
  swoop, octorok spit/duck, enemy-hurt variant, spawn-telegraph shimmer, wave
  cue, reflected-rock zing. Distinct per family; extend the app-side adapter
  only if 1A's adapter doesn't already cover new ids automatically.

### M4 — HUD skin + integration polish

- Restyle 1A's rect HUD with `ui.rs` atlas sprites: hearts row, vertical energy
  bar (frame + fill + flash state), style chip (rank letter + pulse), B-item
  slot (shows "—"/empty frame until Phase 2). Keep 1A layout constants + all
  state reads read-only; toasts get the panel background. Touch overlay
  rendering can stay Phase 0 circles (Phase 4 skins it) — just confirm it still
  draws above the HUD.
- Integration checks you own: dash-through awards style verb vs real enemies;
  perfect-block reflect kills octorok with its own rock (satisfying!); pickups
  drop from all three families per 1A's drop table (bump drop rates if combat
  starves energy — log it); fountain corner remains reachable during waves.

## 4. File ownership

You create/own: `crates/content/src/art/*` (palette, mod, player, enemies,
tiles, ui), `crates/engine/src/atlas.rs` (+ lib.rs module line + web-sys
features in engine Cargo.toml), `Draw::set_atlas`/`Draw::sprite` additions in
`crates/engine/src/render.rs`, `crates/game/src/enemies/*`,
`crates/game/src/assets.rs` (decode/bake glue), entity-render mapping, HUD
restyle inside `crates/game/src/ui/*`, debug viewer (in `ui/` or `fx/` —
pick one, note it), `content::audio::sfx` additions.

You **do not** rewrite: player/combat/fx/style/energy logic, `World`/physics
internals, event enums (additive variants only), map layout, tuning values
outside enemy rows (feel deltas ±30% with log lines are fine). If a seam is
genuinely broken, log it in WORKER_NOTES and make the smallest fix.

## 5. Definition of Done

1. Atlas pipeline: all sheets baked into one atlas at startup; entities/tiles/
   HUD render from it; no per-frame canvas creation; pixelated (no smoothing).
2. Player fully sprited (4-dir idle/walk/slash chain/spin/dash/shield/hurt),
   mirrored-left OK; **passes in-motion QA in the F2 viewer at gameplay scale**
   — stable footing, no drift (Phase 1 acceptance #4).
3. Slime, bat, octorok complete per M3: distinct behaviors, ≥20-tick telegraphs,
   correct damage through the combat pipeline (knockback/flash/hitstop/numbers/
   poof/drops all appear on enemies exactly as on dummies), no HP bars.
4. Octorok rock reflectable by perfect block and lethal to enemies after reflect;
   octorok immune while hidden.
5. Wave director: 3+ escalating waves, telegraphed spawns, WAVE toast, clear
   bonus. Full verb set (combo, spin, beam, dash, shield, perfect block)
   meaningfully usable against the mix — satisfies Phase 1 acceptance #1.
6. HUD skinned with atlas sprites; readable at 1× 480×270; energy flash / rank
   pulse / toasts styled.
7. Enemy + wave SFX distinct and wired; nothing shares a cue.
8. 60 fps with 10 enemies + particles + full HUD (check F1 overlay).
9. `cargo check` + `clippy -D warnings` clean (wasm target);
   `env -u NO_COLOR trunk build --release` produces a working `dist/`;
   no file >600 lines; save roundtrip intact.
10. WORKER_NOTES "Phase 1B completion" entry: seams consumed, deviations, art
    QA notes, tuning deltas. Commits on `main` prefixed `phase1b:`, not pushed.

## 6. Verification protocol

1. `cargo check --workspace --target wasm32-unknown-unknown`
2. `cargo clippy --workspace --target wasm32-unknown-unknown -- -D warnings`
3. `env -u NO_COLOR trunk build --release`; serve via
   `python3 -m http.server 8090 --directory dist`.
4. F2 viewer pass: every sheet in motion at 1× and 3×; screenshot the player
   walk cycles for the record; fix drift/footing before anything else.
5. Playthrough (Playwright or manual + screenshots): survive 3 waves using every
   verb at least once; verify each M3 behavior + telegraph; perfect-block an
   octorok rock into an enemy; die once on purpose (hearts → hurt flow → make
   sure death/reset behavior is sane — if 1A left death unhandled, respawn at
   fountain with 3 hearts + toast, and log it).
6. Perf: F1 fps during wave 3 + spin + particles ≈ 60.
7. Touch + keyboard + reload/save checks still pass (Phase 0 regressions).
