//! World generation: a massive, mostly-ocean map punctuated by islands with
//! distinct biomes. The map is infinite-ish by being chunked and deterministic.
//!
//! Design notes:
//! * The overworld is a 2D heightfield sampled from layered Perlin/Simplex
//!   noise. Land vs sea is a simple threshold on the height value.
//! * Biome is picked from (latitude, temperature, humidity, elevation) —
//!   cheap enough to run per-chunk on the CPU.
//! * Only chunks near the player are streamed in; everything else is
//!   discarded from the ECS so GPU/CPU overhead stays bounded regardless of
//!   world size.

use bevy::prelude::*;
use bevy::utils::HashMap;
use noise::{NoiseFn, Perlin};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WorldSeed(0xC0FFEE_BEEF))
            .insert_resource(WorldGen::default())
            .insert_resource(LoadedChunks::default())
            .add_event::<ChunkLoaded>()
            .add_event::<ChunkUnloaded>()
            .add_systems(Startup, spawn_initial_region)
            .add_systems(Update, (stream_chunks_around_player,));
    }
}

/// Deterministic seed. Everything world-related branches off this.
#[derive(Resource, Clone, Copy)]
pub struct WorldSeed(pub u64);

/// Tunables for world generation. Exposed as a resource so a UI or console
/// can tweak them at runtime for debugging.
#[derive(Resource, Clone)]
pub struct WorldGen {
    /// Height below which we consider the cell to be ocean.
    pub sea_level: f32,
    /// Tiles per chunk side.
    pub chunk_size: i32,
    /// World units per tile.
    pub tile_size: f32,
    /// Streaming radius in chunks.
    pub view_radius: i32,
    /// Frequency of the base continent noise — lower = bigger landmasses.
    pub continent_frequency: f64,
}

impl Default for WorldGen {
    fn default() -> Self {
        Self {
            sea_level: 0.05,
            chunk_size: 64,
            tile_size: 2.0,
            view_radius: 3,
            continent_frequency: 0.002,
        }
    }
}

/// A 2D chunk coordinate (in chunk-space, not world-space).
#[derive(Component, Clone, Copy, Eq, PartialEq, Hash, Debug)]
pub struct ChunkCoord {
    pub x: i32,
    pub y: i32,
}

/// Biomes drive rendering, creature spawning, weather biases and loot tables.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Biome {
    DeepOcean,
    ShallowSea,
    Beach,
    Jungle,
    Desert,
    Savanna,
    Tundra,
    IceCap,
    VolcanicAsh,
    /// Perma-fogged, cursed island. Undead + eldritch spawners prefer these.
    HauntedMarsh,
    CoralReef,
    MangroveSwamp,
}

impl Biome {
    /// Base ambient temperature in Celsius for the biome. Weather + day phase
    /// modulate this further.
    pub fn base_temperature(&self) -> f32 {
        match self {
            Biome::DeepOcean => 10.0,
            Biome::ShallowSea => 18.0,
            Biome::Beach => 25.0,
            Biome::Jungle => 28.0,
            Biome::Desert => 40.0,
            Biome::Savanna => 32.0,
            Biome::Tundra => -5.0,
            Biome::IceCap => -25.0,
            Biome::VolcanicAsh => 55.0,
            Biome::HauntedMarsh => 12.0,
            Biome::CoralReef => 24.0,
            Biome::MangroveSwamp => 26.0,
        }
    }

