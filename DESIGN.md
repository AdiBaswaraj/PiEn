# PiEn — Design Doc

The canonical "what we're building" reference. This is intentionally short
and opinionated — it's what we point at when scope creep tries to sneak in.

## 1. Pitch

A single-player open-world pirate sandbox. The world is mostly ocean,
sparsely interrupted by wildly different islands. Player choice — what
race you start as, which deity (if any) you serve, who you trade with or
sink — drives the world's reaction. Gods, factions, and apex monsters
react to *what you actually do*, not to scripted triggers.

Combat, weather, and traversal all share one rule: **emergent over
scripted**. The engine knows physics + affinities + memory; quests and
"boss strategies" fall out of those, not out of bespoke code per fight.

## 2. Pillars

These are the four things every feature must serve. If a feature doesn't
serve one of them, it doesn't ship.

1. **Player choice has consequences the world remembers.** No reset
   buttons; raided villages stay ruined; betrayed factions stay angry.
2. **Environment is a weapon.** Fire dragons drown. Ice dragons melt.
   Krakens fight serpents. The player who reads the world wins.
3. **Multiple valid paths.** A player who never picks up a sword and only
   trades should reach an ending. So should a player who burns the world.
4. **Scales from potato to nice.** 60 FPS at 1080p on integrated
   graphics is the floor; richer hardware unlocks volumetric fog,
   soft shadows, longer draw distance, screen-space reflections.
   Art direction is locked to **Valheim-grounded low-poly** with
   **Enshrouded-level shader / post polish** — polish lives in the
   shader / post stack, never in geometry or texture bloat.

## 3. World Structure

