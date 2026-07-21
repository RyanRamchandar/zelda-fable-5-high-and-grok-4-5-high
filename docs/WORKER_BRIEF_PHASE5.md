# WORKER_BRIEF_PHASE5.md — Polish, music, perf, validation, deploy (Gate C)

You are a Grok worker. This brief is self-contained; follow it exactly. Conflict
order: `DECISIONS.md` → `ARCHITECTURE.md` → this brief → `GAME_DESIGN.md`.
You run autonomously — the ONLY allowed human escalations are credentials
(`netlify login`, `gh auth login`); attempt everything first and document
exact blockers.

**GATE: do not start until the "Phase 4 completion" entry exists in
`docs/WORKER_NOTES.md`.** Read it plus every "residual risks" line in earlier
entries — M1 is built from that list. Code + WORKER_NOTES win over this brief;
log drift and adapt.

Single worker, milestones strictly in order (deploy is last so everything
before it lands in the shipped build). Commit prefix `phase5:`.

## 0. Mission

Act 1 is feature-complete (Gate B) and wrapped in its meta shell (Phase 4).
Phase 5 makes it **shippable**: the accumulated feel debts get paid, every
area gets music, perf holds ~60 fps everywhere, one clean full playthrough is
recorded in a real browser, and the game goes live at the locked Netlify slug
with the repo pushed to GitHub. Gate C closes here.

## 1. HARD DEPENDENCY — code reality (verify each before building)

Written against post-Phase-4 main. Verify:

1. **Audio**: `engine/src/audio.rs` = one-shot SFX synth (osc/noise +
   envelope) + `set_muted` (Phase 4). There is **no sequencer**;
   `content/src/audio/` has `sfx.rs` only. App adapter maps
   `GameEvent::Sfx`/`Save`/`SetMuted` in `crates/app/src/lib.rs`.
2. **Frame glue**: `engine::time::run_loop(window, FnMut(steps))` — the app
   closure is the only per-rAF hook; music scheduling ticks from there.
3. **Events**: `GameEvent` enum in `game/src/lib.rs`, drained per update in
   `game/src/events.rs`. Adding `SetMusic(TrackId)` follows the exact
   `Sfx`/`SetMuted` precedent (content id → app adapter → engine).
4. **Region signal**: `WorldEvent::RegionEntered` + banner already fire on
   region triggers; `game.current_map: MapId` distinguishes overworld/
   interiors/dungeon; boss state lives in `game.boss`
   (`boss::update`, intro/victory flow); credits = `ui/credits.rs`;
   title mode = Phase 4 `GameMode::Title`.
5. **Feel-debt registry** (from WORKER_NOTES residual-risk lines — re-read
   each, the code may have moved):
   - perfect-block rock reflect feel (1B), tap-K-bomb vs hold-K-shield
     tension, chime finale 4 s window, block push latency (2C-A)
   - camp wave-2/3 spike, wisp harassment, raider-spear guard readability,
     torch arc aim (2C-B)
   - seal-room throw tolerance, Trials shutter fight density (3A)
   - Warden telegraph readability, fake-core-flash teach, death mid-P2 →
     checkpoint-9 retry flow, Ironshell bomb-cheese margin (3B)
   - tunic cosmetic palette swap deferred (3B); ambient leaf/ember/fountain
     particles not shipped (2B); NPC/prop art at "readable stub" bar (2B);
     animated water overdraw if >180 visible (2A); real-iPhone verification
     owed (2A → Phase 4)
