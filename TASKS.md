# TASKS — what's done, what's next

The single source of truth for "where are we and what's left". Update at
the end of each working session. Phases reference the plan in
`DESIGN.md` §8.

Legend:
- ✅ done & landed
- 🟡 in progress / partially landed
- ⬜ not started
- 🧊 deferred (with reason)
- 👤 needs Adi (only the human can do it)

---

## ✅ Done — Phase 0 (foundations)

### Engine substrate (Vortexel)
- ✅ Bevy 0.14 ECS scaffold; one plugin per simulation slice (world,
  weather, physics, creatures, combat, treasure, player, render).
- ✅ Deterministic chunked world generation — domain-warped Perlin
  heightfield, 12 biomes, biome-driven temperature + monster spawn bias.
- ✅ Chunk streaming around the player; despawn on exit; bounded RAM
  regardless of map size.
- ✅ Cell-based weather grid with advection, fog/rain/wind/temperature/
  freeze/heat/storm; weather events (lightning, freeze front, waterspout).
- ✅ Buoyancy + wave + wind physics on Body+Floats components;
  semi-implicit (symplectic Euler) integrator.
- ✅ Environment tagging on physics bodies (submerged / in_rain / in_fog /
  in_heat / in_freeze) — what the rest of the game queries.
- ✅ `combat::environmental_damage` rule: submerged + can't breathe water
  → Water damage. Drowning a fire dragon emerges from data, not script.
- ✅ Affinity system + DamageKind enum; per-species archetypes centralise
  balance.
- ✅ Species roster: Kraken, Sea Serpent, Fire/Ice/Sand Dragon, Wyvern,
  Giant Snake, Undead, Eldritch.
- ✅ Treasure component + Loot enum + biome-biased loot tables.
- ✅ Player ship + Captain stats + WASD/Space/Shift controls + camera follow.
- ✅ Render layer: per-chunk vertex-coloured terrain mesh, water plane
  that follows the player, ship hull/mast/sail primitives, creature visuals.
- ✅ Spawn fix: ship lands in shallow sea within sight of an island
  (was previously dropped at world origin which Perlin loved to make a
  mountain).

### Process & docs
- ✅ `DESIGN.md` — pillars, phase plan, cuts, low-spec rules, open Qs.
- ✅ `WORKFLOW.md` — Adi vs Claude responsibilities, day-to-day loop,
  bug-report template, branch policy.
- ✅ `ASSETS.md` — sources, repo layout, naming, RON content example,
  licence ledger, style guardrails (Enshrouded primary / Valheim floor),
  AI-3D workflow (Tripo → Blender) for hero assets.
- ✅ `README.md` — game-and-engine intro under the new names.
- ✅ Mobile/wasm build removed (was a temp stopgap while away from PC).
- ✅ Project rename: **The Unseeing Tides** (game) on **Vortexel** (engine);
  cargo package renamed `pien` → `unseeing_tides`.

### Decisions resolved
- ✅ Art direction — Valheim-grounded low-poly (geometry) +
  Enshrouded-level shader/post polish (visual), tiered Potato/Standard/Nice.
- ✅ Save target — desktop only, local file. (Web/IndexedDB dropped.)
- ✅ AI 3D generation allowed for hero assets (paid tier, post-process in
  Blender, generic prompts only). 2D AI images still banned.
- ✅ Custom Blender modelling allowed but only as *reskin / retopo* on CC0
  bases for hero assets; from-scratch is hobby stretch.

---

## 🟡 / ⬜ Next up — Phase 1: Survival slice

Goal (DESIGN.md §8): you can sail across biomes, get caught in a storm,
beach on an island, save, quit, reload, and continue.

- ✅ Spawn lands in open water near an island.
- ⬜ HUD — surface Captain stats (stamina / sanity / warmth) + ship hull
  bar + a compass / heading readout. `bevy_ui`, top-left + top-right
  corners. No frills yet.
- ⬜ Save / load to a local file (JSON or RON via serde). Save key state:
  player transform, captain, ship, inventory, world seed. `Ctrl+S` saves,
  `Ctrl+L` loads on a fresh boot.
- ⬜ Visible weather — rain particles in rainy cells, fog tint shader on
  the camera, freeze halo when in cold cells. Driven by the existing
  WeatherCell field.
- ⬜ Currents — `Current` resource + a slow vector field (separate Perlin
  on the same world coords) that adds drift to floating bodies. Tiny
  system in `physics`.
