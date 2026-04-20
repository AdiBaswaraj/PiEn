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

Reference north stars:
- **Enshrouded** — **primary target** for environment *and* characters
  *and* bosses. Grounded proportions, weathered armor, PBR materials,
  serious faces. When in doubt, look at an Enshrouded screenshot.
- **Valheim** — perf floor. Potato-tier degradation lands here:
  flat-shaded, smaller textures, no detail normals. Still grounded.
- **Sea of Thieves** — *too stylized*. Avoid the face / eye language.
- **Skull and Bones** — *too realistic*. Avoid the material complexity
  (no cloth sim, no rope physics, no per-plank weathering).

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
- **Target fidelity: Enshrouded-tier.** Real 7–8 head proportions,
  weathered armor / cloth / leather with distinct PBR materials,
  serious or plain faces — no cartoon expressions.
- **Polycount:** ~5–10k tris for hero characters (player, named NPCs,
  bosses), ~2–3k for generic townsfolk / crew, with one LOD step down
  at distance. LOD0 is what close-ups see; LOD1 is what you see from
  the deck of your ship.
- **Texture set:** 2K PBR (`_diffuse`, `_normal`, `_orm`) for hero;
  1K for generic. Detail normals on metal + leather + coarse cloth.
- **Rig:** Mixamo humanoid base — decent geometry, good weight paint,
  retargets cleanly. Reskin / retopo in Blender if we need custom
  armor or race silhouettes (merman tail, orc mass).
- **Animations:** **idle, walk, run, sprint, attack, hit, death,
  wield-sword, aim-pistol, swim, climb-rope**. All free on Mixamo.
- **Race variants:** material tint + optional swap mesh (helmets,
  tails, horns), **not** new rigs. One skeleton, many silhouettes.
- **Potato-tier fallback:** the same mesh with flat-shaded material,
  1K textures, no detail normals. Still reads grounded, just plainer.

### Monsters & bosses (non-humanoid)
- **Target fidelity: Enshrouded-tier** for bosses (dragons, kraken,
  named apex creatures) — they carry the memorable fights, so they
  get the same polycount + PBR budget as hero characters (~8–12k tris,
  2K PBR). Regular wildlife / grunts stay cheaper (~2–4k tris, 1K).
- Fire / Ice / Sand Dragon → Itch.io CC0 dragon base, reskinned in
  Blender for per-element palette and weathering. Quaternius "Dragon
  Pack" is *too* cartoon for this tier — use only if nothing else.
- Kraken → CC0 octopus base, scaled up, tentacle sway in shader.
  Needs the beak and eye detail to land; reskin if the base is too
  toy-like.
- Sea Serpent / Giant Snake → serpent mesh with a swim clip; add
  weathered scale normal map in Blender.
- Wyvern → small-dragon / bat-wing base, reskinned.
- Undead → Quaternius skeleton rig (shape works; material repaint to
  weathered bone, no cartoon grin), reuse humanoid animations.
- Eldritch → Stretch. Stay on primitive until a fitting CC0 base
  emerges or we're ready to Blender one from the kraken base.

### Ships
- **Target fidelity: Enshrouded-tier** — grounded proportions, real
  1:1 scale (a galleon is ~40m long, not 8m), weathered wood planks
  via detail normal, tarnished metal fittings via ORM texture.
- **Polycount:** ~15–25k tris for a hero hull (player ship, named
  faction flagships), ~5–8k for generic NPC hulls. Rigging/ropes are
  baked into normal maps, not modeled geometry.
- Hull comes from curated Sketchfab / Itch.io CC0 pulls (galleons,
  brigantines, sloops). Kenney Pirate Kit hulls are **too small and
  too toy-shaped** for this tier.
- Sail + mast + wheel + flag + cannons — separate meshes, parented to
  hull. Sail animates via vertex shader wind deformation (cheap).
- Flag colours driven by faction component → material tint. One mesh,
  many factions. Same for sail stripe patterns.

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
- Target: grounded, weathered wood + stone, not cartoon kit-bashed
  blocks. Docks, crates, barrels, taverns, huts → curated Sketchfab /
  Itch.io CC0; Quaternius "Medieval" works for inland towns after a
  palette retint.
- Castle / tower stuff for faction capitals → Quaternius "Castle Kit"
  or equivalent; Kenney "Castle Kit" is placeholder-only (too toy).
- Ruins: reuse castle kit + scatter code + wear overlay in shader.

### Weapons & gear
- Cutlass, flintlock, musket, harpoon, cannon → curated CC0 pulls
  with real proportions (cannon is a *heavy* bronze tube, not a
  cartoon pipe). Kenney "Weapon Pack" is placeholder-only.
- Hats, coats, boots → Quaternius or pirate-specific Itch CC0,
  palette retinted to the muted world look.
- Parented to character hand / head / back sockets by name.

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
