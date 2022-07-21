use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_inspector_egui::Inspectable;
use bevy_rapier2d::prelude::*;
use iyes_loopless::prelude::*;
use rand::{self, thread_rng, Rng};
use std::time::Duration;

use crate::{GameState, MyAssets};

use super::{
    player::{Direction, Player},
    ColliderBundle,
};

#[derive(Debug, Component, Inspectable)]
pub struct Enemy {
    pub hp: f32,
    pub velocity: f32,
    pub direction: Direction,
    pub is_moving: bool,
}

impl Default for Enemy {
    fn default() -> Self {
        Self {
            hp: 10.,
            direction: Direction::None,
            velocity: 200.,
            is_moving: false,
        }
    }
}

pub struct EnemySpawnTimer {
    timer: Timer,
}

#[derive(Bundle, Default, LdtkEntity)]
pub struct EnemyBundle {
    #[sprite_bundle("background.png")]
    #[bundle]
    pub sprite_bundle: SpriteBundle,
    #[from_entity_instance]
    #[bundle]
    pub collider_bundle: ColliderBundle,
    pub enemy: Enemy,
    #[worldly]
    pub worldly: Worldly,
    // The whole EntityInstance can be stored directly as an EntityInstance component
    #[from_entity_instance]
    pub entity_instance: EntityInstance,
}

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::Playing)
                // .with_system(Self::death)
                .with_system(Self::spawn_enemies)
                .with_system(Self::in_range)
                .with_system(Self::hit)
                .into(),
        )
        .add_startup_system(Self::setup_enemy_spawning);
    }
}

impl EnemyPlugin {
    fn setup_enemy_spawning(mut commands: Commands) {
        commands.insert_resource(EnemySpawnTimer {
            // create the repeating timer
            timer: Timer::new(Duration::from_secs(5), true),
        })
    }

    fn spawn_enemies(
        mut commands: Commands,
        time: Res<Time>,
        mut spawn_timer: ResMut<EnemySpawnTimer>,
        level_query: Query<&Handle<LdtkLevel>, (Without<OrthographicProjection>, Without<Player>)>,
        ldtk_levels: Res<Assets<LdtkLevel>>,
        my_assets: Res<MyAssets>,
    ) {
        for level_handle in level_query.iter() {
            if let Some(ldtk_level) = ldtk_levels.get(level_handle) {
                // tick the timer
                spawn_timer.timer.tick(time.delta());

                if spawn_timer.timer.finished() {
                    commands
                        .spawn_bundle(SpriteBundle {
                            texture: my_assets.bg.clone(),
                            ..Default::default()
                        })
                        .insert(Enemy::default())
                        .insert(Name::new("Enemy"))
                        .insert(Transform::from_xyz(
                            thread_rng().gen_range(0.0..ldtk_level.level.px_wid as f32 - 20.),
                            thread_rng().gen_range(0.0..ldtk_level.level.px_hei as f32 - 20.),
                            2.,
                        ))
                        .insert(GravityScale(0.))
                        .insert(Collider::cuboid(7., 7.))
                        .insert(RigidBody::Dynamic)
                        .insert(Sensor)
                        .insert(LockedAxes::ROTATION_LOCKED);
                }
            }
        }
    }

    fn in_range(
        mut enemy_query: Query<&mut Transform, (With<Enemy>, Without<Player>)>,
        player_query: Query<&Transform, With<Player>>,
    ) {
        enemy_query.for_each_mut(|mut enemy_transform| {
            let player_transform = player_query.single();
            let distance = enemy_transform
                .translation
                .distance(player_transform.translation);
            if distance < 100. {
                enemy_transform.translation = enemy_transform
                    .translation
                    .lerp(player_transform.translation, 0.01);
            }
        })
    }

    fn hit(
        mut commands: Commands,
        mut player_query: Query<(&mut Player, Entity)>,
        sensor_enemy_query: Query<Entity, (With<Enemy>, With<Sensor>)>,
        enemy_query: Query<Entity, (With<Enemy>, Without<Sensor>)>,
        rapier_context: Res<RapierContext>,
        time: Res<Time>,
    ) {
        for enemy_e in enemy_query.iter() {
            commands.entity(enemy_e).insert(Sensor);
        }
        for (mut player, player_e) in player_query.iter_mut() {
            for enemy_e in sensor_enemy_query.iter() {
                if rapier_context.intersection_pair(player_e, enemy_e) == Some(true) {
                    player.hp -= 1. * time.delta_seconds();
                }
            }
        }
    }
}

pub fn death(enemy: &Enemy) -> bool {
    if enemy.hp <= 0. {
        return true;
    }
    false
}