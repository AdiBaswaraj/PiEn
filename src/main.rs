//! PiEn — pirate game engine entry point.
//!
//! The engine is composed as Bevy plugins. Each plugin owns a slice of the
//! simulation (world, weather, physics, creatures, combat, treasure, player).
//! Systems talk through components and events, which keeps subsystems
//! decoupled and lets new behaviours emerge from their interactions rather
//! than from hand-scripted sequences.

use bevy::prelude::*;

mod combat;
mod creatures;
mod physics;
mod player;
mod treasure;
mod weather;
mod world;

use combat::CombatPlugin;
use creatures::CreaturesPlugin;
use physics::PhysicsPlugin;
use player::PlayerPlugin;
use treasure::TreasurePlugin;
use weather::WeatherPlugin;
use world::WorldPlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "PiEn — Pirate Engine".into(),
                resolution: (1280.0, 720.0).into(),
                present_mode: bevy::window::PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(ClearColor(Color::srgb(0.02, 0.05, 0.10)))
        .insert_resource(SimTime::default())
        .add_plugins((
            WorldPlugin,
            WeatherPlugin,
            PhysicsPlugin,
            CreaturesPlugin,
            CombatPlugin,
            TreasurePlugin,
            PlayerPlugin,
        ))
        .add_systems(Update, advance_sim_time)
        .run();
}

/// Global simulation clock. Keeps weather, tides and creature AI in lockstep
/// so a world "day" is a single source of truth across every subsystem.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct SimTime {
    /// Seconds since the world was spawned.
    pub seconds: f32,
    /// Fractional day in [0, 1). Drives day/night, temperature cycles.
    pub day_phase: f32,
}

const SECONDS_PER_DAY: f32 = 1200.0; // 20 real minutes == 1 in-game day

fn advance_sim_time(time: Res<Time>, mut sim: ResMut<SimTime>) {
    sim.seconds += time.delta_seconds();
    sim.day_phase = (sim.seconds / SECONDS_PER_DAY).fract();
}
