use std::f32::consts::FRAC_PI_2;
use std::time::Duration;
use bevy::{prelude::*, sprite::Anchor};
use bevy_ecs_ldtk::prelude::*;
use bevy_inspector_egui::Inspectable;
use bevy_rapier2d::prelude::*;
use iyes_loopless::prelude::*;

use crate::{GameState, MainCamera, MyAssets};

use super::{enemy::{Enemy, death}, tilemap::WallCollision, ColliderBundle};

#[derive(Default, Debug, Inspectable)]
pub enum Direction {
    #[default]
    None,
    Right,
    Left,
    Up,
    Down,
}

#[derive(Inspectable)]
pub enum Weapon {
    Gun,
    Laser,
}

#[derive(Component, Inspectable)]
pub struct Player {
    pub hp: f32,
    pub velocity: f32,
    pub direction: Direction,
    pub is_moving: bool,
    pub weapon: Weapon
}

impl Default for Player {
    fn default() -> Self {
        Self {
            hp: 10.,
            direction: Direction::Right,
            velocity: 200.,
            is_moving: false,
            weapon: Weapon::Gun
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
pub struct Bullet;

#[derive(Component)]
pub struct Ray;

#[derive(Component)]
pub struct Arrow;

#[derive(Component)]
struct FlashingTimer {
    timer: Timer
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            ConditionSet::new()
                .run_in_state(GameState::Playing)
                .with_system(Self::player_movement)
                .with_system(Self::player_dash)
                .with_system(Self::player_arrow)
                .with_system(Self::player_shoot)
                .with_system(Self::hit)
                .with_system(Self::flashing)
                .into(),
        );
    }
}

impl PlayerPlugin {
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
        mut player_query: Query<(&mut Player ,Entity, &Transform), Without<Enemy>>,
        windows: Res<Windows>,
        my_assets: Res<MyAssets>,
        q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
        mut ray_query: Query<(&mut Transform, Entity, &mut Sprite), (With<Ray>, Without<Player>, Without<Enemy>)>,
        mut enemy_query: Query<(&mut Enemy, Entity)>,
        mut commands: Commands,
        mouse: Res<Input<MouseButton>>,
        keyboard: Res<Input<KeyCode>>,
        rapier_context: Res<RapierContext>,
        time: Res<Time>,
    ) {
        let window = windows.get_primary().unwrap();
        if let Ok((camera, camera_transform)) = q_camera.get_single() {
            if let Some(mouse_position) = window.cursor_position() {
                if let Ok((mut player, player_e, player_transform)) = player_query.get_single_mut() {
                    if keyboard.just_pressed(KeyCode::Key1) {
                        player.weapon = Weapon::Gun
                    }
                    if keyboard.just_pressed(KeyCode::Key2) {
                        player.weapon = Weapon::Laser
                    }
                    match player.weapon {
                        Weapon::Gun =>{
                                if mouse.just_pressed(MouseButton::Left) {
   
                                let world_pos = to_world_coordinates(camera, camera_transform, window, mouse_position);
    
                                let player_pos = player_transform.translation.truncate();
                                let target_position = world_pos.truncate() - player_pos;
                                let bullet_direction = target_position.normalize();
                                commands
                                    .spawn_bundle(SpriteBundle {
                                        texture: my_assets.arrow.clone(),
                                        ..Default::default()
                                    })
                                    .insert(Bullet)
                                    .insert(Transform::from_translation(player_transform.translation.truncate().extend(1.)))
                                    .insert(RigidBody::KinematicVelocityBased)
                                    .insert(Collider::ball(6.))
                                    .insert(Ccd::enabled())
                                    .insert(Sensor)
                                    .insert(Velocity {
                                        linvel: bullet_direction * 700.,
                                        ..Default::default()
                                    });
                                }
                            },
                            Weapon::Laser =>{
                                if mouse.pressed(MouseButton::Left) {
                                let ray_origin = player_transform.translation.truncate();
                                let world_pos = to_world_coordinates(camera, camera_transform, window, mouse_position);
                                let target_position = world_pos.truncate() - ray_origin;
                                let ray_dir = target_position.normalize();
                                let max_toi = 100.0;
                                let solid = true;
                                let filter = QueryFilter{
                                    exclude_collider: Some(player_e),
                                    ..Default::default()
                                };
                                let target_rotation = look_at(target_position);
                                if let Ok((mut ray_transform, _ray_e, mut ray_sprite)) = ray_query.get_single_mut() {
                                    if let Some((entity, _toi)) = rapier_context.cast_ray(
                                        ray_origin, ray_dir, max_toi, solid, filter
                                    ) {
                                        let hit_point = ray_origin + ray_dir * _toi;
                                        
                                        ray_sprite.custom_size = Some(Vec2::new(1., player_transform.translation.truncate().distance(hit_point)));
                                        ray_transform.translation = player_transform.translation.truncate().extend(1.);
                                        ray_transform.rotation = target_rotation;
            
                                        for (mut enemy, enemy_e) in enemy_query.iter_mut() {
                                            if enemy_e == entity {
                                                // The first collider hit has the entity `entity` and it hit after
                                                // the ray travelled a distance equal to `ray_dir * toi`.
                                                enemy.hp -= 10. * time.delta_seconds();
                                                if death(&enemy){
                                                    commands.entity(enemy_e).despawn();
                                                } else {
                                                    commands.entity(enemy_e).insert(FlashingTimer{
                                                        timer: Timer::new(Duration::from_millis(50), true),
                                                    });
                                                }
                                            }
                                        }
            
                                    } else {
                                        ray_sprite.custom_size = Some(Vec2::new(1., max_toi));
                                        ray_transform.translation = player_transform.translation.truncate().extend(1.);
                                        ray_transform.rotation = target_rotation;
                                    }
                                } else {
                                    commands
                                        .spawn_bundle(SpriteBundle{
                                            sprite: Sprite {
                                                anchor: Anchor::BottomCenter,
                                                ..Default::default()
                                            },
                                            texture: my_assets.arrow.clone(),
                                            ..Default::default()
                                        })
                                        .insert(Ray);
                                }
                            }
                            if mouse.just_released(MouseButton::Left) {
                                if let Ok((mut _ray_transform, ray_e, _ray_sprite)) = ray_query.get_single_mut() {
                                    commands.entity(ray_e).despawn_recursive();
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    
    fn player_arrow(
        player_query: Query<(&Transform, Entity), (With<Player>, Without<Enemy>)>,
        mut commands: Commands,
        my_assets: Res<MyAssets>,
        q_camera: Query<(&Camera, &GlobalTransform), With<MainCamera>>,
        windows: Res<Windows>,
        mut arrow_query: Query<&mut Transform, (With<Arrow>, Without<Player>)>,
    ) {
        let window = windows.get_primary().unwrap();
        match arrow_query.get_single_mut() {
            Ok(mut arrow_transform) => {
                for (player_transform, _player_entity) in player_query.iter() {
                    if let Some(mouse_position) = window.cursor_position() {
                        if let Ok((camera, camera_transform)) = q_camera.get_single() {
                            let world_pos = to_world_coordinates(camera, camera_transform, window, mouse_position);
                            let player_pos = player_transform.translation.truncate();
                            let target_position = world_pos.truncate() - player_pos;
                            let arrow_direction = target_position.normalize().extend(55.) * 11.;
                            arrow_transform.translation = arrow_direction;
                        }
                    }
                }
            }
            _ => {
                for (_player_transform, player_entity) in player_query.iter() {
                    let arrow = commands
                        .spawn_bundle(SpriteBundle {
                            sprite: Sprite {
                                custom_size: Some(Vec2::splat(3.)),
                                ..Default::default()
                            },
                            texture: my_assets.arrow.clone(),
                            ..Default::default()
                        })
                        .insert(Name::new("Arrow"))
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
        wall_collision_query: Query<Entity, With<WallCollision>>,
        bullet_query: Query<Entity, With<Bullet>>,
        rapier_context: Res<RapierContext>,
    ) {
        for bullet_e in bullet_query.iter() {
            for (collider1, collider2, _intersecting) in rapier_context.intersections_with(bullet_e) {
                for (mut enemy, enemy_e) in enemy_query.iter_mut() {
                    if collider1 == enemy_e || collider2 == enemy_e {
                        enemy.hp -= 5.;
                        if death(&enemy){
                            commands.entity(enemy_e).despawn();
                        } else {
                            commands.entity(enemy_e).insert(FlashingTimer{
                                timer: Timer::new(Duration::from_millis(50), true),
                            });
                        }
                        commands.entity(bullet_e).despawn_recursive();
                    }
                }
                if wall_collision_query.contains(collider1) || wall_collision_query.contains(collider2)
                {
                    commands.entity(bullet_e).despawn_recursive();
                }
            }
        }
    }
    
    fn flashing (
        mut commands: Commands,
        mut flashing_query: Query<(&mut FlashingTimer, Entity, &mut Sprite)>,
        time: Res<Time>,
    ) {
        for (mut timer, timer_e, mut timer_sprite) in flashing_query.iter_mut() {
            timer_sprite.color = Color::rgba(255., 255., 255., 1.);
                        
            timer.timer.tick(time.delta());
            
            if timer.timer.finished() {
                timer_sprite.color = Color::rgba(1.0, 1.0, 1.0, 1.0);
                commands.entity(timer_e).remove::<FlashingTimer>();
            }
        }
    }
}

fn to_world_coordinates(
    camera: &Camera,
    camera_transform: &GlobalTransform,
    window: &Window, 
    target_position: Vec2
) -> Vec3 {
    let window_size = Vec2::new(window.width() as f32, window.height() as f32);
    let ndc = (target_position / window_size) * 2.0 - Vec2::ONE;
    let ndc_to_world = camera_transform.compute_matrix() * camera.projection_matrix.inverse();
    let world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));
    
    world_pos
}

fn look_at(
    target_position: Vec2,
) -> Quat {
    let diff = target_position;
    let angle = diff.y.atan2(diff.x) - FRAC_PI_2; // Add/sub FRAC_PI here optionally
    Quat::from_axis_angle(Vec3::new(0., 0., 1.), angle)
}