#![cfg_attr(not(feature = "console"), windows_subsystem = "windows")]

use bevy::{core_pipeline::bloom::Bloom, prelude::*, render::camera::ScalingMode};

mod audio;
mod edge;
mod level;
mod vertex;

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins,
            MeshPickingPlugin,
            audio::plugin,
            edge::plugin,
            level::plugin,
            vertex::plugin,
        ))
        .init_state::<GameState>()
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .run()
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: 1440.0,
                min_height: 1440.0,
            },
            ..OrthographicProjection::default_2d()
        },
        Camera {
            hdr: true,
            ..default()
        },
        Bloom::NATURAL,
    ));
}

#[derive(States, Default, Clone, Debug, PartialEq, Eq, Hash)]
enum GameState {
    #[default]
    Playing,
    LevelTransition,
}
