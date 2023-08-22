use bevy::{
    math::Vec3Swizzles, prelude::*,
    render::camera::ScalingMode,
};
use bevy_asset_loader::prelude::*;
use bevy_ggrs::{ggrs::PlayerType, *};
use bevy_matchbox::prelude::*;
use bevy_prototype_lyon::prelude::ShapePlugin;
use bevy_rapier2d::{
    prelude::{
        NoUserData, RapierPhysicsPlugin, Sleeping, Velocity,
    },
    render::RapierDebugRenderPlugin,
};
use leafwing_input_manager::prelude::InputManagerPlugin;
use ping::components::*;
use ping::input::*;
use ping::*;

fn main() {
    App::new()
        .add_state::<GameState>()
        .add_loading_state(
            LoadingState::new(GameState::AssetLoading)
                .continue_to_state(GameState::Matchmaking),
        )
        // .add_collection_to_loading_state::<_, ImageAssets>(
        // GameState::AssetLoading,
        // )


        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                // fill the entire browser window
                fit_canvas_to_parent: true,
                ..default()
            }),
            ..default()
        }),
        RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(1.0),
        // RapierDebugRenderPlugin::default(),
        ShapePlugin,
        InputManagerPlugin::<PlayerAction>::default()
        ))
        .add_ggrs_plugin(
            GgrsPlugin::<GgrsConfig>::new()
                .with_input_system(input)
                .register_rollback_component::<GlobalTransform>()
                .register_rollback_component::<Transform>()
                .register_rollback_component::<Velocity>()
                .register_rollback_component::<Sleeping>()
        )
        .insert_resource(ClearColor(Color::rgb(
            0.53, 0.53, 0.53,
        )))
        .add_systems(
            OnEnter(GameState::Matchmaking),
            (setup, start_matchbox_socket),
        )
        .add_systems(
            OnEnter(GameState::InGame),
            (
                spawn_players,
                spawn_playing_area,
                spawn_ball
            ),
        )
        .add_systems(
            Update,
            (wait_for_players
                .run_if(in_state(GameState::Matchmaking)),),
        )
        .add_systems(
            GgrsSchedule,
            (
                move_players,
                gizmos
                // move_ball
            ),
        )
        .run();
}
