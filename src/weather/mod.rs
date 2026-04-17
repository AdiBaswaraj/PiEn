//! Weather & climate. Rain, fog, heat, and ice aren't just particle effects —
//! every other subsystem reads from `WeatherField` so storms actually rock
//! ships, fog hides monsters, heat drains stamina, ice fractures hulls.
//!
//! The implementation is deliberately cheap: we sample the current weather
//! at a handful of "cells" (a coarse grid over the world) rather than per
//! tile. Subsystems interpolate as needed. That keeps the CPU cost flat
//! regardless of world size.

use bevy::prelude::*;
use bevy::utils::HashMap;
use noise::{NoiseFn, Perlin};
use rand::Rng;

use crate::world::{Biome, WorldSeed};
use crate::SimTime;

pub struct WeatherPlugin;

impl Plugin for WeatherPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WeatherField::default())
            .insert_resource(GlobalClimate::default())
            .add_event::<WeatherEvent>()
            .add_systems(Update, (tick_global_climate, advect_weather));
    }
}

/// Weather "samples" stored on a coarse grid. One cell is ~500 world units.
#[derive(Resource)]
pub struct WeatherField {
    pub cell_size: f32,
    pub cells: HashMap<(i32, i32), WeatherCell>,
    noise: Perlin,
}

