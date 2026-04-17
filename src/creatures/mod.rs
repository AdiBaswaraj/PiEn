//! Creatures: krakens, dragons, sea serpents, undead, eldritch things.
//!
//! The whole design leans on one idea: a creature is a bag of components +
//! a small utility-AI brain. Because elemental affinities and environment
//! tags are components, combat can say "a fire dragon that becomes submerged
//! takes 300 damage/sec" without any creature-specific code. That's how we
//! get environmental kills emerging from the systems instead of scripting
//! every boss fight.

use bevy::prelude::*;
use rand::Rng;

use crate::physics::{Body, Environment, Floats};
use crate::world::{Biome, Chunk, ChunkTiles};

pub struct CreaturesPlugin;

impl Plugin for CreaturesPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnCreature>()
            .add_systems(
                Update,
                (
                    spawn_creatures_for_new_chunks,
                    tick_creature_ai,
                    creature_vs_creature_aggression,
                ),
            );
    }
}

/// What flavour of monster are we?
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Species {
    Kraken,
    SeaSerpent,
    FireDragon,
    IceDragon,
    SandDragon,
    Wyvern,
    GiantSnake,
    Undead,
    Eldritch,
}

/// Physical traits + stats. Everything combat or AI reads hangs off this.
#[derive(Component, Clone, Copy, Debug)]
pub struct Creature {
    pub species: Species,
    pub hp: f32,
    pub max_hp: f32,
    pub aggression: f32,
    /// How close an intruder must be before the creature notices.
    pub perception_range: f32,
    /// Strong-preferred habitat — creatures pathfind back to it when idle.
    pub home_biome: Biome,
}

/// Elemental / damage-type affinities. Key rule of combat:
/// * `vulnerability` > 1.0 → extra damage from this source.
/// * `vulnerability` < 1.0 → resistance.
/// * `vulnerability` == 0.0 → immune.
/// These are the switches that let "drown the fire dragon" work for free.
#[derive(Component, Clone, Copy, Debug)]
pub struct Affinity {
    pub fire: f32,
    pub ice: f32,
    pub lightning: f32,
    /// Water damage — applied when an air-breathing creature is submerged.
    pub water: f32,
    /// Holy damage — baked into many environmental interactions with undead.
    pub holy: f32,
    /// Abstract "reality" damage applied to eldritch things by sunlight etc.
    pub sanity: f32,
}

impl Affinity {
    pub const fn neutral() -> Self {
        Self { fire: 1.0, ice: 1.0, lightning: 1.0, water: 1.0, holy: 1.0, sanity: 1.0 }
    }
}

#[derive(Component, Default, Clone, Copy, Debug)]
pub struct Brain {
    pub state: BrainState,
    pub target: Option<Entity>,
    pub think_cooldown: f32,
}

#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub enum BrainState {
    #[default]
    Idle,
    Hunting,
    Fleeing,
    Feuding, // locked in combat with another creature
}

/// Event that another system can fire to request a spawn.
#[derive(Event, Clone, Copy, Debug)]
pub struct SpawnCreature {
    pub species: Species,
    pub position: Vec3,
}

