use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_inspector_egui::Inspectable;
use bevy_rapier2d::prelude::*;
use iyes_loopless::prelude::*;

use crate::{MainCamera, MyAssets, GameState};

use super::{enemy::Enemy, ColliderBundle};

#[derive(Default, Debug, Inspectable)]
pub enum Direction {
    #[default]
    None,
    Right,
    Left,
    Up,
    Down,
}

#[derive(Component, Inspectable)]
pub struct Player {
    pub hp: f32,
    pub velocity: f32,
    pub direction: Direction,
    pub is_moving: bool,
}

impl Default for Player {
    fn default() -> Self {
        Self {
            hp: 10.,
            direction: Direction::Right,
            velocity: 200.,
            is_moving: false,
        }
    }
}

#[derive(Bundle, Default, LdtkEntity)]
pub struct PlayerBundle {
    #[sprite_bundle("Player.png")]
    #[bundle]
    pub sprite_bundle: SpriteBundle,
    #[from_entity_instance]
    #[bundle]
    pub collider_bundle: ColliderBundle,
    pub player: Player,
    #[worldly]
    pub worldly: Worldly,
    // The whole EntityInstance can be stored directly as an EntityInstance component
    #[from_entity_instance]
    pub entity_instance: EntityInstance,
}

#[derive(Component)]
struct Bullet;

#[derive(Component)]
pub struct Arrow;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_system_set(ConditionSet::new()
            .run_in_state(GameState::Playing)
            .with_system(player_movement)
            .with_system(player_dash)
            .with_system(player_arrow)
            .with_system(player_shoot)
            .with_system(hit)
            .into()
        );
    }
}

fn player_movement(
    mut player_query: Query<(&mut Player, &mut Velocity)>,
    keyboard: Res<Input<KeyCode>>,
) {
    for (mut player, mut rb_vels) in player_query.iter_mut() {
        let up = if keyboard.pressed(KeyCode::W) || keyboard.pressed(KeyCode::Up) {
            player.direction = Direction::Up;
            player.is_moving = true;
            true
        } else {
            false
        };
        let down = if keyboard.pressed(KeyCode::S) || keyboard.pressed(KeyCode::Down) {
            player.direction = Direction::Down;
            player.is_moving = true;
            true
        } else {
            false
        };
        let left = if keyboard.pressed(KeyCode::A) || keyboard.pressed(KeyCode::Left) {
            player.direction = Direction::Left;
            player.is_moving = true;
            true
        } else {
            false
        };
        let right = if keyboard.pressed(KeyCode::D) || keyboard.pressed(KeyCode::Right) {
            player.direction = Direction::Right;
            player.is_moving = true;
            true
        } else {
            false
        };
        if !up && !down && !right && !left {
            player.direction = Direction::None;
            player.is_moving = false;
        }
        let x_axis = -(left as i8) + right as i8;
        let y_axis = -(down as i8) + up as i8;

        let mut move_delta = Vec2::new(x_axis as f32, y_axis as f32);
        if move_delta != Vec2::ZERO {
            move_delta /= move_delta.length();
        }

        // Update the velocity on the rigid_body_component,
        // the bevy_rapier plugin will update the Sprite transform.
        rb_vels.linvel = move_delta * player.velocity;
    }
}

pub fn player_dash(
    mut player_query: Query<(&Player, &mut Velocity)>,
    keyboard: Res<Input<KeyCode>>,
) {
    for (player, mut vel) in player_query.iter_mut() {
        if keyboard.just_pressed(KeyCode::LShift) {
            let move_dir: Vec2 = match player.direction {
                Direction::Right => Vec2::new(1., 0.),
                Direction::Left => Vec2::new(-1., 0.),
                Direction::Up => Vec2::new(0., 1.),
                Direction::Down => Vec2::new(0., -1.),
                _ => Vec2::new(0., 0.),
            };
            vel.linvel = move_dir * 3000.;
        }
    }
}