- ⬜ Hull damage — collision with chunk terrain or storm waves drops
  `Ship.hull`. Below 20%: ship slows; below 0: game-over screen + reload
  last save.
- ⬜ Phase 1 Done test: real sail-storm-beach-save-reload run on PC.

---

## ⬜ Phase 2 — Combat slice

(See DESIGN.md §8.)

- ⬜ Cannons + projectiles + reload time + cannon shot SFX hookup point.
- ⬜ Boarding melee: leave the ship, sword/pistol on enemy decks.
- ⬜ One dragon + one kraken tuned so the only low-level win is
  environmental (drown the fire dragon, lure the kraken to volcanic water).
- ⬜ Treasure pickup UI — interact prompt (E to grab) + inventory screen.
- ⬜ Phase 2 Done test: sink a hostile ship; kill a fire dragon by
  dragging it over the sea; recover loot.

### Phase 2 polish (parallel)
- ⬜ Water shader: foam, depth fog, fake refraction, surface ripple.
- ⬜ HDR + ACES/AgX tonemapping.
- ⬜ Volumetric fog (Standard/Nice tiers) — Bevy's built-in.
- ⬜ Baked AO maps on static hero assets.
- ⬜ First **real asset** swap: player ship hull (Tripo-generated, Blender-cleaned).

---

## ⬜ Phase 3 — RPG spine

- ⬜ XP / Levels (1–99) on actions, not just kills.
- ⬜ Race component bundle at character creation (Human / Merman / Orc /
  Beastfolk + sub-types).
- ⬜ One faction with reputation that responds to your actions.
- ⬜ Sol deity path — full vertical slice of the ascension trial mechanic.
- ⬜ Memory component on named NPCs (grudges).

## ⬜ Phase 4 — Sandbox layer

- ⬜ Remaining races + remaining deity paths (Moon / Sea / Void).
- ⬜ Bounty system + assassin spawns.
- ⬜ Nation stability tick + collapse rules.
- ⬜ Cross-session world persistence (raids stay raided).
- ⬜ Underwater biome — trench, pressure, decompression.

## ⬜ Phase 5 — Incarnation + endgame

- ⬜ Lv100 trials per deity.
- ⬜ Aura / physics-distortion effects on deity ascension.
- ⬜ NG+ on ascension.
- ⬜ Sky layer: hovering ships with gravity-fin sockets.

---

## 👤 Adi action items (only you can do these)

Pickup whenever — none are blocking.

- 👤 GitHub repo rename: Settings → General → rename `pien` to
  `unseeing-tides` (or whatever you like). GitHub auto-redirects.
  After rename, my GitHub MCP scope needs updating — flag if you see
  permission denials.
- 👤 (Optional) Sign up for Tripo AI paid tier when ready to generate
  the first hero asset (probably the player ship hull). Screenshot the
  commercial-rights clause on the day you generate.
- 👤 (Optional) Install Blender + Quad Remesher addon for cleaning AI
  output. Not urgent — can wait until Phase 2 polish.
- 👤 (When Phase 1 lands) Run the full Phase-1 done-test path: sail →
  storm → beach → save → quit → reload. Bug-report template in
  WORKFLOW.md.

---

## 🧊 Deferred — revisit later

- 🧊 Cargo workspace split (`vortexel` lib + `unseeing_tides` binary).
  Reason: premature; one crate is fine until Vortexel powers a second
  project.
- 🧊 Power-Diagram AoI for factions. Reason: cached overlap circles are
  cheaper and good enough.
- 🧊 Eldritch creature mesh. Reason: no fitting CC0 base; stay primitive
  until either an Itch CC0 emerges or Tripo+Blender produces one.
- 🧊 Multiplayer / networking. Reason: not in scope, ever.
- 🧊 Mobile / wasm builds. Reason: cleaned up; PC is the target.

---

## ⚠️ Open questions (decide before they bite)

- ⚠️ Modding hooks via RON beyond the planned ones — community-creator
  story. Decision deferred to after Phase 3.
- ⚠️ Procedural quests vs hand-authored beats — current plan: procedural
  default (treasure maps, bounties, faction missions); hand-author only
  the Cursed Survivor opening + four deity trials. Re-evaluate at Phase 3.
- ⚠️ Whether the Sol trial's "sun-glass arena" is a fixed handcrafted
  level or a procgen biome. Default: handcrafted, since it's lore-heavy.
