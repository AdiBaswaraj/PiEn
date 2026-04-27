//! Player. In a pirate game the "player" is really the ship first, the
//! captain second. Controls are straightforward: WASD steers, Space hoists
//! sail, Shift drops sail, E interacts with whatever's under the bow.
//!
//! The ship is a regular physics body that floats — buoyancy, wind, waves
//! all already affect it via the physics plugin. This module just wires
//! input to forces.

use bevy::prelude::*;

use crate::physics::{Body, Environment, Floats};
use crate::treasure::{Treasure, TreasureFound};
use crate::world::{find_player_spawn, WorldGen, WorldSeed};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Inventory::default())
            .add_systems(Startup, spawn_player_ship)
            .add_systems(
                Update,
                (
                    handle_input,
                    update_camera,
                    pickup_nearby_treasure,
                    crew_status_readout,
                ),
            );
    }
}

/// Marker for the player ship entity.
#[derive(Component)]
pub struct Player;

/// Captain stats that tick with the environment. Heat stroke, frostbite,
/// scurvy — all just drains on these numbers.
#[derive(Component, Debug, Clone, Copy)]
pub struct Captain {
    pub stamina: f32,
    pub sanity: f32,
    pub warmth: f32,
}

impl Default for Captain {
    fn default() -> Self {
        Self { stamina: 100.0, sanity: 100.0, warmth: 100.0 }
    }
}

/// Ship-specific state (heading, rudder angle, hull integrity).
#[derive(Component, Debug, Clone, Copy)]
pub struct Ship {
    pub hull: f32,
    pub max_hull: f32,
    pub heading: f32, // radians
    pub rudder: f32,  // -1..1
}

impl Default for Ship {
    fn default() -> Self {
        Self { hull: 1000.0, max_hull: 1000.0, heading: 0.0, rudder: 0.0 }
    }
}

#[derive(Resource, Default, Debug)]
pub struct Inventory {
    pub gold: u32,
    pub maps: Vec<Vec2>,
    pub relics: Vec<String>,
    pub dragon_scales: u32,
    pub kraken_ink: u32,
    pub undead_bones: u32,
    pub eldritch_shards: u32,
    pub gems: u32,
}

fn spawn_player_ship(
    mut commands: Commands,
    seed: Res<WorldSeed>,
    gen: Res<WorldGen>,
) {
    // Phase 1 spec: spawn in open water within sight of an island. World
    // origin is NOT guaranteed to be water — Perlin noise puts whatever it
    // wants there, so we search outward for a shallow-sea tile with land
    // nearby instead of trusting (0, 0, 0).
    let spawn = find_player_spawn(seed.0, &gen);
    info!("Player spawn picked at world ({:.1}, {:.1})", spawn.x, spawn.z);

    // The player ship. Mass is tuned so the buoyant force from the float
    // volume (≈230 m³ when fully submerged, with water_density=1) holds
    // the ship at roughly 35% submerged — i.e. a sensible waterline.
    // If you raise mass much past ~120 the ship sinks; past ~50 it floats
    // like a cork on top of the waves.
    commands.spawn((
        Player,
        Ship::default(),
        Captain::default(),
        Body { velocity: Vec3::ZERO, mass: 80.0, linear_damping: 0.15 },
        Floats {
            half_extents: Vec3::new(3.0, 1.2, 8.0),
            sail: 0.0,
            air_breather: true,
        },
        Environment::default(),
        SpatialBundle::from_transform(Transform::from_translation(spawn)),
        Name::new("player_ship"),
    ));

    // Camera initialised at the same offset `update_camera` will follow with
    // — otherwise the first ~30 frames are a slow lerp from world origin.
    let cam_offset = Vec3::new(0.0, 30.0, 25.0);
    commands.spawn(Camera3dBundle {
        transform: Transform::from_translation(spawn + cam_offset)
            .looking_at(spawn, Vec3::Y),
        ..default()
    });

    // Ambient sun. One directional light is effectively free.
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 11000.0,
            shadows_enabled: false, // cheapest lighting path
            ..default()
        },
        transform: Transform::from_rotation(Quat::from_euler(
            EulerRot::XYZ, -0.8, 0.3, 0.0,
        )),
        ..default()
    });
}

