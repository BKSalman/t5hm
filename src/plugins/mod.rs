use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

pub mod debug;
pub mod enemy;
pub mod player;
pub mod tilemap;

#[derive(Clone, Default, Bundle, LdtkIntCell)]
pub struct ColliderBundle {
    pub name: Name,
    pub collider: Collider,
    pub rigid_body: RigidBody,
    pub velocity: Velocity,
    pub rotation_constraints: LockedAxes,
    pub gravity: GravityScale,
}

impl From<EntityInstance> for ColliderBundle {
    fn from(entity_instance: EntityInstance) -> ColliderBundle {
        let rotation_constraints = LockedAxes::ROTATION_LOCKED;

        match entity_instance.identifier.as_ref() {
            "Player" => ColliderBundle {
                name: Name::new("Player"),
                collider: Collider::cuboid(7., 7.),
                rigid_body: RigidBody::Dynamic,
                rotation_constraints,
                gravity: GravityScale(0.),
                ..Default::default()
            },
            "Enemy" => ColliderBundle {
                name: Name::new("Enemy"),
                collider: Collider::cuboid(7., 7.),
                rigid_body: RigidBody::Dynamic,
                rotation_constraints,
                gravity: GravityScale(0.),
                ..Default::default()
            },
            _ => ColliderBundle::default(),
        }
    }
}

impl From<IntGridCell> for ColliderBundle {
    fn from(int_grid_cell: IntGridCell) -> ColliderBundle {
        let rotation_constraints = LockedAxes::ROTATION_LOCKED;

        if int_grid_cell.value == 1 {
            ColliderBundle {
                collider: Collider::cuboid(8., 8.),
                rotation_constraints,
                ..Default::default()
            }
        } else {
            ColliderBundle::default()
        }
    }
}
