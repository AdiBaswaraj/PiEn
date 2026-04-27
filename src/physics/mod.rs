//! Physics. Not a full rigid body engine — just what the game actually needs:
//! buoyancy, drag, wave forces, wind, and "environment tags" that combat
//! reads from. Everything is integrated semi-implicitly (symplectic Euler)
//! which is stable at large timesteps and effectively free on the CPU.

use bevy::prelude::*;

use crate::weather::WeatherField;
use crate::SimTime;

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(OceanParams::default())
            .add_systems(
                Update,
                (
                    integrate_bodies,
                    apply_buoyancy,
                    apply_wind_and_waves,
                    tag_environment,
                )
                    .chain(),
            );
    }
}

/// World-level knobs for the ocean. Tuned once, read everywhere.
#[derive(Resource)]
pub struct OceanParams {
    /// Y coordinate of the still-water surface. Land is anything above this
    /// that has Biome != ocean; the y value is still consistent.
    pub sea_level_y: f32,
    /// Base gravity.
    pub gravity: f32,
    /// Water density — bigger = more buoyant hulls.
    pub water_density: f32,
    /// Amplitude of the swell.
    pub wave_amplitude: f32,
    /// Spatial frequency of the primary swell.
    pub wave_frequency: f32,
    /// Angular frequency of the swell.
    pub wave_speed: f32,
}

impl Default for OceanParams {
    fn default() -> Self {
        Self {
            sea_level_y: 0.0,
            gravity: 9.81,
            water_density: 1.0,
            wave_amplitude: 0.6,
            wave_frequency: 0.05,
            wave_speed: 1.2,
        }
    }
}

impl OceanParams {
    /// Analytic height of the water surface at (x, z) and time t.
    /// Sum of two sines — cheap, tileable, enough for buoyancy.
    pub fn surface_height(&self, x: f32, z: f32, t: f32) -> f32 {
        let a = self.wave_amplitude;
        let f = self.wave_frequency;
        let s = self.wave_speed;
        self.sea_level_y
            + a * (x * f + t * s).sin()
            + 0.5 * a * (z * f * 1.3 - t * s * 0.7).sin()
    }
}

/// Velocity + mass. Attach to anything you want the physics loop to move.
#[derive(Component, Debug, Clone, Copy)]
pub struct Body {
    pub velocity: Vec3,
    pub mass: f32,
    /// Damping on linear velocity per second.
    pub linear_damping: f32,
}

impl Default for Body {
    fn default() -> Self {
        Self { velocity: Vec3::ZERO, mass: 1.0, linear_damping: 0.2 }
    }
}

/// Tag an entity as floating so the buoyancy system considers it.
#[derive(Component, Clone, Copy)]
pub struct Floats {
    /// Axis-aligned half-extents for an approximate submerged-volume calc.
    pub half_extents: Vec3,
    /// 0..1 — how much sail is catching wind (0 = sails down).
    pub sail: f32,
    /// 1 for a normal hull; fire dragons have `air_breather = false`
    /// so when buoyancy forces them under they just drown.
    pub air_breather: bool,
}

impl Default for Floats {
    fn default() -> Self {
        Self {
            half_extents: Vec3::new(2.0, 1.0, 5.0),
            sail: 0.0,
            air_breather: true,
        }
    }
}

/// Environment tags — set each frame by `tag_environment`, read by combat
/// and AI. Cheap, avoids repeated biome/weather lookups.
#[derive(Component, Default, Debug, Clone, Copy)]
pub struct Environment {
    pub submerged: bool,
    pub in_rain: bool,
    pub in_fog: bool,
    pub in_heat: bool,
    pub in_freeze: bool,
    pub temperature: f32,
}

fn integrate_bodies(time: Res<Time>, mut q: Query<(&mut Transform, &mut Body)>) {
    let dt = time.delta_seconds();
    for (mut t, mut b) in &mut q {
        let damp = (1.0 - b.linear_damping * dt).max(0.0);
        b.velocity *= damp;
        let v = b.velocity;
        t.translation += v * dt;
    }
}

fn apply_buoyancy(
    time: Res<Time>,
    sim: Res<SimTime>,
    ocean: Res<OceanParams>,
    mut q: Query<(&Transform, &mut Body, &Floats)>,
) {
    let dt = time.delta_seconds();
    let t = sim.seconds;
    for (tr, mut body, floats) in &mut q {
        let pos = tr.translation;
        let water_y = ocean.surface_height(pos.x, pos.z, t);
        // Approximate submerged depth by box bottom vs water surface.
        let bottom = pos.y - floats.half_extents.y;
        let depth = (water_y - bottom).max(0.0);
        if depth <= 0.0 {
            // In air — gravity only.
            body.velocity.y -= ocean.gravity * dt;
            continue;
        }
        // Volume submerged (clamped to full box).
        let submerged_h = depth.min(floats.half_extents.y * 2.0);
        let volume = floats.half_extents.x * 2.0
            * floats.half_extents.z * 2.0
            * submerged_h;
        let buoyant_force = ocean.water_density * ocean.gravity * volume;
        let net = buoyant_force - body.mass * ocean.gravity;
        body.velocity.y += (net / body.mass) * dt;
        // Water drag — heavy so things don't oscillate forever. Y damping
        // stops the buoyancy/gravity tug-of-war from making the ship bob
        // forever; it settles to the waterline within a couple of seconds.
        let drag = 1.5 * submerged_h;
        body.velocity.x *= (1.0 - drag * dt).max(0.0);
        body.velocity.y *= (1.0 - drag * dt).max(0.0);
        body.velocity.z *= (1.0 - drag * dt).max(0.0);
    }
}

fn apply_wind_and_waves(
    time: Res<Time>,
    weather: Res<WeatherField>,
    mut q: Query<(&Transform, &mut Body, &Floats)>,
) {
    let dt = time.delta_seconds();
    for (tr, mut body, floats) in &mut q {
        let cell = weather.sample(Vec2::new(tr.translation.x, tr.translation.z));
        // Sail catches wind.
        let wind_force = Vec3::new(cell.wind.x, 0.0, cell.wind.y) * floats.sail * 2.0;
        let inv_mass = 1.0 / body.mass;
        body.velocity += wind_force * dt * inv_mass;
        // Storm chop — small random-ish perpetual shove vertically.
        if cell.storm > 0.0 {
            body.velocity.y += cell.storm * (tr.translation.x * 0.1).sin() * dt;
        }
    }
}

fn tag_environment(
    ocean: Res<OceanParams>,
    sim: Res<SimTime>,
    weather: Res<WeatherField>,
    mut q: Query<(&Transform, &mut Environment)>,
) {
    let t = sim.seconds;
    for (tr, mut env) in &mut q {
        let water_y = ocean.surface_height(tr.translation.x, tr.translation.z, t);
        let cell = weather.sample(Vec2::new(tr.translation.x, tr.translation.z));
        env.submerged = tr.translation.y < water_y - 0.2;
        env.in_rain = cell.rain > 0.3;
        env.in_fog = cell.fog > 0.5;
        env.in_heat = cell.heat_haze > 0.3 || cell.temperature > 35.0;
        env.in_freeze = cell.freeze > 0.5 || cell.temperature < 0.0;
        env.temperature = cell.temperature;
    }
}
