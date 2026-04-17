//! Combat. Not "press attack button". Combat here is a rules engine that
//! converts game world facts (this creature is submerged, that one is on
//! fire, lightning struck here) into damage events.
//!
//! No per-monster scripts. If you want to kill a fire dragon by drowning it,
//! you don't invoke special code — you just get it submerged and the
//! environment-damage rule does the rest.

use bevy::prelude::*;

use crate::creatures::{Affinity, Creature, Species};
use crate::physics::Environment;
use crate::weather::WeatherEvent;
use crate::SimTime;

pub struct CombatPlugin;

impl Plugin for CombatPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<DamageEvent>()
            .add_event::<CreatureKilled>()
            .add_systems(
                Update,
                (
                    environmental_damage,
                    weather_damage,
                    apply_damage,
                    reap_dead,
                )
                    .chain(),
            );
    }
}

/// Kinds of damage. Each hits a specific Affinity channel.
#[derive(Clone, Copy, Debug)]
pub enum DamageKind {
    Physical,
    Fire,
    Ice,
    Lightning,
    Water,
    Holy,
    Sanity,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct DamageEvent {
    pub target: Entity,
    pub kind: DamageKind,
    pub amount: f32,
    pub source: Option<Entity>,
}

#[derive(Event, Clone, Copy, Debug)]
pub struct CreatureKilled {
    pub entity: Entity,
    pub species: Species,
    pub at: Vec3,
}

/// Read the `Environment` tag on each creature and convert inhospitable
/// conditions into damage per second. This is where "drown the fire dragon"
/// actually happens.
fn environmental_damage(
    time: Res<Time>,
    mut writer: EventWriter<DamageEvent>,
    q: Query<(Entity, &Creature, &Environment, &Affinity)>,
) {
    let dt = time.delta_seconds();
    for (e, creature, env, _aff) in &q {
        if env.submerged && !can_breathe_water(creature.species) {
            writer.send(DamageEvent {
                target: e,
                kind: DamageKind::Water,
                amount: 80.0 * dt,
                source: None,
            });
        }
        if env.in_heat && matches!(creature.species, Species::IceDragon | Species::Undead) {
            writer.send(DamageEvent {
                target: e,
                kind: DamageKind::Fire,
                amount: 12.0 * dt,
                source: None,
            });
        }
        if env.in_freeze && matches!(creature.species, Species::FireDragon | Species::SandDragon) {
            writer.send(DamageEvent {
                target: e,
                kind: DamageKind::Ice,
                amount: 12.0 * dt,
                source: None,
            });
        }
        // Eldritch things dissolve in direct sun / strong heat haze.
        if env.in_heat && matches!(creature.species, Species::Eldritch) {
            writer.send(DamageEvent {
                target: e,
                kind: DamageKind::Sanity,
                amount: 25.0 * dt,
                source: None,
            });
        }
    }
}

/// Lightning and freeze-fronts can hit anything nearby — creatures and
/// players alike. Radius-damage, cheap O(n) scan because there are only a
/// handful of weather events per frame.
fn weather_damage(
    mut weather_events: EventReader<WeatherEvent>,
    mut writer: EventWriter<DamageEvent>,
    q: Query<(Entity, &Transform), With<Creature>>,
) {
    for ev in weather_events.read() {
        match *ev {
            WeatherEvent::LightningStrike { at, power } => {
                for (e, tr) in &q {
                    let d = Vec2::new(tr.translation.x, tr.translation.z).distance(at);
                    if d < 30.0 {
                        writer.send(DamageEvent {
                            target: e,
                            kind: DamageKind::Lightning,
                            amount: power * (1.0 - d / 30.0),
                            source: None,
                        });
                    }
                }
            }
            WeatherEvent::FreezeFront { at, radius } => {
                for (e, tr) in &q {
                    let d = Vec2::new(tr.translation.x, tr.translation.z).distance(at);
                    if d < radius {
                        writer.send(DamageEvent {
                            target: e,
                            kind: DamageKind::Ice,
                            amount: 40.0 * (1.0 - d / radius),
                            source: None,
                        });
                    }
                }
            }
            WeatherEvent::Waterspout { at } => {
                for (e, tr) in &q {
                    let d = Vec2::new(tr.translation.x, tr.translation.z).distance(at);
                    if d < 40.0 {
                        writer.send(DamageEvent {
                            target: e,
                            kind: DamageKind::Water,
                            amount: 30.0,
                            source: None,
                        });
                    }
                }
            }
        }
    }
}

fn apply_damage(
    mut events: EventReader<DamageEvent>,
    mut q: Query<(&mut Creature, &Affinity)>,
) {
    for ev in events.read() {
        if let Ok((mut creature, aff)) = q.get_mut(ev.target) {
            let mul = match ev.kind {
                DamageKind::Physical => 1.0,
                DamageKind::Fire => aff.fire,
                DamageKind::Ice => aff.ice,
                DamageKind::Lightning => aff.lightning,
                DamageKind::Water => aff.water,
                DamageKind::Holy => aff.holy,
                DamageKind::Sanity => aff.sanity,
            };
            creature.hp -= ev.amount * mul;
        }
    }
}

fn reap_dead(
    mut commands: Commands,
    sim: Res<SimTime>,
    mut killed: EventWriter<CreatureKilled>,
    q: Query<(Entity, &Creature, &Transform)>,
) {
    for (e, creature, tr) in &q {
        if creature.hp <= 0.0 {
            killed.send(CreatureKilled {
                entity: e,
                species: creature.species,
                at: tr.translation,
            });
            commands.entity(e).despawn_recursive();
        }
    }
    // `sim` is intentionally read so future rules (day/night-only kills) can
    // branch on it without plumbing a new resource through.
    let _ = sim.seconds;
}

fn can_breathe_water(species: Species) -> bool {
    matches!(species, Species::Kraken | Species::SeaSerpent | Species::Undead | Species::Eldritch)
}
