# GAME_DESIGN.md — Shard of the Triforce (Act 1 focus)

Design source of truth for gameplay. Numbers here are starting tunings; `game::combat::tuning`
holds the live values and workers may adjust ±30% for feel, logging changes in WORKER_NOTES.

Pillars: Courage/Power/Wisdom entangled (Act 1 ≈ 34/33/33). No arbitrary locks, no
stat-inflation difficulty, no disconnected minigames, no untelegraphed secrets.
**Gameplay feel is priority #1**: movement, combat, juice come before content breadth.

## 1. Player kit (all Act 1)

Movement: 8-way, 1.5 px/tick run (90 px/s), 10-tick accel/decel ramps, 4-dir facing
with 8-way velocity, subtle dust particles on direction snap. Dash: 4.5 px/tick for
10 ticks + 6 tick recovery, i-frames ticks 2–8, costs 25 energy, cancellable into slash.

Sword:
- **Quick slash**: 90° arc in facing dir, 12-tick swing, combos on buffered press.
- **3-hit combo**: slash → backslash → lunging finisher (1.5× dmg, extra knockback,
  small forward hop). 20-tick combo-drop window.
- **Hold-to-charge** (≥30 ticks, drains 20 energy on release): **full spin**, 360°,
  2× dmg, big knockback, distinct wind-up shimmer + release cue.
- **Sword beam** at full hearts: projectile at slash, 0.75× dmg, 8 tiles range.

Shield: hold to block front-90°; blocked hits cost 5 energy, chip-free, small pushback;
perfect-block window (first 6 ticks) reflects projectiles and refunds 10 energy.

Bombs: B-item, 90-tick fuse, 1.5-tile blast, damages player too, opens cracked walls.
B-item cycling: boomerang / bombs (Act 1) on Item button, cycle via Q / d-pad up / touch swipe on item icon.

**Energy meter** (left-side vertical bar, 100 units): fuels dash (25), charge slash (20),
perfect-block refund (+10). Regen 12/s after 0.5 s without spend; fairy fountains and
rare green orbs refill instantly. Meter flashes + dull SFX when an action is refused.

## 2. Combat feel & juice (non-negotiable checklist)

Hitstop 3 ticks (player hits) / 5 (finisher, charge) / 12 (boss phase break).
Knockback scaled by attack, 8-tick decay. Slash arc FX, impact sparks, enemy flash
white 4 ticks, damage numbers (small, pop-and-fade, crits gold), screenshake ≤3 px,
kill poof + rupee/heart/energy drops with pickup magnetism. Callouts ("COMBO FINISH!",
"PERFECT BLOCK", style-rank shifts) as small HUD toasts. Boss-only HP bar (bottom,
segmented per phase). Normal enemies never show HP bars — danger is read through
animation, sound, and hit feedback.

## 3. Style / momentum system (lightweight, readable)

- Rank chip next to energy bar: **D → C → B → A → S**. Points from: varied verbs
  (slash/finisher/charge/dash-through/perfect block/boomerang stun each have a
  short repeat-decay), no-damage streaks. Decays slowly out of combat; drops one
  rank on taking damage.
- Effects (economy only, never damage): B+ → energy regen ×1.5; A+ → dash cost 20;
  S → kills drop +1 rupee and small energy orbs. Exact thresholds in `combat::style.rs`.
- Read: rank chip pulse + short cue on rank change. That's all — no combo counters.

## 4. Act 1 overworld (ambitious, contiguous, scrolling)

One **240×240-tile contiguous map** (~15× a GBA screen per side), smooth-follow camera
(soft-zone + lookahead in facing dir), **no screen-by-screen transitions outdoors**.
Terrain shaping via cliffs/elevation, waterways+bridges, dense forest, gates. Region
name banner fades in on entry. Layout (compass):

- **Mosslight Village** (center-south): hub. Shop (bombs, heart piece 200₹, tunic
  upgrade post-boss), fairy fountain, 6 NPC houses (interiors as rooms), elder intro
  quest → "recover the three gems." Lanterns, drifting leaves.
- **Whispering Grove** (west): dense forest maze shaped by tree walls; **Courage Gem**
  behind a wind-chime puzzle (ring 3 chimes in one boomerang throw — taught by two
  single-chime gates earlier). Secret: bomb-wall cave with rupee cache.
- **Ashen Raider Camp** (north-east): combat pocket; raiders (spear + torch variants)
  around bonfires; **Power Gem** in the war-chest after a 3-wave camp fight with
  destructible barricades. Watchtower telegraphs patrol routes.
- **Echoing Ruins** (east): crumbling arches, wisps + skeletons; **Wisdom Gem** atop a
  plate-and-block puzzle court (weighted plates keep gates open; boomerang flips a
  far switch you can see but not reach).