/// Build the archetype for a given species — one place to tune balance.
pub fn archetype(species: Species) -> (Creature, Affinity, Floats) {
    use Species::*;
    let (hp, aggression, range, home, aff, floats) = match species {
        Kraken => (
            2500.0, 0.8, 120.0, Biome::DeepOcean,
            Affinity {
                fire: 0.5, ice: 1.2, lightning: 2.5, water: 0.0, holy: 1.0, sanity: 1.0,
            },
            // Tentacled, largely submerged — doesn't "float" in the usual sense.
            Floats { half_extents: Vec3::new(6.0, 3.0, 6.0), sail: 0.0, air_breather: false },
        ),
        SeaSerpent => (
            900.0, 0.6, 80.0, Biome::ShallowSea,
            Affinity { fire: 0.6, lightning: 1.8, water: 0.0, ..Affinity::neutral() },
            Floats { half_extents: Vec3::new(1.2, 1.2, 6.0), sail: 0.0, air_breather: false },
        ),
        FireDragon => (
            1800.0, 0.9, 200.0, Biome::VolcanicAsh,
            Affinity {
                fire: 0.0, ice: 2.0, water: 3.0, // THE critical knob — drowning one-shots it
                lightning: 1.0, holy: 1.0, sanity: 1.0,
            },
            Floats { half_extents: Vec3::new(2.5, 2.0, 5.0), sail: 0.0, air_breather: true },
        ),
        IceDragon => (
            1700.0, 0.85, 200.0, Biome::IceCap,
            Affinity { ice: 0.0, fire: 2.5, water: 0.5, ..Affinity::neutral() },
            Floats { half_extents: Vec3::new(2.5, 2.0, 5.0), sail: 0.0, air_breather: true },
        ),
        SandDragon => (
            1400.0, 0.75, 180.0, Biome::Desert,
            Affinity { fire: 0.8, water: 1.6, ..Affinity::neutral() },
            Floats { half_extents: Vec3::new(2.2, 1.5, 4.0), sail: 0.0, air_breather: true },
        ),
        Wyvern => (
            700.0, 0.7, 150.0, Biome::Jungle,
            Affinity { lightning: 1.6, ..Affinity::neutral() },
            Floats { half_extents: Vec3::new(1.2, 1.0, 2.5), sail: 0.0, air_breather: true },
        ),
        GiantSnake => (
            500.0, 0.5, 60.0, Biome::Jungle,
            Affinity::neutral(),
            Floats { half_extents: Vec3::new(0.8, 0.6, 4.0), sail: 0.0, air_breather: true },
        ),
        Undead => (
            200.0, 0.95, 40.0, Biome::HauntedMarsh,
            Affinity {
                fire: 1.8, holy: 3.0, water: 0.8,
                ice: 0.5, lightning: 1.0, sanity: 1.0,
            },
            Floats { half_extents: Vec3::new(0.4, 0.9, 0.4), sail: 0.0, air_breather: false },
        ),
        Eldritch => (
            3500.0, 0.5, 400.0, Biome::HauntedMarsh,
            Affinity {
                sanity: 3.0, // direct sunlight / fire rituals hurt it the most
                fire: 1.4, holy: 2.0, water: 0.6, ice: 1.0, lightning: 0.4,
            },
            Floats { half_extents: Vec3::new(4.0, 4.0, 4.0), sail: 0.0, air_breather: false },
        ),
    };
    (
        Creature {
            species,
            hp,
            max_hp: hp,
            aggression,
            perception_range: range,
            home_biome: home,
        },
        aff,
        floats,
    )
}

/// When a chunk loads, use its biome bias to spawn some creatures in it.
fn spawn_creatures_for_new_chunks(
    mut commands: Commands,
    new_chunks: Query<(&Chunk, &ChunkTiles, &Transform), Added<Chunk>>,
) {
    let mut rng = rand::thread_rng();
    for (chunk, tiles, chunk_tr) in &new_chunks {
        let bias = chunk.dominant_biome.monster_bias();
        // One spawn attempt per ~16 tiles. Keeps things sparse.
        let attempts = (tiles.size * tiles.size) / 16;
        for _ in 0..attempts {
            let lx = rng.gen_range(0..tiles.size);
            let ly = rng.gen_range(0..tiles.size);
            let b = tiles.biomes[tiles.index(lx, ly)];
            let roll: f32 = rng.gen();
            let species = pick_species(b, &bias, roll);
            let Some(species) = species else { continue };

            let pos = chunk_tr.translation
                + Vec3::new(
                    lx as f32 * tiles.tile_size,
                    0.0,
                    ly as f32 * tiles.tile_size,
                );
            spawn_creature(&mut commands, species, pos);
        }
    }
}

fn pick_species(biome: Biome, bias: &crate::world::MonsterBias, roll: f32) -> Option<Species> {
    use Species::*;
    let b = biome.monster_bias();
    // Blend chunk-dominant bias with the specific tile's biome bias.
    let blend = |a: f32, b: f32| (a + b) * 0.5 * 0.02; // 2% per full-weight roll
    let candidates = [
        (Kraken, blend(bias.kraken, b.kraken)),
        (SeaSerpent, blend(bias.sea_serpent, b.sea_serpent)),
        (FireDragon, blend(bias.fire_dragon, b.fire_dragon)),
        (IceDragon, blend(bias.ice_dragon, b.ice_dragon)),
        (SandDragon, blend(bias.sand_dragon, b.sand_dragon)),
        (Wyvern, blend(bias.wyvern, b.wyvern)),
        (GiantSnake, blend(bias.giant_snake, b.giant_snake)),
        (Undead, blend(bias.undead, b.undead)),
        (Eldritch, blend(bias.eldritch, b.eldritch)),
    ];
    let mut acc = 0.0;
    for (s, w) in candidates {
        acc += w;
        if roll < acc {
            return Some(s);
        }
    }
    None
}

