//! Treasure hunting. Every chunk gets a chance of a buried cache. Kills can
//! drop loot. Maps, chests and keys are plain components so UI and combat
//! can reason about them uniformly.
//!
//! The point of the game is choice: loot doesn't funnel the player toward
//! scripted quests. A map is just a hint pointing to coordinates; getting
//! there is the player's problem, and the world + weather decide whether
//! it's actually feasible.

use bevy::prelude::*;
use rand::Rng;

use crate::combat::CreatureKilled;
use crate::creatures::Species;
use crate::world::{Biome, Chunk, ChunkTiles};

pub struct TreasurePlugin;

impl Plugin for TreasurePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TreasureFound>()
            .add_systems(Update, (spawn_treasure_in_new_chunks, drop_loot_on_kill));
    }
}

/// A buried, findable treasure. Typically has a map somewhere in the world
/// hinting at the coordinate — we don't enforce it, just generate both.
#[derive(Component, Clone, Debug)]
pub struct Treasure {
    pub gold: u32,
    pub rarity: Rarity,
    pub contents: Vec<Loot>,
    pub buried: bool,
}

#[derive(Clone, Copy, Debug)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    Legendary,
    Cursed,
}

#[derive(Clone, Debug)]
pub enum Loot {
    Gold(u32),
    TreasureMap { target: Vec2 },
    Gem,
    Relic { name: &'static str },
    // Crafting reagents harvested from monsters
    DragonScale,
    KrakenInk,
    UndeadBone,
    EldritchShard,
}

#[derive(Event, Clone, Debug)]
pub struct TreasureFound {
    pub entity: Entity,
    pub at: Vec3,
}

fn spawn_treasure_in_new_chunks(
    mut commands: Commands,
    chunks: Query<(&Chunk, &ChunkTiles, &Transform), Added<Chunk>>,
) {
    let mut rng = rand::thread_rng();
    for (chunk, tiles, chunk_tr) in &chunks {
        // Ocean chunks rarely hold surface treasure. Cursed islands hold more.
        let odds: f64 = match chunk.dominant_biome {
            Biome::DeepOcean => 0.05,
            Biome::ShallowSea | Biome::CoralReef => 0.15,
            Biome::HauntedMarsh => 0.7,
            Biome::VolcanicAsh => 0.4,
            _ => 0.25,
        };
        if !rng.gen_bool(odds) {
            continue;
        }

        let lx = rng.gen_range(0..tiles.size);
        let ly = rng.gen_range(0..tiles.size);
        let tile_biome = tiles.biomes[tiles.index(lx, ly)];
        let rarity = roll_rarity(tile_biome, &mut rng);

        let pos = chunk_tr.translation
            + Vec3::new(lx as f32 * tiles.tile_size, 0.5, ly as f32 * tiles.tile_size);

        let mut contents = vec![Loot::Gold(rng.gen_range(20..500))];
        if rng.gen_bool(0.4) {
            contents.push(Loot::Gem);
        }
        if matches!(rarity, Rarity::Rare | Rarity::Legendary) {
            contents.push(Loot::Relic { name: pick_relic(&mut rng) });
        }
        if rng.gen_bool(0.25) {
            let tx = rng.gen_range(-4000.0..4000.0);
            let ty = rng.gen_range(-4000.0..4000.0);
            contents.push(Loot::TreasureMap { target: Vec2::new(tx, ty) });
        }

        let gold = contents.iter().filter_map(|l| if let Loot::Gold(g) = l { Some(*g) } else { None }).sum();
        commands.spawn((
            Treasure { gold, rarity, contents, buried: true },
            SpatialBundle::from_transform(Transform::from_translation(pos)),
            Name::new(format!("treasure_{:?}", rarity)),
        ));
    }
}

fn roll_rarity(biome: Biome, rng: &mut impl Rng) -> Rarity {
    let r: f32 = rng.gen();
    match biome {
        Biome::HauntedMarsh => {
            if r < 0.5 { Rarity::Cursed }
            else if r < 0.8 { Rarity::Rare }
            else { Rarity::Legendary }
        }
        Biome::VolcanicAsh => {
            if r < 0.6 { Rarity::Rare } else { Rarity::Legendary }
        }
        _ => {
            if r < 0.6 { Rarity::Common }
            else if r < 0.9 { Rarity::Uncommon }
            else { Rarity::Rare }
        }
    }
}

fn pick_relic(rng: &mut impl Rng) -> &'static str {
    const RELICS: &[&str] = &[
        "Compass of Unknown Harbours",
        "Sextant of the Drowned Queen",
        "Locket of Saint Barnacle",
        "The Unsinkable Lantern",
        "Kraken Tooth Cutlass",
    ];
    RELICS[rng.gen_range(0..RELICS.len())]
}

/// Monsters drop loot on death. Component-driven — the specific species map
/// is the only species-specific thing in the whole combat/loot pipeline.
fn drop_loot_on_kill(
    mut commands: Commands,
    mut kills: EventReader<CreatureKilled>,
) {
    let mut rng = rand::thread_rng();
    for kill in kills.read() {
        let mut contents: Vec<Loot> = Vec::new();
        contents.push(Loot::Gold(rng.gen_range(10..250)));
        match kill.species {
            Species::Kraken => contents.push(Loot::KrakenInk),
            Species::FireDragon | Species::IceDragon | Species::SandDragon => {
                contents.push(Loot::DragonScale);
                if rng.gen_bool(0.25) {
                    contents.push(Loot::Relic { name: "Dragonheart Ember" });
                }
            }
            Species::Undead => contents.push(Loot::UndeadBone),
            Species::Eldritch => contents.push(Loot::EldritchShard),
            _ => {}
        }
        let gold = contents.iter().filter_map(|l| if let Loot::Gold(g) = l { Some(*g) } else { None }).sum();
        commands.spawn((
            Treasure { gold, rarity: Rarity::Uncommon, contents, buried: false },
            SpatialBundle::from_transform(Transform::from_translation(kill.at)),
            Name::new("loot_drop"),
        ));
    }
}