- **Razor Cliffs** (north): elevation showcase — switchback paths, one-way ledge hops,
  octoroks lobbing from ridges; heart container piece on a summit reached via a
  hidden cave (telegraphed by birds circling + cracked wall).
- **Triforce Shrine** (far north, gated): three gem pedestals open the shrine door →
  dungeon entrance. Scenic vista beat before entering.
- Connective tissue: river from Razor Cliffs through the map with 2 bridges + 1
  broken bridge (boomerang-flip a crank to lower it — optional shortcut), fairy
  fountain grotto near the Grove, 8–10 secrets total (bomb walls, hidden caves,
  rupee caches, 4 heart-piece quarters).

**Overworld minimap**: corner minimap + pause map; explored-terrain fog reveal,
objective marker, POI icons (shop, fountain, dungeon, discovered secrets).

## 5. Act 1 dungeon — Triforce Shrine

Room-based interior (discrete rooms, room-transition slides), discovered-room minimap
with exit feedback; strict exit ⇄ minimap reciprocity. Room-family palette: cool stone
+ teal water accents. Flow:

1. **Temple Vestibule** — safe intro, lore tablet, locked left/right wings.
2. **Hall of Trials** (left wing) — 3-room combat/plate gauntlet → **Gale Boomerang**
   chest (cinematic item get).
3. **Hall of Currents** — boomerang curriculum: wind crystals (toggle blue/amber gates),
   multi-target throws around corners, carry-a-flame puzzle (boomerang snuffs/carries),
   two **seal rooms** (west/east): route one throw through 3 crystals in the right
   order (order telegraphed by floor runes).
4. **Sanctum Core** — both seals broken → central lift; miniboss: **Ironshell duo**
   (shielded — stun by boomerang-to-the-back or perfect block).
5. **Guardian Arena** — boss.

Keys: 2 small keys + boss key, all on visible, clued paths (no blind sweeps).

## 6. Boss — Granite Warden (boomerang-read boss)

48×48 stone golem + two floating **wind crystals** at arena edges. Boss-only HP bar,
cinematic intro (name plate, camera push).

- **Phase 1**: slow slam (shockwave ring), rock-throw fan. Prime **both** wind crystals
  with one or two boomerang throws within 5 s of each other → gale staggers the Warden,
  core exposed 4 s → sword window. 3 windows to break phase.
- **Phase 2** (75%): crystals orbit slowly, slam now spawns 2 pebble crawlers, adds a
  sweeping arm (jump ↔ no — dash-through i-frames are the dodge). Same crystal logic,
  tighter 3.5 s pairing.
- **Phase 3** (35%): arena rim crumbles (smaller floor), rock fan is 5-way, crystals
  swap positions after each stagger; a **fake core flash** teaches "wait for the gale."
- Defeat → heart container, **Shard of Courage**, credits-stub → return to village
  (Act 1 victory state; chapter-select unlock).

Tool gate: core is only exposable via crystals (no DPS cheese). Every attack is
telegraphed ≥30 ticks with distinct audio.

## 7. Enemy roster (Act 1)

slime (chase+lunge), bat (sine swoop), wisp (phases in/out, contact burn), octorok
(ranged lob, hides), raider spear (poke+guard), raider torch (arc throw), skeleton
(shield-probe, boomerang stun), ironshell (front-armored). Combinational pressure:
encounters mix a ranged + melee + harasser rather than raising HP.

## 8. Meta & UX in Act 1 scope

Title (New Game / Select Chapter / mute), chapter select (Act 1 unlocked, 2–3 shown
locked "contract" cards), pause/help overlay (objective + keyboard/gamepad/touch
bindings side by side), credits (auto-scroll, R to skip), shop economy
(rupees 1/5/20; bombs 10₹×5, heart piece 200₹, bomb-bag 100₹, tunic 300₹ post-boss),
localStorage checkpoints at: village start, each gem, shrine entrance, pre-boss.
Full input matrix per DECISIONS §5.

## 9. Acts 2–3 — deferred contracts (do NOT implement yet)

Locked progression contracts so Act 1 code stays forward-compatible:
- Act 2: Storm Hookshot (Chainwright Workshop pressure-plate puzzle), 3 storm sigils,
  west/east conduits, **Tempest Sentinel** (anchor-overload chains, fake core flashes,
  phase-3 support wave). Regions per prompt §6.
- Act 3: Quake Gauntlets (Ruined Forge block/plate puzzle), 3 molten sigils, crucible
  seals, sanctum mastery chain (boomerang rune → hook rune → slam → both pillars),
  **Molten Colossus** (pillar lures + slam windows). Regions per prompt §6.
- Forward-compat requirements on Act 1 code: B-item system supports ≥3 items; puzzle
  crystals/plates/blocks are reusable systems (not one-off flags); chapter starts
  preload earlier tools; map loader supports multiple overworlds.
