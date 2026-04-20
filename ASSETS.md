# ASSETS.md — the art / audio / content pipeline

The asset library + pipeline for PiEn. Read alongside `DESIGN.md` (scope)
and `WORKFLOW.md` (process). The goal: every asset type has one sanctioned
source, one file format, and one place in the repo. No ambiguity.

## 1. Principles

1. **CC0 or permissive only.** If we can't ship it in a binary without
   paying royalties, we don't use it. See §7 for the licence ledger.
2. **Placeholder first, swap later.** Every system works with primitives
   before a single asset is added. The game is playable even if the art
   never arrives.
3. **Valheim-grounded low-poly, Enshrouded-level shader polish.**
   Geometry stays cheap (low polycount, hard edges, no subdivision).
   Visual quality comes from the shader / post stack: tonemapping,
   volumetric fog, baked AO, detail normals, a proper water shader.
   **Grounded proportions, weathered palette, serious tone.** If a
   pack looks like a kids' mobile game (chibi bodies, googly eyes,
   saturated primary colors), it's the wrong pack.
4. **Everything GLB / PNG / OGG.** No FBX, no WAV, no TTF over OTF.
   One format per slot means I never write two importers.
5. **Content is data, not code.** Abilities, creatures, biomes, loot
   tables live in `.ron` files. "Add a new spell" is editing text, not
   writing Rust.

### Style guardrails — how we pass/fail a pack

Use this checklist before adding anything. If a source fails two of
these, reject it; if it fails one, reskin it in Blender before use.

| Axis | Pass | Fail |
|---|---|---|
| Proportions | 7–8 head-heights for humans; real-world scale for ships | 3–5 head-heights; chibi bodies; toy-scale ships |
| Silhouette | Weathered, asymmetric, purposeful detail | Clean / symmetric / "new toy" feel |
| Palette | Desaturated, muted, with strong neutrals | Saturated primaries; pastel; rainbow |
| Face | None, stylized plain, or bearded/grim | Big eyes; round cheeks; smiling |
| Surface | Flat or one detail-normal pass; baked AO ok | Smooth-shaded; glossy plastic; no wear |
| Materials | PBR-ready (diffuse / normal / ORM) | Baked vertex color only, no room to re-light |

Reference north stars: **Valheim** (floor — we must at least match),
**Enshrouded** (ceiling — where the Nice tier should land),
**Sea of Thieves** (too stylized — avoid the face/eye language),
**Skull and Bones** (too realistic — avoid the material complexity).

## 2. Source rolodex

| Category | Primary source | Licence | Notes |
|---|---|---|---|
| Hero models (ships, player, named NPCs, dragons) | **Curated Sketchfab + Itch.io CC0** | CC0 | First stop for anything that has to read "grounded". Filter: CC0 + downloadable; pick individually, not pack-wholesale. Reskin in Blender if close-but-not-right. |
| Decor / kit models (rocks, trees, crates, buildings) | **Quaternius.com** | CC0 | Use the grounded packs (Nature, Medieval, Castle); skip the cutesy ones. Re-tint palette in engine to match world. |
| Humanoid rigs + ~2500 animations | **Mixamo** (Adobe) | Free for commercial use | Upload a rig, auto-retarget any animation. |
| Weird / bespoke monsters | **Itch.io** (filter: free, CC0) | Varies — check per pack | Krakens, serpents, eldritch shapes. Check licence per pack, always. |
| UI frames, particle textures, misc throwaway props | **Kenney.nl** | CC0 | Demoted from "first stop" to **UI + particles + filler props only** — too cartoon for hulls, characters, or hero assets. Kenney Particle Pack is still the go-to for smoke / sparks / embers. |
| UI icons (spells, items, status) | **game-icons.net** | CC BY 3.0 (attribution only) | 4000+ icons. Recolor in engine. |
| Fonts | **Google Fonts** | OFL / free | "IM Fell English", "Pirata One" for theme. |
| SFX (ocean, cannons, monsters) | **Freesound.org** (filter: CC0) | CC0 | Individual clips; we layer in engine. |
| Music | **FreePD** / **OpenGameArt** / **Itch.io** | Public domain or CC0 | One ambient loop per biome, one combat theme. |
| Self-made / reskinned hero assets | **Blender** (your machine) | Your own / derived from a CC0 base | Flagship ship, player body, named NPCs, deity relics. Reskin CC0 bases first; full from-scratch is a hobby stretch goal. |

