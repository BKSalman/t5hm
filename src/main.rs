use bevy::{prelude::*, window::PresentMode};
use bevy_asset_loader::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;
use iyes_loopless::prelude::*;
use iyes_progress::prelude::*;
use plugins::{
    enemy::{EnemyBundle, EnemyPlugin},
    player::{PlayerBundle, PlayerPlugin},
    tilemap::{TileMapPlugin, WallBundle},
    debug::DebugPlugin,
};

mod plugins;

const HEIGHT: f32 = 640.;
const RESOLUTION: f32 = 16. / 9.;

#[derive(Component)]
pub struct MainCamera;

fn main() {
    let mut app = App::new();
    app.add_loopless_state(GameState::AssetLoading);

    AssetLoader::new(GameState::AssetLoading)
        // https://github.com/NiklasEi/bevy_asset_loader/issues/54
        .continue_to_state(GameState::Playing)
        .with_collection::<MyAssets>()
        .build(&mut app);

    // .add_system_set(SystemSet::on_update(GameState::Playing).with_system(systems::pause_physics_during_load))
    app.insert_resource(WindowDescriptor {
        height: HEIGHT,
        width: HEIGHT * RESOLUTION,
        position: Some(Vec2::new(200., 20.)),
        title: "Letters".into(),
        present_mode: PresentMode::Fifo,
        #[cfg(target_arch = "wasm32")]
        canvas: Some("#bevy-canvas".to_string()),
        resizable: false,
        ..Default::default()
    })
    .add_plugins(DefaultPlugins)
    .add_plugin(ProgressPlugin::new(GameState::AssetLoading))
    .add_plugin(LdtkPlugin)
    .add_plugin(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
    .add_plugin(RapierDebugRenderPlugin::default())
    .add_plugin(DebugPlugin)
    .insert_resource(LdtkSettings {
        level_spawn_behavior: LevelSpawnBehavior::UseWorldTranslation {
            load_level_neighbors: true,
        },
        set_clear_color: SetClearColor::FromLevelBackground,
        ..Default::default()
    })
    .insert_resource(LevelSelection::Uid(0))
    .add_enter_system(GameState::Playing, setup)
    .add_plugin(TileMapPlugin)
    .add_plugin(PlayerPlugin)
    .add_plugin(EnemyPlugin)
    .register_ldtk_int_cell::<WallBundle>(1)
    .register_ldtk_entity::<PlayerBundle>("Player")
    .register_ldtk_entity::<EnemyBundle>("Enemy")
    .run();
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum GameState {
    AssetLoading,
    Playing,
}

#[derive(AssetCollection)]
pub struct MyAssets {
    #[asset(path = "Player.png")]
    pub player: Handle<Image>,
    #[asset(path = "background.png")]
    pub bg: Handle<Image>,
    #[asset(path = "arrow.png")]
    pub arrow: Handle<Image>,
    #[asset(path = "thm_map.ldtk")]
    pub map: Handle<LdtkAsset>,
}

fn setup(mut commands: Commands, my_assets: Res<MyAssets>) {
    let camera = OrthographicCameraBundle::new_2d();
    commands.spawn_bundle(camera).insert(MainCamera);

    commands.spawn_bundle(LdtkWorldBundle {
        ldtk_handle: my_assets.map.clone(),
        ..Default::default()
    });
}