    /// How likely monsters of a given flavour are to spawn here.
    pub fn monster_bias(&self) -> MonsterBias {
        use Biome::*;
        match self {
            DeepOcean => MonsterBias { kraken: 0.9, sea_serpent: 0.6, ..MonsterBias::zero() },
            ShallowSea => MonsterBias { sea_serpent: 0.3, ..MonsterBias::zero() },
            CoralReef => MonsterBias { sea_serpent: 0.5, ..MonsterBias::zero() },
            Jungle => MonsterBias { giant_snake: 0.6, wyvern: 0.2, ..MonsterBias::zero() },
            Desert => MonsterBias { sand_dragon: 0.7, ..MonsterBias::zero() },
            VolcanicAsh => MonsterBias { fire_dragon: 0.9, ..MonsterBias::zero() },
            Tundra | IceCap => MonsterBias { ice_dragon: 0.6, ..MonsterBias::zero() },
            HauntedMarsh => MonsterBias {
                undead: 0.9,
                eldritch: 0.4,
                ..MonsterBias::zero()
            },
            MangroveSwamp => MonsterBias { undead: 0.3, giant_snake: 0.3, ..MonsterBias::zero() },
            _ => MonsterBias::zero(),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MonsterBias {
    pub kraken: f32,
    pub sea_serpent: f32,
    pub fire_dragon: f32,
    pub ice_dragon: f32,
    pub sand_dragon: f32,
    pub wyvern: f32,
    pub giant_snake: f32,
    pub undead: f32,
    pub eldritch: f32,
}

impl MonsterBias {
    pub const fn zero() -> Self {
        Self {
            kraken: 0.0,
            sea_serpent: 0.0,
            fire_dragon: 0.0,
            ice_dragon: 0.0,
            sand_dragon: 0.0,
            wyvern: 0.0,
            giant_snake: 0.0,
            undead: 0.0,
            eldritch: 0.0,
        }
    }
}

/// Registry of chunks currently resident in the ECS.
#[derive(Resource, Default)]
pub struct LoadedChunks {
    pub map: HashMap<(i32, i32), Entity>,
}

#[derive(Event)]
pub struct ChunkLoaded(pub ChunkCoord);

#[derive(Event)]
pub struct ChunkUnloaded(pub ChunkCoord);

/// Sampler that turns world coordinates into (elevation, moisture, temperature).
/// Deterministic given the same seed — so every client regenerates the same
/// world from nothing.
pub struct WorldSampler {
    elevation: Perlin,
    moisture: Perlin,
    warp: Perlin,
}

impl WorldSampler {
    pub fn new(seed: u64) -> Self {
        Self {
            elevation: Perlin::new(seed as u32),
            moisture: Perlin::new(seed.wrapping_add(1) as u32),
            warp: Perlin::new(seed.wrapping_add(2) as u32),
        }
    }

    /// Returns elevation in roughly [-1, 1]. Values above sea_level are land.
    pub fn elevation(&self, x: f64, y: f64, freq: f64) -> f32 {
        // Domain warping gives islands less "blobby" shapes.
        let wx = self.warp.get([x * freq * 2.0, y * freq * 2.0]) * 40.0;
        let wy = self.warp.get([x * freq * 2.0 + 100.0, y * freq * 2.0 + 100.0]) * 40.0;
        let base = self.elevation.get([(x + wx) * freq, (y + wy) * freq]);
        let detail = self.elevation.get([x * freq * 8.0, y * freq * 8.0]) * 0.15;
        (base + detail) as f32
    }

    pub fn moisture(&self, x: f64, y: f64, freq: f64) -> f32 {
        self.moisture.get([x * freq * 1.5, y * freq * 1.5]) as f32
    }

    /// Latitude-driven temperature band. Poles are cold, equator is hot.
    pub fn latitude_temp(&self, y: f64, world_span: f64) -> f32 {
        let lat = (y / world_span).clamp(-1.0, 1.0);
        30.0 - 55.0 * lat.abs() as f32
    }

    pub fn biome_at(&self, x: f64, y: f64, gen: &WorldGen) -> Biome {
        let h = self.elevation(x, y, gen.continent_frequency);
        if h < gen.sea_level - 0.25 {
            return Biome::DeepOcean;
        }
        if h < gen.sea_level - 0.05 {
            return Biome::ShallowSea;
        }
        if h < gen.sea_level {
            // Occasional reef ring around islands.
            let reef = self.moisture.get([x * 0.01, y * 0.01]);
            return if reef > 0.4 { Biome::CoralReef } else { Biome::ShallowSea };
        }
        if h < gen.sea_level + 0.02 {
            return Biome::Beach;
        }

        let temp = self.latitude_temp(y, 4000.0);
        let moist = self.moisture(x, y, gen.continent_frequency);
        let cursed = self.warp.get([x * 0.003, y * 0.003]);

        // Rare haunted islands — always foggy, always hostile.
        if cursed > 0.55 && h > gen.sea_level + 0.1 {
            return Biome::HauntedMarsh;
        }

        if h > 0.55 && temp > 15.0 {
            return Biome::VolcanicAsh;
        }
        if temp < -10.0 {
            return Biome::IceCap;
        }
        if temp < 5.0 {
            return Biome::Tundra;
        }
        if moist < -0.25 && temp > 20.0 {
            return Biome::Desert;
        }
        if moist < 0.1 && temp > 15.0 {
            return Biome::Savanna;
        }
        if moist > 0.3 && temp > 20.0 && h < gen.sea_level + 0.1 {
            return Biome::MangroveSwamp;
        }
        Biome::Jungle
    }
}

/// Marker component for a generated chunk entity.
#[derive(Component)]
pub struct Chunk {
    pub coord: ChunkCoord,
    pub dominant_biome: Biome,
}

/// A single sampled tile inside a chunk. In the MVP we do not give each tile
/// its own ECS entity — that would explode memory. Instead the chunk owns a
/// packed grid and creatures/weather query it by world position.
#[derive(Component)]
pub struct ChunkTiles {
    pub size: i32,
    pub tile_size: f32,
    pub biomes: Vec<Biome>,
    pub elevation: Vec<f32>,
}

impl ChunkTiles {
    pub fn index(&self, lx: i32, ly: i32) -> usize {
        (ly * self.size + lx) as usize
    }
}

fn spawn_initial_region(
    mut commands: Commands,
    seed: Res<WorldSeed>,
    gen: Res<WorldGen>,
    mut loaded: ResMut<LoadedChunks>,
    mut events: EventWriter<ChunkLoaded>,
) {
    let sampler = WorldSampler::new(seed.0);
    for cy in -gen.view_radius..=gen.view_radius {
        for cx in -gen.view_radius..=gen.view_radius {
            load_chunk(
                &mut commands,
                &sampler,
                &gen,
                &mut loaded,
                &mut events,
                ChunkCoord { x: cx, y: cy },
            );
        }
    }
}

/// Streams chunks in/out based on the player's position. This is the big
/// lever for keeping CPU/GPU bounded on a "massive" map.
fn stream_chunks_around_player(
    mut commands: Commands,
    seed: Res<WorldSeed>,
    gen: Res<WorldGen>,
    mut loaded: ResMut<LoadedChunks>,
    mut load_ev: EventWriter<ChunkLoaded>,
    mut unload_ev: EventWriter<ChunkUnloaded>,
    player: Query<&Transform, With<crate::player::Player>>,
    chunks: Query<(Entity, &Chunk)>,
) {
    let Ok(ptr) = player.get_single() else { return };
    let chunk_world = gen.chunk_size as f32 * gen.tile_size;
    let pcx = (ptr.translation.x / chunk_world).floor() as i32;
    let pcy = (ptr.translation.z / chunk_world).floor() as i32;

    let sampler = WorldSampler::new(seed.0);

    for cy in (pcy - gen.view_radius)..=(pcy + gen.view_radius) {
        for cx in (pcx - gen.view_radius)..=(pcx + gen.view_radius) {
            if !loaded.map.contains_key(&(cx, cy)) {
                load_chunk(
                    &mut commands,
                    &sampler,
                    &gen,
                    &mut loaded,
                    &mut load_ev,
                    ChunkCoord { x: cx, y: cy },
                );
            }
        }
    }

    let r = gen.view_radius + 1;
    for (entity, chunk) in &chunks {
        if (chunk.coord.x - pcx).abs() > r || (chunk.coord.y - pcy).abs() > r {
            commands.entity(entity).despawn_recursive();
            loaded.map.remove(&(chunk.coord.x, chunk.coord.y));
            unload_ev.send(ChunkUnloaded(chunk.coord));
        }
    }
}

fn load_chunk(
    commands: &mut Commands,
    sampler: &WorldSampler,
    gen: &WorldGen,
    loaded: &mut LoadedChunks,
    events: &mut EventWriter<ChunkLoaded>,
    coord: ChunkCoord,
) {
    let size = gen.chunk_size;
    let mut biomes = Vec::with_capacity((size * size) as usize);
    let mut elevation = Vec::with_capacity((size * size) as usize);
    let mut histogram: HashMap<Biome, u32> = HashMap::new();

    for ly in 0..size {
        for lx in 0..size {
            let wx = (coord.x * size + lx) as f64 * gen.tile_size as f64;
            let wy = (coord.y * size + ly) as f64 * gen.tile_size as f64;
            let b = sampler.biome_at(wx, wy, gen);
            let h = sampler.elevation(wx, wy, gen.continent_frequency);
            *histogram.entry(b).or_insert(0) += 1;
            biomes.push(b);
            elevation.push(h);
        }
    }
    let dominant = histogram
        .into_iter()
        .max_by_key(|(_, n)| *n)
        .map(|(b, _)| b)
        .unwrap_or(Biome::DeepOcean);

    let world_x = coord.x as f32 * size as f32 * gen.tile_size;
    let world_z = coord.y as f32 * size as f32 * gen.tile_size;

    let id = commands
        .spawn((
            Chunk { coord, dominant_biome: dominant },
            ChunkTiles {
                size,
                tile_size: gen.tile_size,
                biomes,
                elevation,
            },
            SpatialBundle::from_transform(Transform::from_xyz(world_x, 0.0, world_z)),
            Name::new(format!("chunk_{}_{}", coord.x, coord.y)),
        ))
        .id();
    loaded.map.insert((coord.x, coord.y), id);
    events.send(ChunkLoaded(coord));
}