pub fn spawn_creature(commands: &mut Commands, species: Species, position: Vec3) {
    let (creature, affinity, floats) = archetype(species);
    commands.spawn((
        creature,
        affinity,
        floats,
        Brain::default(),
        Body { velocity: Vec3::ZERO, mass: 200.0, linear_damping: 0.5 },
        Environment::default(),
        SpatialBundle::from_transform(Transform::from_translation(position)),
        Name::new(format!("{:?}", species)),
    ));
}

/// Utility AI. Each creature periodically re-evaluates what to do.
/// Cheap: only runs every `think_cooldown` seconds, not every frame.
fn tick_creature_ai(
    time: Res<Time>,
    player: Query<(Entity, &Transform), With<crate::player::Player>>,
    mut creatures: Query<(Entity, &mut Brain, &Creature, &Transform, &mut Body)>,
) {
    let dt = time.delta_seconds();
    let Ok((player_entity, player_tr)) = player.get_single() else { return };
    for (_, mut brain, creature, tr, mut body) in &mut creatures {
        brain.think_cooldown -= dt;
        if brain.think_cooldown > 0.0 {
            continue;
        }
        brain.think_cooldown = 0.4;

        let to_player = player_tr.translation - tr.translation;
        let dist = to_player.length();

        brain.state = if creature.hp < creature.max_hp * 0.2 {
            BrainState::Fleeing
        } else if dist < creature.perception_range
            && rand::random::<f32>() < creature.aggression
        {
            brain.target = Some(player_entity);
            BrainState::Hunting
        } else {
            BrainState::Idle
        };

        let speed = match creature.species {
            Species::Kraken | Species::Eldritch => 4.0,
            Species::FireDragon | Species::IceDragon | Species::SandDragon => 14.0,
            Species::Wyvern => 16.0,
            Species::SeaSerpent => 10.0,
            Species::GiantSnake => 6.0,
            Species::Undead => 3.0,
        };

        match brain.state {
            BrainState::Hunting if dist > 1.0 => {
                let dir = to_player / dist;
                body.velocity.x = dir.x * speed;
                body.velocity.z = dir.z * speed;
            }
            BrainState::Fleeing if dist > 0.01 => {
                let dir = -to_player / dist;
                body.velocity.x = dir.x * speed * 1.3;
                body.velocity.z = dir.z * speed * 1.3;
            }
            _ => {
                body.velocity.x *= 0.5;
                body.velocity.z *= 0.5;
            }
        }
    }
}

/// Creatures of opposing elements that wander close enough will fight each
/// other. That's how "make a fire dragon and an ice dragon fight" becomes
/// a real tactic — you just bait them.
fn creature_vs_creature_aggression(
    mut creatures: Query<(Entity, &mut Brain, &Creature, &Transform)>,
) {
    // Collect into a small buffer so we can iterate pairs without the
    // borrow checker complaining.
    let snapshot: Vec<_> = creatures
        .iter()
        .map(|(e, _, c, t)| (e, c.species, t.translation, c.perception_range))
        .collect();

    for (entity, mut brain, creature, tr) in &mut creatures {
        if brain.state == BrainState::Hunting && brain.target.is_some() {
            continue;
        }
        for (other_entity, other_species, other_pos, _) in &snapshot {
            if *other_entity == entity {
                continue;
            }
            let d = tr.translation.distance(*other_pos);
            if d < creature.perception_range && are_enemies(creature.species, *other_species) {
                brain.state = BrainState::Feuding;
                brain.target = Some(*other_entity);
                break;
            }
        }
    }
}

/// Elemental / ecological rivalries. Kept as a matrix of facts so it's
/// trivial to extend without touching AI code.
fn are_enemies(a: Species, b: Species) -> bool {
    use Species::*;
    matches!(
        (a, b),
        (FireDragon, IceDragon) | (IceDragon, FireDragon)
            | (Kraken, SeaSerpent) | (SeaSerpent, Kraken)
            | (Kraken, FireDragon) | (FireDragon, Kraken)
            | (Eldritch, Undead) | (Undead, Eldritch) // eldritch don't tolerate rivals
            | (Wyvern, GiantSnake) | (GiantSnake, Wyvern)
    )
}