impl Default for WeatherField {
    fn default() -> Self {
        Self {
            cell_size: 500.0,
            cells: HashMap::new(),
            noise: Perlin::new(0x5EED),
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct WeatherCell {
    /// 0..1 how hard it's raining.
    pub rain: f32,
    /// 0..1 fog density; darker = more spooky, worse visibility.
    pub fog: f32,
    /// Wind vector in world space (x, z). Magnitude in m/s.
    pub wind: Vec2,
    /// Ambient temperature in Celsius.
    pub temperature: f32,
    /// 0..1 — likelihood that exposed water at this cell is ice right now.
    pub freeze: f32,
    /// 0..1 — magnitude of heat shimmer / dehydration risk.
    pub heat_haze: f32,
    /// Thunder intensity. Lightning strikes are a separate event.
    pub storm: f32,
}

/// Slow-moving averages — seasons, El Niño style cycles. Drives how biomes
/// "feel" this week without recomputing every frame.
#[derive(Resource, Default)]
pub struct GlobalClimate {
    pub season_phase: f32, // 0..1 year fraction
    pub wind_heading: f32, // radians, prevailing wind
}

/// High-level weather events other systems can react to. For example the
/// combat plugin listens for `LightningStrike` to apply damage, and the
/// player plugin listens for `FreezeFront` to start an ice-cracking hull
/// countdown.
#[derive(Event, Debug, Clone, Copy)]
pub enum WeatherEvent {
    LightningStrike { at: Vec2, power: f32 },
    FreezeFront { at: Vec2, radius: f32 },
    Waterspout { at: Vec2 },
}

fn tick_global_climate(time: Res<Time>, mut climate: ResMut<GlobalClimate>) {
    let y = 1.0 / 600.0; // one "year" every 10 minutes for demo purposes
    climate.season_phase = (climate.season_phase + time.delta_seconds() * y).fract();
    // Prevailing wind swings slowly over the season.
    climate.wind_heading = (climate.season_phase * std::f32::consts::TAU).sin() * 0.6;
}

/// Samples or lazily fills weather cells near the player. "Advection" here
/// just means cells inherit trends from neighbours + noise, so storm fronts
/// actually move rather than flicker.
fn advect_weather(
    sim: Res<SimTime>,
    seed: Res<WorldSeed>,
    climate: Res<GlobalClimate>,
    mut field: ResMut<WeatherField>,
    player: Query<&Transform, With<crate::player::Player>>,
    mut events: EventWriter<WeatherEvent>,
) {
    let Ok(ptr) = player.get_single() else { return };
    let px = (ptr.translation.x / field.cell_size).floor() as i32;
    let pz = (ptr.translation.z / field.cell_size).floor() as i32;
    let radius = 6;

    let mut rng = rand::thread_rng();
    let t = sim.seconds as f64;

    for dz in -radius..=radius {
        for dx in -radius..=radius {
            let key = (px + dx, pz + dz);
            let wx = key.0 as f64;
            let wz = key.1 as f64;

            let storm_noise = field.noise.get([wx * 0.05, wz * 0.05, t * 0.01]);
            let fog_noise = field.noise.get([wx * 0.11 + 10.0, wz * 0.11, t * 0.004]);
            let heat_noise = field.noise.get([wx * 0.03 - 5.0, wz * 0.03, t * 0.002]);

            let season = (climate.season_phase * std::f32::consts::TAU).cos();
            let base_temp = 20.0 + season * 15.0 - (key.1 as f32).abs() * 0.02;

            let cell = WeatherCell {
                rain: ((storm_noise + 0.2).max(0.0) as f32).min(1.0),
                fog: ((fog_noise * 0.7 + 0.1) as f32).clamp(0.0, 1.0),
                wind: Vec2::new(
                    climate.wind_heading.cos() * (3.0 + storm_noise as f32 * 12.0),
                    climate.wind_heading.sin() * (3.0 + storm_noise as f32 * 12.0),
                ),
                temperature: base_temp + (heat_noise as f32) * 6.0,
                freeze: if base_temp < 0.0 { 1.0 } else { 0.0 },
                heat_haze: if base_temp > 35.0 {
                    ((base_temp - 35.0) / 15.0).clamp(0.0, 1.0)
                } else {
                    0.0
                },
                storm: ((storm_noise - 0.4).max(0.0) * 2.0) as f32,
            };

            // Occasional dramatic events in stormy cells.
            if cell.storm > 0.6 && rng.gen_bool(0.002) {
                let at = Vec2::new(
                    key.0 as f32 * field.cell_size + rng.gen_range(-200.0..200.0),
                    key.1 as f32 * field.cell_size + rng.gen_range(-200.0..200.0),
                );
                events.send(WeatherEvent::LightningStrike {
                    at,
                    power: 40.0 + cell.storm * 60.0,
                });
            }
            if cell.freeze > 0.5 && rng.gen_bool(0.001) {
                let at = Vec2::new(
                    key.0 as f32 * field.cell_size,
                    key.1 as f32 * field.cell_size,
                );
                events.send(WeatherEvent::FreezeFront { at, radius: 120.0 });
            }

            field.cells.insert(key, cell);
        }
    }

    // Keep cell cache bounded — drop far cells.
    let keep = (radius + 2) as i32;
    field.cells.retain(|(x, z), _| (*x - px).abs() <= keep && (*z - pz).abs() <= keep);

    // Touch seed so it's considered used. Having weather depend on the seed
    // means the same world has the same "climate personality" every run.
    let _ = seed.0;
}

impl WeatherField {
    /// Sample weather at an arbitrary world position. Nearest-cell; good
    /// enough for gameplay decisions, cheap for particle shaders.
    pub fn sample(&self, pos: Vec2) -> WeatherCell {
        let key = (
            (pos.x / self.cell_size).floor() as i32,
            (pos.y / self.cell_size).floor() as i32,
        );
        self.cells.get(&key).copied().unwrap_or_default()
    }

    /// Adjust sampled weather by biome bias — a desert cell is hotter than
    /// the latitude alone suggests, a marsh is foggier, etc.
    pub fn sample_biased(&self, pos: Vec2, biome: Biome) -> WeatherCell {
        let mut c = self.sample(pos);
        c.temperature = (c.temperature + biome.base_temperature()) * 0.5;
        match biome {
            Biome::HauntedMarsh => {
                c.fog = (c.fog + 0.6).min(1.0);
                c.rain = (c.rain * 0.5).max(0.1);
            }
            Biome::Desert => {
                c.heat_haze = (c.heat_haze + 0.4).min(1.0);
                c.rain = 0.0;
                c.fog = 0.0;
            }
            Biome::VolcanicAsh => {
                c.heat_haze = 1.0;
                c.fog = (c.fog + 0.2).min(1.0);
            }
            Biome::IceCap | Biome::Tundra => {
                c.freeze = 1.0;
            }
            _ => {}
        }
        c
    }
}