If you ever find a pack somewhere else, check the licence *before* you
send it my way. If it says "non-commercial", "no redistribution", or has
no licence at all — we can't use it.

## 3. Per-category plan

### Characters & NPCs
- Rig: Mixamo humanoid (any of their default bases) OR Quaternius humanoid.
- Animations we need: **idle, walk, run, attack, hit, death, wield-sword,
  aim-pistol, swim**. All free on Mixamo.
- Race variants handled by material tint + optional swap mesh, not by new rigs.

### Monsters (non-humanoid)
- Fire / Ice / Sand Dragon → Quaternius "Dragon Pack" or Itch.io CC0 dragon.
- Kraken → CC0 octopus, scaled up. Tentacle sway handled in shader.
- Sea Serpent / Giant Snake → Same source; serpent meshes with a swim clip.
- Wyvern → Any small-dragon / bat-wing model.
- Undead → Quaternius skeleton rig, reuse humanoid animations.
- Eldritch → Stretch. Stay on primitive until a pack emerges.

### Ships
- Hull (rigid body) — Kenney Pirate Kit has small, medium, large hulls.
- Sail + mast + wheel + flag + cannons — separate meshes, parented to hull.
- Flag colours driven by faction component → material tint. One mesh, many factions.

### Biomes / terrain
- Terrain itself is generated in code (done). We don't asset this.
- Vegetation: Quaternius "Nature Pack" (trees, rocks, bushes) — per biome:

  | Biome | Typical props |
  |---|---|
  | Jungle | Palm trees, ferns, mossy rocks |
  | Desert | Cacti, sand rocks, bleached wood |
  | Tundra / IceCap | Pine / dead trees, ice shards |
  | Volcanic | Dead trees, obsidian rocks, magma vents |
  | Haunted marsh | Twisted trees, gravestones, fog volume |
  | Coral reef (underwater) | Coral clusters, kelp, shell piles |

### Buildings / props
- Kenney "Pirate Kit" covers docks, crates, barrels, taverns, huts.
- Castle / tower stuff for faction capitals: Kenney "Castle Kit".
- Ruins: reuse castle kit + scatter code.

### Weapons & gear
- Cutlass, flintlock, musket, harpoon, cannon → Kenney "Weapon Pack".
- Hats, coats, boots → Quaternius or pirate-specific Itch packs.
- Parented to character hand / head bones by name.

### UI
- Frame / panels / buttons → Kenney UI Pack (pick the parchment one).
- Icons → game-icons.net. We'll need ~100 icons (spells, items, statuses).
- Fonts → two: one decorative (titles) + one readable (body).
- Cursor / HUD bars → Kenney UI.

### VFX
- No asset — particles and shaders are code.
- Exception: particle textures (single 64×64 PNGs) from Kenney Particle Pack.

### Audio
- **SFX:** one clip per event (cannon fire, wood crack, sail flap, dragon
  roar, coin pickup). Aim for ~50 clips total by end of Phase 2.
- **Ambient:** loops per biome (ocean, jungle, storm, silence-with-wind).
  6–10 clips.
- **Music:** sparse. One title theme, one per deity, one "danger" sting,
  one victory sting. That's enough to feel composed without being
  repetitive.

## 4. Repo layout

```
assets/
├── models/
│   ├── characters/        *.glb — player + npcs
│   ├── creatures/         *.glb — monsters
│   ├── ships/             *.glb — hulls + parts
│   ├── props/             *.glb — crates, flags, rocks, etc.
│   └── buildings/         *.glb — kit pieces
├── textures/
│   ├── biomes/            terrain tint / detail textures
│   ├── particles/         64×64 PNGs for VFX
│   ├── ui/                panels, frames, cursors
│   └── icons/             ability / item icons
├── audio/
│   ├── sfx/               short one-shots
│   ├── ambient/           biome loops
│   └── music/             themes, stings
├── fonts/                 *.ttf / *.otf
└── data/
    ├── abilities/         *.ron — spell / ability definitions
    ├── creatures/         *.ron — stat / affinity overrides per species
    ├── biomes/            *.ron — biome tuning (temp, spawn bias, loot)
    ├── factions/          *.ron — deity / nation defs
    ├── loot/              *.ron — loot tables per chest rarity / biome
    └── dialogue/          *.ron — "barks" (short lines); no full dialogue trees
```

