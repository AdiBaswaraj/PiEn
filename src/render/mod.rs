//! Placeholder rendering. Everything here is intended to be swapped for real
//! assets later — each visual is driven by an ECS component so replacing
//! "cuboid with a colour" with "loaded GLTF model" is a one-line change.
//!
//! We use one mesh per chunk (terrain as a grid of coloured quads), a single
//! large water plane that follows the player, and one primitive mesh per
//! ship / creature. No textures, no shadows: flat-shaded, dead cheap.

use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

use crate::creatures::{Creature, Species};
use crate::physics::OceanParams;
use crate::player::Player;
use crate::treasure::Treasure;
use crate::world::{Biome, ChunkTiles};

pub struct RenderPlugin;

impl Plugin for RenderPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AmbientLight {
            color: Color::srgb(0.8, 0.85, 1.0),
            brightness: 300.0,
        })
        .add_systems(Startup, spawn_water_plane)
        .add_systems(
            Update,
            (
                mesh_new_chunks,
                mesh_new_creatures,
                mesh_new_ship,
                mesh_new_treasure,
                follow_water_to_player,
            ),
        );
    }
}

/// Marker for the single ocean plane that tracks the player so the ocean
/// feels infinite without actually tiling it.
#[derive(Component)]
struct WaterPlane;

fn spawn_water_plane(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    ocean: Res<OceanParams>,
) {
    let mesh = meshes.add(Plane3d::default().mesh().size(4000.0, 4000.0));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.08, 0.22, 0.38, 0.85),
        perceptual_roughness: 0.6,
        reflectance: 0.3,
        alpha_mode: AlphaMode::Blend,
        ..default()
    });
    commands.spawn((
        WaterPlane,
        PbrBundle {
            mesh,
            material,
            transform: Transform::from_xyz(0.0, ocean.sea_level_y, 0.0),
            ..default()
        },
        Name::new("water_plane"),
    ));
}

fn follow_water_to_player(
    player: Query<&Transform, (With<Player>, Without<WaterPlane>)>,
    mut water: Query<&mut Transform, With<WaterPlane>>,
    ocean: Res<OceanParams>,
) {
    let Ok(p) = player.get_single() else { return };
    let Ok(mut w) = water.get_single_mut() else { return };
    w.translation.x = p.translation.x;
    w.translation.z = p.translation.z;
    w.translation.y = ocean.sea_level_y;
}

/// Build one mesh per chunk: a grid of coloured quads at tile elevation.
/// Cheap because the per-vertex data is trivial and drawn in one call.
fn mesh_new_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    chunks: Query<(Entity, &ChunkTiles), Added<ChunkTiles>>,
) {
    for (entity, tiles) in &chunks {
        let mesh = meshes.add(build_chunk_mesh(tiles));
        let material = materials.add(StandardMaterial {
            base_color: Color::WHITE,
            perceptual_roughness: 1.0,
            reflectance: 0.05,
            ..default()
        });
        commands.entity(entity).insert((mesh, material));
    }
}

const ELEVATION_SCALE: f32 = 12.0;
const SEA_THRESHOLD: f32 = 0.05;

fn build_chunk_mesh(tiles: &ChunkTiles) -> Mesh {
    let size = tiles.size;
    let ts = tiles.tile_size;
    let n_quads = (size * size) as usize;

    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(n_quads * 4);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(n_quads * 4);
    let mut colors: Vec<[f32; 4]> = Vec::with_capacity(n_quads * 4);
    let mut indices: Vec<u32> = Vec::with_capacity(n_quads * 6);

    for ly in 0..size {
        for lx in 0..size {
            let idx = tiles.index(lx, ly);
            let biome = tiles.biomes[idx];
            let elev = tiles.elevation[idx];
            // Above sea: push up. Below sea: flatten to a shallow seabed so
            // the water plane covers it.
            let y = if elev > SEA_THRESHOLD {
                (elev - SEA_THRESHOLD) * ELEVATION_SCALE
            } else {
                -0.5 + (elev * 2.0).max(-4.0)
            };

            let x0 = lx as f32 * ts;
            let x1 = x0 + ts;
            let z0 = ly as f32 * ts;
            let z1 = z0 + ts;

            let color = biome_color(biome);
            let base = positions.len() as u32;
            positions.push([x0, y, z0]);
            positions.push([x1, y, z0]);
            positions.push([x1, y, z1]);
            positions.push([x0, y, z1]);
            for _ in 0..4 {
                normals.push([0.0, 1.0, 0.0]);
                colors.push(color);
            }
            indices.extend_from_slice(&[base, base + 3, base + 2, base, base + 2, base + 1]);
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));
    mesh
}

