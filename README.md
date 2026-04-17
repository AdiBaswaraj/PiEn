# PiEn — Pirate Engine

A small, performance-minded engine for an open-world pirate game focused on
**emergent systems over scripted events**. A massive ocean, sparse islands
with wildly different biomes, simulated weather, and monsters whose strengths
and weaknesses are described declaratively so the player can beat them with
the environment instead of a prescribed strategy.

## Why this stack

| Choice | Reason |
| --- | --- |
| **Rust** | Zero-cost abstractions + memory safety. Great for simulation code that has to run every frame. |
| **Bevy (ECS)** | Data-oriented ECS parallelises trivially across cores. Components make emergent monster/environment interactions almost free to add. Permissive licence. |
| **noise (Perlin/Simplex)** | Deterministic world generation from a seed. No asset pipeline needed for terrain. |
| **Flat / unlit rendering, no shadows** | You asked for low GPU overhead. Bevy's PBR is off by default in shadows; one directional light is effectively free. Upgrade later when the game is fun. |
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

## What's intentionally missing

- No art. The world is gizmos + a camera right now; you asked for low
  graphical priority, so the engine leaves rendering open for you to style
  later (flat shaded islands, stylised sprites, low-poly ships — all cheap).
- No networking. Single player, local save.
- No save/load yet. Components derive `Serialize` where it's sensible so
  adding `bevy_save` or a custom serde layer later is a weekend of work.
- No audio. Fog horns, kraken groans etc. belong in an audio plugin that
  subscribes to `WeatherEvent` / `CreatureKilled`.