6. **Tuning home**: `game/src/combat/tuning.rs` (±30% latitude per
   GAME_DESIGN header, log every change). Sprite QA loop = F2 viewer
   (`ui/viewer.rs`). Debug: F1 overlay, F1+H grants, F3 map cycle, F4
   teleport — leave all in (they're behind F1 gates), but verify they can't
   fire from touch/gamepad-only play.
7. **Deploy config**: `netlify.toml` has `publish = "dist"` + no-cache on
   `index.html` only. No git remote is configured. `Trunk` emits hashed
   `.js`/`.wasm`. Known env quirk: `NO_COLOR=1` breaks trunk — always
   `env -u NO_COLOR trunk build --release`; never ship `trunk serve` output.

## 2. Locked constraints

- No new crates; no `web-sys` outside `engine`; notes/patterns data in
  `content::audio::music`, synth voices in `engine::audio` (DECISIONS §4).
- Netlify slug is LOCKED: `zelda-fable-5-high-and-grok-4-5-high` (dashes,
  never underscores). Deploy from a fresh local release build.
- Tuning changes stay inside ±30% of GAME_DESIGN numbers, each logged in the
  WORKER_NOTES completion entry. No new mechanics, no new content areas.
- `SAVE_VERSION` stays 2 (`#[serde(default)]` for any new settings field).
- Files <600 lines; commit per milestone; check/clippy/trunk clean at every
  commit.

## 3. Milestones

### M1 — Feel + readability pass (the debt list)

Work through §1.5 top to bottom. For each item: reproduce in browser, fix or
tune, and one-line-log the disposition (fixed / tuned X→Y / won't-fix because…)
in the completion entry. Mandatory specifics:

1. **Perfect block**: reflect an octorok rock and block a raider poke on
   keyboard AND gamepad; if the 6-tick window feels unhittable at 60 fps,
   widen toward 8 (±30% bound) and/or add a 2-tick pre-flash on the shield
   sprite so the timing is readable.
2. **Camp waves**: full 3-wave war-chest fight from a fresh save (no debug
   grants); wave 2/3 must be tense-not-spiky — adjust spawn counts/mix via
   the placement defs (spawner group slots), not enemy stats, first.
3. **Warden**: full New Game → Warden kill playthrough (this doubles as M5
   groundwork); confirm every attack telegraph reads ≥30 ticks, the P3
   fake-core flash teaches (differs visibly + audibly from the real
   4 s window), death mid-P2 resumes at checkpoint 9 with intro-short path.
4. **Tunic**: implement the deferred cosmetic — palette-swap row for the
   player sheet (atlas bake precedent: `PaletteSwap` in
   `content/src/art/palette.rs`), applied when `TUNIC_BOUGHT`; verify in F2
   viewer in motion.
5. **Ambient life** (2B debt, GAME_DESIGN §4): drifting leaves in
   Grove/village, lantern ember glints in village evenings, fountain
   sparkle — as `fx` particles spawned by region, hard-capped (≤24 live
   ambient particles) and excluded when `map_stats.direct` (perf fallback).
6. **Sprite QA in motion**: F2-viewer sweep of player (incl. tunic swap), all
   8 enemy families, Ironshell, Warden frames at 1×; fix foot-slide/frame
   drift only — no art-direction rework.

### M2 — Music: sequencer + per-area tracks

1. **Engine** (`engine/src/audio.rs`, keep <600 — split `audio/music.rs`
   submodule if needed): chiptune pattern sequencer, 2 pulse + 1 triangle +
   1 noise channel (DECISIONS §4). WebAudio-time lookahead scheduler: a
   `tick_music(now)` called once per rAF from the app closure schedules all
   note-ons falling in the next ~0.25 s (classic two-clock pattern — do NOT
   schedule from game ticks). Master music gain node (~0.35) separate from
   SFX; `set_muted` silences both.
2. **Content** (`content/src/audio/music.rs`): `TrackId { Title, Village,
   Overworld, Dungeon, Boss, Victory }` + `MusicTrack { bpm, channels:
   [&[Note]; 4], loop_len }` where `Note { start, len, midi, vol }` (or
   equivalent plain-data shape). Compose 6 loops: Title 8-bar wistful,
   Village 16-bar warm, Overworld 16-bar adventurous, Dungeon 16-bar cool/
   sparse, Boss 8-bar driving, Victory 8-bar fanfare-then-calm. Keep each
   channel readable data (pattern helper fns allowed in content).
3. **Game**: `GameEvent::SetMusic(TrackId)` emitted from a tiny
   `game::music_director` (new file, ~80 lines): Title mode → Title;
   `current_map` Village-region overworld → Village (region from
   `RegionEntered`, default Overworld); interiors keep the overworld's
   choice; Dungeon → Dungeon; Warden intro start → Boss; victory/credits →
   Victory; re-emit only on change. App adapter: track switch = stop channels
   + start new track (hard cut is fine; a 0.2 s gain dip is a bonus).
4. Every prompt-§12-style cue check: walk the SFX id list in
   `content/src/audio/sfx.rs` and verify each fires distinctly in play with
   music running (adjust per-SFX gain if masked).

### M3 — Performance pass

1. Measure with F1 overlay in the three hot spots: camp wave 3, dungeon
   Currents (water + crystals), Warden P3 (pebbles + rim + fan). Target ~60,
   floor 55.
2. Levers (in order): particle caps (`fx`), animated-tile overdraw list
   coarsening (2A seam), chunk budget (48) / bakes-per-frame (2), enemy
   projectile caps. ≤12 active AI stays the law (2C).
3. One `console.log` startup line with build info (version/commit short hash
   via `env!` if trivial, else static string) — helps live-site bug reports.
   No per-frame logging.

### M4 — Validation sweep (prompt §13)

All in a real browser against a fresh release build (server:
`python3 -m http.server 8090 --directory dist`; kill server + headless
afterwards):

1. **Full critical path, keyboard, fresh save**: title → New Game → elder →
   3 gems (real mechanisms) → shrine → boomerang → 2 seals → Ironshell →
   Warden → credits → title Chapters "COMPLETE". No console errors, no
   soft-locks; screenshot each beat to `/tmp/p5_validation/`.
2. **Puzzle reset audit**: die/reload mid chime-finale, mid plate-court, mid
   seal room, mid Warden P2 → each resets to a solvable state (3A/2C reset
   rules).
3. **Reciprocity + minimap**: dungeon load assert clean; walk every dungeon
   exit both ways; overworld fog/POI/objective sane on Continue.
4. **Touch spot-check** (Playwright mobile emulation): title → New Game →
   first chime gate → shop purchase; pause Help readable. **Gamepad
   spot-check** if hardware present, else re-mark as owed.
5. **Save matrix**: no-save cold boot; Gate-B-era save (no `muted` field)
   continues cleanly; corrupted JSON → New Game (existing `from_json`
   fallback).
6. Fix what the sweep finds before proceeding; re-run the failing check.

### M5 — README + docs final

1. README: what the game is (1 paragraph + screenshot), play-now Netlify
   link (add after M6 deploy), controls table (keyboard/gamepad/touch),
   run/build/deploy instructions, architecture pointer to `docs/`, Act 1
   content summary, known-issues list (from remaining owed items).
2. Screenshot: title screen + one action shot committed under `docs/media/`
   (small PNGs, <300 KB each).

### M6 — Netlify production deploy (locked slug)

1. Build: `cargo check` + `clippy -D warnings` (wasm32) clean, then
   `env -u NO_COLOR trunk build --release`.
2. `netlify.toml`: add immutable long-cache for hashed assets before deploy:

```toml
[[headers]]
  for = "/*.wasm"
  [headers.values]
    Cache-Control = "public, max-age=31536000, immutable"
[[headers]]
  for = "/*.js"
  [headers.values]
    Cache-Control = "public, max-age=31536000, immutable"
```

3. **Credential escalation protocol**: run `netlify status`. If not logged
   in, run `netlify login` (opens browser) and STOP with a clear WORKER_NOTES
   line + chat message telling the human to complete the browser auth, then
   resume. Do not thrash retries.
4. Site link: `netlify sites:list` → if `zelda-fable-5-high-and-grok-4-5-high`
   exists, `netlify link --name zelda-fable-5-high-and-grok-4-5-high`; else
   `netlify sites:create --name zelda-fable-5-high-and-grok-4-5-high` (team
   default). NEVER create an underscore variant; if the name is taken by
   another team, escalate rather than renaming.
5. `netlify deploy --prod --dir dist`. Verify the live URL: loads, title
   renders, New Game reaches the overworld, no console errors (Playwright
   against the live URL). Paste the URL in README (M5) + completion entry.
   Real-iPhone Safari check: attempt only if the human offers a device;
   otherwise document as owed with the mobile-emulation evidence.

### M7 — GitHub push

1. `git remote -v` is empty. Run `gh auth status`; if unauthenticated, run
   `gh auth login` and STOP for the human (same protocol as Netlify), then
   resume.
2. Create + push: `gh repo create zelda-fable-5-high-and-grok-4-5-high
   --public --source . --remote origin --push` (dashed name to match the
   slug; if the name is taken, escalate). Verify `git log origin/main -1`
   matches local HEAD.
3. History hygiene check before push: `git status` clean, no `dist/` or
   secrets tracked (`.gitignore` should already cover `dist/`; verify).

### M8 — Gate C completion entry

Append "Phase 5 completion — Gate C" to WORKER_NOTES: feel-debt disposition
table, tuning log, music tracks shipped, perf numbers per hot spot,
validation results + screenshot paths, live URL, GitHub URL, anything still
owed (device pass, gamepad hardware) — explicit and honest.

## 4. Definition of Done (Gate C)

1. Every §1.5 debt item dispositioned; tunic swap + ambient particles live.
2. Six music tracks playing in the right places; mute kills both buses; all
   SFX cues distinct under music.
3. ~60 fps (≥55 floor) in camp wave 3, Currents, Warden P3 on a mid laptop.
4. M4 validation sweep green, screenshots archived.
5. README final with live URL; `docs/media/` screenshots committed.
6. Live at `https://zelda-fable-5-high-and-grok-4-5-high.netlify.app` (or
   team-domain equivalent), verified by browser against production.
7. `main` pushed to GitHub, remote HEAD == local HEAD.
8. check/clippy/trunk clean; files <600; Gate C entry appended.

## 5. File ownership (Phase 5)

- **New**: `engine/src/audio/music.rs` (or in-file if it fits),
  `content/src/audio/music.rs`, `game/src/music_director.rs`,
  `docs/media/*.png`.
- **Edit**: `engine/src/audio.rs`, `content/src/audio/{mod,sfx}.rs`,
  `game/src/{lib,events,fx/*,combat/tuning}.rs`, feel-pass touches inside
  `game::{player,enemies,boss,puzzle}` (tuning + telegraph readability only),
  `content/src/art/*` (tunic row, QA fixes), `crates/app/src/lib.rs`
  (SetMusic arm + tick_music), `netlify.toml`, `README.md`, `.gitignore` if
  needed.
- **Do not touch**: map geometry, room topology, flag/catalog id allocations,
  save schema (beyond `#[serde(default)]` additions), Phase 4 menu flows.
