use bevy::prelude::*;
use bevy_inspector_egui::{RegisterInspectable, WorldInspectorPlugin};
// use bevy_rapier2d::prelude::*;

use crate::plugins::player::Player;

use super::enemy::Enemy;

// use super::{
//     inventory::Inventory,
//     items::{Pickupable, WorldObject},
// };

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        if cfg!(debug_assertions) {
            app.add_plugin(WorldInspectorPlugin::new())
                .register_inspectable::<Player>()
                .register_inspectable::<Enemy>();
        }
    }
}
