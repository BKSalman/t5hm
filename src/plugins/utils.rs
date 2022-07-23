use bevy::prelude::*;
use std::f32::consts::FRAC_PI_2;


pub fn to_world_coordinates(
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

pub fn look_at(
    target_position: Vec2,
) -> Quat {
    let diff = target_position;
    let angle = diff.y.atan2(diff.x) - FRAC_PI_2; // Add/sub FRAC_PI here optionally
    Quat::from_axis_angle(Vec3::new(0., 0., 1.), angle)
}