fn biome_color(b: Biome) -> [f32; 4] {
    use Biome::*;
    match b {
        DeepOcean => [0.02, 0.06, 0.18, 1.0],
        ShallowSea => [0.08, 0.30, 0.50, 1.0],
        CoralReef => [0.30, 0.60, 0.65, 1.0],
        Beach => [0.95, 0.88, 0.62, 1.0],
        Jungle => [0.10, 0.38, 0.12, 1.0],
        Desert => [0.92, 0.78, 0.45, 1.0],
        Savanna => [0.72, 0.68, 0.30, 1.0],
        Tundra => [0.55, 0.62, 0.55, 1.0],
        IceCap => [0.88, 0.93, 0.98, 1.0],
        VolcanicAsh => [0.18, 0.09, 0.08, 1.0],
        HauntedMarsh => [0.18, 0.22, 0.16, 1.0],
        MangroveSwamp => [0.22, 0.38, 0.22, 1.0],
    }
}

fn mesh_new_ship(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    ships: Query<Entity, Added<Player>>,
) {
    for entity in &ships {
        let hull = meshes.add(Cuboid::new(3.0, 1.5, 8.0));
        let mast = meshes.add(Cuboid::new(0.3, 6.0, 0.3));
        let sail = meshes.add(Cuboid::new(0.1, 4.0, 3.0));

        let hull_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.35, 0.20, 0.10),
            perceptual_roughness: 0.9,
            ..default()
        });
        let mast_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.30, 0.18, 0.08),
            ..default()
        });
        let sail_mat = materials.add(StandardMaterial {
            base_color: Color::srgb(0.90, 0.88, 0.82),
            ..default()
        });

        commands.entity(entity).insert((hull, hull_mat));
        commands.entity(entity).with_children(|p| {
            p.spawn(PbrBundle {
                mesh: mast,
                material: mast_mat,
                transform: Transform::from_xyz(0.0, 3.5, 0.0),
                ..default()
            });
            p.spawn(PbrBundle {
                mesh: sail,
                material: sail_mat,
                transform: Transform::from_xyz(0.0, 4.5, 0.5),
                ..default()
            });
        });
    }
}

fn mesh_new_creatures(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    new_creatures: Query<(Entity, &Creature), Added<Creature>>,
) {
    for (entity, creature) in &new_creatures {
        let (size, color) = visual_for(creature.species);
        let mesh = meshes.add(Cuboid::new(size.x, size.y, size.z));
        let material = materials.add(StandardMaterial {
            base_color: color,
            perceptual_roughness: 0.8,
            reflectance: 0.1,
            ..default()
        });
        commands.entity(entity).insert((mesh, material));
    }
}

fn visual_for(species: Species) -> (Vec3, Color) {
    use Species::*;
    match species {
        Kraken => (Vec3::new(10.0, 4.0, 10.0), Color::srgb(0.35, 0.10, 0.40)),
        SeaSerpent => (Vec3::new(1.5, 1.5, 10.0), Color::srgb(0.10, 0.50, 0.30)),
        FireDragon => (Vec3::new(4.0, 3.0, 8.0), Color::srgb(0.80, 0.15, 0.05)),
        IceDragon => (Vec3::new(4.0, 3.0, 8.0), Color::srgb(0.70, 0.85, 0.95)),
        SandDragon => (Vec3::new(3.5, 2.5, 7.0), Color::srgb(0.80, 0.65, 0.30)),
        Wyvern => (Vec3::new(2.0, 2.0, 4.0), Color::srgb(0.30, 0.20, 0.35)),
        GiantSnake => (Vec3::new(1.0, 1.0, 6.0), Color::srgb(0.20, 0.35, 0.10)),
        Undead => (Vec3::new(0.8, 1.8, 0.8), Color::srgb(0.60, 0.60, 0.55)),
        Eldritch => (Vec3::new(6.0, 6.0, 6.0), Color::srgb(0.05, 0.02, 0.10)),
    }
}

fn mesh_new_treasure(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    new_treasures: Query<(Entity, &Treasure), Added<Treasure>>,
) {
    for (entity, treasure) in &new_treasures {
        let mesh = meshes.add(Cuboid::new(1.2, 0.8, 0.8));
        let material = materials.add(StandardMaterial {
            base_color: if treasure.buried {
                Color::srgb(0.30, 0.22, 0.12)
            } else {
                Color::srgb(0.95, 0.80, 0.25)
            },
            ..default()
        });
        commands.entity(entity).insert((mesh, material));
    }
}