pub fn player_shoot(
    player_query: Query<(&Player, &Transform, Entity)>,
    windows: Res<Windows>,
    my_assets: Res<MyAssets>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut commands: Commands,
    mouse: Res<Input<MouseButton>>,
) {
    let window = windows.get_primary().unwrap();
    if let Ok((camera, camera_transform)) = q_camera.get_single() {
        if let Some(mouse_position) = window.cursor_position() {
            for (_player, player_transform, player_entity) in player_query.iter() {
                    if mouse.just_pressed(MouseButton::Left) {
                        let window_size = Vec2::new(window.width() as f32, window.height() as f32);
                        let ndc = (mouse_position / window_size) * 2.0 - Vec2::ONE;
                        let ndc_to_world =
                            camera_transform.compute_matrix() * camera.projection_matrix.inverse();
                        let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));
                        let world_pos: Vec2 = world_pos.truncate();
                        let player_pos = player_transform.translation.truncate();
                        let target_position = world_pos - player_pos;
                        let bullet_direction = target_position.normalize();
                        let bullet = commands
                            .spawn_bundle(SpriteBundle {
                                texture: my_assets.arrow.clone(),
                                ..Default::default()
                            })
                            .insert(Bullet)
                            .insert(RigidBody::KinematicVelocityBased)
                            .insert(Collider::ball(6.))
                            .insert(Ccd::enabled())
                            .insert(Sensor)
                            .insert(Velocity {
                                linvel: bullet_direction * 700.,
                                ..Default::default()
                            })
                            .id();

                        commands.entity(player_entity).add_child(bullet);
                    }
                }
            }
        } else {
            // cursor is not inside the window
        }
}

fn player_arrow(
    player_query: Query<(&Player, &Transform, Entity)>,
    mut commands: Commands,
    my_assets: Res<MyAssets>,
    q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
    windows: Res<Windows>,
    mut arrow_query: Query<(&Arrow, &mut Transform), Without<Player>>,
) {
        let window = windows.get_primary().unwrap();
        match arrow_query.get_single_mut() {
            Ok((_arrow, mut arrow_transform)) => {
                for (_player, player_transform, _player_entity) in player_query.iter() {
                    if let Some(mouse_position) = window.cursor_position() {
                        if let Ok((camera, camera_transform)) = q_camera.get_single() {
                            let window_size =
                                Vec2::new(window.width() as f32, window.height() as f32);
                            let ndc = (mouse_position / window_size) * 2.0 - Vec2::ONE;
                            let ndc_to_world = camera_transform.compute_matrix()
                                * camera.projection_matrix.inverse();
                            let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));
                            let world_pos: Vec2 = world_pos.truncate();
                            let player_pos = player_transform.translation.truncate();
                            let target_position = world_pos - player_pos;
                            let arrow_direction = target_position.normalize().extend(55.) * 11.;
                            arrow_transform.translation = arrow_direction;
                        }
                    }
                }
            }
            _ => {
                for (_player, _player_transform, player_entity) in player_query.iter() {
                    let arrow = commands
                        .spawn_bundle(SpriteBundle {
                            sprite: Sprite {
                                custom_size: Some(Vec2::splat(3.)),
                                ..Default::default()
                            },
                            texture: my_assets.arrow.clone(),
                            ..Default::default()
                        })
                        .insert(Arrow)
                        .id();
                    commands.entity(player_entity).add_child(arrow);
                }
            }
        }
}

fn hit(
    mut commands: Commands,
    mut enemy_query: Query<(&mut Enemy, Entity)>,
    bullet_query: Query<Entity, With<Bullet>>,
    rapier_context: Res<RapierContext>,
) {
    for (mut enemy, enemy_e) in enemy_query.iter_mut() {
        for bullet_e in bullet_query.iter() {
            if rapier_context.intersection_pair(enemy_e, bullet_e) == Some(true) {
                enemy.hp -= 5.;
                commands.entity(bullet_e).despawn();
            }
        }
    }
}