fn handle_input(
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut q: Query<(&mut Ship, &mut Body, &mut Floats), With<Player>>,
) {
    let dt = time.delta_seconds();
    let Ok((mut ship, mut body, mut floats)) = q.get_single_mut() else { return };

    let mut steer = 0.0;
    if keys.pressed(KeyCode::KeyA) { steer -= 1.0; }
    if keys.pressed(KeyCode::KeyD) { steer += 1.0; }
    ship.rudder = (ship.rudder + steer * dt * 2.0).clamp(-1.0, 1.0);
    ship.rudder *= 1.0 - dt * 1.2;
    ship.heading += ship.rudder * dt * 0.8;

    if keys.pressed(KeyCode::Space) {
        floats.sail = (floats.sail + dt * 0.8).min(1.0);
    }
    if keys.pressed(KeyCode::ShiftLeft) {
        floats.sail = (floats.sail - dt * 1.5).max(0.0);
    }
    if keys.pressed(KeyCode::KeyW) {
        let fwd = Vec3::new(ship.heading.sin(), 0.0, ship.heading.cos());
        body.velocity += fwd * dt * 6.0;
    }
    if keys.pressed(KeyCode::KeyS) {
        let fwd = Vec3::new(ship.heading.sin(), 0.0, ship.heading.cos());
        body.velocity -= fwd * dt * 3.0;
    }
}

fn update_camera(
    player: Query<&Transform, (With<Player>, Without<Camera3d>)>,
    mut cam: Query<&mut Transform, With<Camera3d>>,
) {
    let Ok(ptr) = player.get_single() else { return };
    let Ok(mut c) = cam.get_single_mut() else { return };
    let desired = ptr.translation + Vec3::new(0.0, 30.0, 25.0);
    c.translation = c.translation.lerp(desired, 0.1);
    c.look_at(ptr.translation, Vec3::Y);
}

fn pickup_nearby_treasure(
    mut commands: Commands,
    mut inv: ResMut<Inventory>,
    mut found: EventWriter<TreasureFound>,
    player: Query<&Transform, With<Player>>,
    treasures: Query<(Entity, &Transform, &Treasure)>,
) {
    let Ok(ptr) = player.get_single() else { return };
    for (e, tr, treasure) in &treasures {
        if ptr.translation.distance(tr.translation) < 4.0 && !treasure.buried {
            absorb_treasure(&mut inv, treasure);
            found.send(TreasureFound { entity: e, at: tr.translation });
            commands.entity(e).despawn_recursive();
        }
    }
}

fn absorb_treasure(inv: &mut Inventory, t: &Treasure) {
    use crate::treasure::Loot;
    inv.gold += t.gold;
    for l in &t.contents {
        match l {
            Loot::Gold(_) => {} // already summed in `gold`
            Loot::Gem => inv.gems += 1,
            Loot::Relic { name } => inv.relics.push((*name).into()),
            Loot::TreasureMap { target } => inv.maps.push(*target),
            Loot::DragonScale => inv.dragon_scales += 1,
            Loot::KrakenInk => inv.kraken_ink += 1,
            Loot::UndeadBone => inv.undead_bones += 1,
            Loot::EldritchShard => inv.eldritch_shards += 1,
        }
    }
}

/// Environment slowly erodes the captain. Read-only UI / death loop hook.
fn crew_status_readout(
    time: Res<Time>,
    mut q: Query<(&mut Captain, &Environment), With<Player>>,
) {
    let dt = time.delta_seconds();
    for (mut cap, env) in &mut q {
        if env.in_heat { cap.stamina -= 2.0 * dt; cap.warmth = (cap.warmth + 2.0 * dt).min(100.0); }
        if env.in_freeze { cap.warmth -= 3.0 * dt; }
        if env.in_fog { cap.sanity -= 0.5 * dt; }
        if env.submerged { cap.stamina -= 10.0 * dt; }
        // Passive recovery in fair weather.
        if !env.in_heat && !env.in_freeze && !env.in_fog && !env.submerged {
            cap.stamina = (cap.stamina + 3.0 * dt).min(100.0);
            cap.sanity = (cap.sanity + 1.0 * dt).min(100.0);
            cap.warmth = (cap.warmth + 1.0 * dt).min(100.0);
        }
    }
}