## 5. Naming conventions

- **Snake case everywhere.** `fire_dragon.glb`, not `FireDragon.glb`.
- **Category prefix for ambiguous names.** `ship_hull_large.glb`, not
  `large.glb`.
- **Kit-style packs keep their short names.** `crate_wooden_small.glb`.
- **Textures: 1K–2K PBR set** with `_diffuse`, `_normal`, `_orm`
  suffixes (ORM = occlusion / roughness / metallic packed into one
  texture). 2K only for hero assets (player ship, player character,
  named NPCs); 1K for decor; 512 for far-LOD or small props. No 4K —
  bloats build size and kills the Potato tier.
- **Audio: include an approximate duration in the name for loops** —
  `ambient_jungle_30s.ogg` — so we don't accidentally loop a 0.5 s clip.

## 6. Data-driven content

Content files are [RON](https://docs.rs/ron) — serde-friendly,
comment-supporting, diff-friendly. Example — `data/abilities/ice_dart.ron`:

```ron
Ability(
    id: "ice_dart",
    display_name: "Ice Dart",
    damage: (kind: Ice, amount: 25.0),
    area: Line(range: 30.0),
    cost: (stamina: 5.0, sanity: 0.0),
    cooldown: 2.0,
    cast_time: 0.4,
    vfx: "particles/ice_shard",
    sfx: "sfx/spell_ice_hit",
    icon: "icons/spells/ice_dart",
    requires: Deity(MoonMatriarch),
)
```

Adding a new ability is: drop a file, reload. No recompile.
Same pattern for creatures (override base stats), loot tables, biome
tuning, faction definitions.

## 7. Licence ledger

`NOTICE.md` at the repo root tracks **every pack we pull in**:

```
## Kenney Pirate Kit
- Source: https://kenney.nl/assets/pirate-kit
- Licence: CC0 (public domain)
- Used for: ship hulls, crates, barrels, dock props
```

Two rules:

1. **Pack must be CC0 or clearly permissive** (CC BY with attribution is
   OK, CC BY-SA is **not** — it would force the game to be the same
   licence).
2. **One entry per pack in `NOTICE.md` before the file is committed.** If
   it's not in the ledger, it's not in the game.

## 8. Priority order — what to grab when

We grab nothing right now. We sail primitive cuboids through Phase 1.
From Phase 2 on, in roughly this order:

| Phase | First asset priorities |
|---|---|
| 2 — Combat | Ship hull + sail + cannon; one dragon; cannon SFX; parchment UI frame; decorative font. |
| 2 (later) | Humanoid player rig with idle/walk/run/attack; cutlass + pistol. |
| 3 — RPG spine | Faction flags + tabards; tavern kit for towns; ~30 item icons; one ambient music loop per occupied biome. |
| 4 — Sandbox | Rest of races (merman tail, orc body variants); remaining dragon elementals; underwater kit (coral, kelp, sunken ship). |
| 5 — Incarnation | Deity trial arena props (sun temple, moon grotto, whirlpool, void rift); aura VFX. |

## 9. Things we explicitly are *not* doing

- **From-scratch custom modelling in Blender is a stretch hobby.**
  Reskinning / retopo-ing / repainting CC0 bases in Blender *is* on
  the table for hero assets (flagship ship, player body, named NPCs,
  deity relics) — faster to learn, every hour yields a shippable
  asset. Don't start modelling a new galleon from a cube.
- Commissioning art. Too expensive for a hobby project.
- AI-generated images for anything shipped. Licence story is murky;
  avoids the CC0 guarantee we want.
- Mixed art styles. Once the first pack is picked, everything else has
  to fit *that* style or we swap it. No "realistic dragon with cartoon
  pirate".
- Motion capture / voice acting. Bark lines are text only.

## 10. When you're ready to pick the first pack

Ping me with:

1. Which pack URL you like (probably Kenney Pirate Kit).
2. The first model to swap (probably the ship hull).

I'll wire it up, push, you'll see a proper ship at spawn. That's the
hook — after that, each pack-swap is the same five-minute loop.