- **Massive ocean** with chunked, deterministic island generation
  (heightfield, not voxel — voxels only buy us destructible terrain we
  don't need).
- **Twelve+ biomes**, each with distinct creatures, weather bias, loot
  rarity, and visual palette.
- **Underwater layer** (Phase 4): trenches, reefs, sunken cities. Pressure
  + decompression are real damage sources.
- **Sky layer** (Phase 5): hovering ships use thrust vectoring, fuel
  consumption is a function of mass × engine efficiency.

## 4. Player Paths

### Races (`RaceComponent` bundle of trait components)

| Race      | Trait components                                |
|-----------|-------------------------------------------------|
| Human     | `Diplomat(+10% rep gain)`                       |
| Merman    | `Gills`, `Dehydration` on land                  |
| Orc       | `HighInertia(stagger ×1.5)`, `BigAppetite`      |
| Beastfolk | sub-types: `LowFallDamage`, `Nightvision`, etc. |

Race is a soft-lock at character creation; it modifies physics, not story.

### Deities (post-Level-99 ascension paths)

| Deity            | Philosophy           | Trial                          |
|------------------|----------------------|--------------------------------|
| Sol Sovereign    | Order, sunlight      | 1v1 in a sun-glass arena       |
| Moon Matriarch   | Stealth, tides       | Fight a shadow clone of self   |
| Sea Father       | Chaos, power         | Survive a 10-min maelstrom     |
| Void Entity      | Entropy, mutation    | Build a path through silence   |

Triggered by *gameplay history*, not by NPC dialogue. The Sol path opens
itself if you hit Lv99 with high Sol-faction rep + clean record. The Void
path opens if you hit Lv99 with negative sanity and 5+ Eldritch relics
consumed.

### Cursed Survivor (the starter narrative)

A specific opening situation, not the whole game: slave on a sinking
ship → rescued by a merchant → first map → first island. Once you're at
sea, the sandbox takes over.

## 5. Core Systems → Engine Plugins

| Concept                          | Plugin / module                            |
|----------------------------------|--------------------------------------------|
| Buoyancy, hovering, drag         | `physics`                                  |
| Currents (vector field)          | `physics` (`Current` resource)             |
| Pressure / implosion / bends     | `physics` + `combat::depth_damage`         |
| Weather, fog, storms, freeze     | `weather`                                  |
| Race / faction / deity traits    | `creatures` (extended), `factions` (new)   |
| Memory + grudges                 | `creatures::brain` + `MemoryComponent`     |
| AoI / territory                  | `factions` (Voronoi-ish, cached)           |
| XP & Levels                      | `progression` (new)                        |
| Modular ships (sockets)          | `player::ship` + child entities            |
| Bounties                         | `factions::bounty`                         |
| Nation stability tick (0.1 Hz)   | `factions::nation` (FixedUpdate)           |
| Save / load                      | `persistence` (new, serde)                 |
| HUD                              | `ui` (new, `bevy_ui`)                      |
| Rendering                        | `render` (extended; instancing for flora)  |

## 6. Cuts & Softens

Things in the brainstorm we are *not* doing, or doing differently:

- **No 144 Hz physics.** Fix at 60 Hz; AI at 20 Hz; world sim at 0.1 Hz.
  Anything faster kills low-end hardware.
- **No sparse voxel chunks.** Heightfield + biome grid is cheaper and
  sufficient for an ocean world.
- **Power-Diagram AoI is a stretch goal.** Ship with cached overlap
  circles; only escalate if a profile demands it.
- **Incarnation is a choice with a cost, not an identity overwrite.**
  Players keep their crew. Aura effects are real but optional to use.
  The "mass metamorphosis of crew" idea pushes the game toward "no good
  ending" and locks out role-play.
- **Slavery exists in the world's *backstory* (the opening) but is not a
  gameplay system.** No slave-trading mechanic.
- **No multiplayer / networking.** Single player, local save.

## 7. Performance Rules & Graphics Tiers

The rules below describe the **Potato tier** — the non-negotiable floor
every PR is reviewed against. Standard and Nice tiers add polish on top;
they never weaken the floor.

### 7a. Potato tier (floor — must always work)

1. Flat-shaded, unlit where the look allows; one directional light.
2. No real-time shadows in the default config (baked AO only).
3. One mesh per chunk; instancing for vegetation/decor.
4. View radius capped — streaming despawns chunks aggressively.
5. AI thinks at 20 Hz, not per frame.
6. World "heartbeat" (faction sim, off-screen events) at 0.1 Hz.
7. Particle counts capped per cell; degrade silently on slow frames.
8. Audio mixed in fewest channels possible.
9. Textures 512–1024 for decor, 1024 for hero, 2K only for the player
   ship hull / player character / named NPCs.

### 7b. Standard tier (most laptops, ~mid-range GPUs)

Enables on top of Potato:
- Low-res (1024²) cascaded shadow maps, one cascade.
- Half-resolution volumetric fog.
- Water shader: foam, depth fog, fake refraction.
- Detail normals on wood / metal / rock materials.
- Longer view radius (+1 chunk).
- HDR + ACES / AgX tonemapping.

### 7c. Nice tier (modern mid-range+)

Enables on top of Standard:
- Full-res volumetric fog.
- Screen-space reflections on water.
- Higher shadow res, two cascades.
- Longer view radius again (+2 chunks total vs Potato).
- Raised particle caps.

No tier is allowed to lift geometry detail or texture resolution past
the caps in §7a — polish is shader / post only.

## 8. Phase Plan

Each phase ends in something that is *playable*, not just buildable. We
don't start phase N+1 until phase N is fun.

### Phase 1 — Survival slice
- Sailing (already in), wind on sail, currents.
- Hull damage from collisions, storms, ice fronts.
- Captain stats already wired — surface them in HUD.
- Save / load to local file.
- Visible weather (rain, fog tint, freeze halo).
- Initial spawn lands the player in open water near an island.
- **Done when:** you can sail across a few biomes, get caught in a storm,
  beach on an island, save, quit, reload, and continue.

### Phase 2 — Combat slice
- Cannons + projectiles + reload time.
- Boarding: melee combat on enemy decks.
- One dragon + one kraken tuned so the *only* way to win at low level is
  environmental (drown the dragon, lure kraken to volcanic water).
- Treasure pickup UI (we already drop the loot — needs the "press E"
  loop and an inventory screen).
- **Done when:** you can sink a hostile ship, kill a dragon by dragging
  it over the sea, and recover loot.

### Phase 3 — RPG spine
- XP / Levels (1–99) tied to actions, not just kills.
- Race component bundle at character creation.
- One faction with reputation that responds to your actions.
- One deity path implemented end-to-end (Sol — easiest to model).
- Memory component on NPCs (named NPCs remember slights).
- **Done when:** you can pick a race, anger one faction, level up to 99
  by playing as either a trader or warrior, and trigger the Sol trial.

### Phase 4 — Sandbox layer
- Remaining races and deity paths.
- Bounty system + assassin spawns.
- Nation stability tick + collapse rules.
- Persistence of village/world state across sessions.
- Underwater biome (trench, pressure, decompression).
- **Done when:** the world reacts to a 50-hour playthrough — burned
  villages stay burned, blockaded routes collapse trade, deity factions
  hunt rivals.

### Phase 5 — Incarnation + endgame
- Level 100 trials per deity.
- Aura / Physics-distortion effects.
- NG+ on ascension.
- Sky layer: hovering ships (gravity-fin sockets).
- **Done when:** all four deity endings are reachable and replayable
  from one save.

### Polish & assets (parallel from Phase 2 onward)
- Swap placeholder cuboids for free CC0 low-poly models.
- Audio: ambient ocean, weather, combat hits, monster cries.
- UI pass: readable HUD, menus, settings.
- Balance pass per phase.

## 9. Open Questions (decide before they bite us)

- **Saving on web?** If we keep wasm builds for later demos, save needs
  IndexedDB instead of files. Decision deferred until we know wasm is
  staying.
- **Modding hooks?** `serde`-driven data files for biomes/creatures
  would make community content trivial. Decision: yes, after Phase 3.
- **Procedural quests vs hand-authored beats?** Default to procedural
  (treasure maps, bounties, faction missions). Hand-author only the
  Cursed Survivor opening and the four deity trials.
- ~~**Art direction?**~~ **Resolved** (2026-04): Valheim-grounded
  low-poly geometry + Enshrouded-level shader / post polish, behind a
  Potato / Standard / Nice graphics-tier slider. See §7 and
  `ASSETS.md` for the detailed style guardrails and source list.
