# The Unseeing Tides

An open-world pirate sandbox where gods, factions and apex monsters react
to *what you actually do*. Sail a massive ocean, beach on wildly different
islands, anger a faction, drown a fire dragon, and watch the world remember.

It's built on **Vortexel** — the engine layer in this repo: a small,
performance-minded ECS substrate (Bevy + Rust) focused on **emergent
systems over scripted events**. Vortexel knows physics, weather,
affinities and memory; quests and "boss strategies" fall out of those, not
out of bespoke code per fight.

For the full design pillars and phase plan, see `DESIGN.md`. For the art /
audio / content pipeline, see `ASSETS.md`. For the day-to-day workflow,
see `WORKFLOW.md`.

## Why this stack

| Choice | Reason |
| --- | --- |
| **Rust** | Zero-cost abstractions + memory safety. Great for simulation code that has to run every frame. |
| **Bevy (ECS)** | Data-oriented ECS parallelises trivially across cores. Components make emergent monster/environment interactions almost free to add. Permissive licence. |
| **noise (Perlin/Simplex)** | Deterministic world generation from a seed. No asset pipeline needed for terrain. |
| **Tiered rendering (Potato / Standard / Nice)** | Geometry budget stays cheap (Valheim-grounded low-poly); polish lives in the shader / post stack and scales with hardware. Potato = flat-shaded, no shadows; Nice = volumetric fog + SSR. See `DESIGN.md` §7. |
| **Cell-based weather** | Weather samples live on a coarse grid, not per-tile. Cost stays flat no matter how big the map grows. |
| **Chunk streaming** | Only chunks around the player are resident in the ECS. The "massive map" is bounded only by `f32` precision, not RAM. |

## Architecture

```
main.rs
├── world        // chunked noise-driven map + biomes
├── weather      // rain / fog / heat / freeze / storms / lightning
├── physics      // buoyancy, waves, wind, environment tagging
├── creatures    // krakens, dragons, undead, eldritch + utility AI
├── combat       // damage events, affinities, environmental kills
├── treasure     // buried caches, loot drops, maps, relics
└── player       // ship + captain + input + camera + inventory
```

Every subsystem is a Bevy `Plugin`. Systems communicate through components
(shared state) and events (transient messages). There is no manager singleton
and no per-monster script.

## The "environmental kill" rule

This is the heart of the design and the thing the user asked for. It's one
rule, in `combat::environmental_damage`:

> If a creature is `submerged` and cannot breathe water, apply
> `DamageKind::Water` each frame.

A fire dragon has `Affinity { water: 3.0, air_breather: true }`. So if the
player can force it into the sea (by baiting, by destroying the ledge it's
on, by a lightning-induced storm surge) the world itself kills it — no boss
script, no "drown fire dragon" state machine.

The same template applies everywhere:

- **Fire dragon vs ice dragon** fights are just `are_enemies` returning true
  and each one being vulnerable to the other's element.
- **Drag a kraken to volcanic waters** → kraken's `fire` affinity is 0.5, so
  heat haze damage + lava-spray from environment ticks faster than in the
  deep ocean. Players find out by experimenting.
- **Haunted marsh is terrifying** because fog drains sanity (player) and
  undead are immune to physical damage by default (physical affinity not
  boosted), so you must use fire or holy — holy being a loot-driven relic.

## Massive map, cheap

- World is `WorldSampler` — a pure function from `(x, z, seed)` to biome.
- Chunks are generated on demand, despawned when the player leaves them.
- Creatures spawn when their chunk loads, despawn with it (or wander out).
- Weather is a coarse grid that advects with noise + prevailing wind.
- No navmesh: creature AI uses direct steering because it's cheap and
  "good enough" for open-water/open-land chases.

## Run it

```sh
cargo run --release
```

Controls:

| Key | Action |
| --- | --- |
| W / S | Throttle forward / back |
| A / D | Steer |
| Space | Hoist sail |
| Shift | Drop sail |

The release build turns on LTO and a single codegen unit; Bevy dependencies
are built with `opt-level = 3` even in dev so physics/AI feel snappy while
iterating.

## Extending

- **New monster** — add a `Species` variant, add an `archetype(...)` arm.
  Everything else (AI, combat, loot drops that still need a match arm for
  reagents) picks it up automatically.
- **New biome** — add a `Biome` variant, a case in `WorldSampler::biome_at`,
  entries in `monster_bias()` and `base_temperature()`. Rendering / weather
  / treasure read from these centrally.
- **New damage type** — add a `DamageKind` variant, a field on `Affinity`,
  and a match arm in `apply_damage`. That's it.
- **New environmental kill** — add a rule in `combat::environmental_damage`
  reading whatever `Environment` / `Creature` / `Affinity` combo you want.

## What's intentionally missing (yet)

- **Final art.** Currently primitives + per-chunk vertex-coloured terrain.
  Asset swap target is **Enshrouded-tier grounded low-poly** with **Valheim
  as the perf floor** — see `ASSETS.md` for sources, naming, style
  guardrails, and the Tripo AI → Blender hero-asset workflow.
- **Networking.** Single player, local save. No multiplayer planned.
- **Save / load.** Components derive `Serialize` where sensible; landing
  the actual save layer is a Phase 1 task.
- **Audio.** Fog horns, kraken groans, cannon thunder — they'll subscribe
  to `WeatherEvent` / `CreatureKilled` from a future audio plugin.
