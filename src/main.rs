#![cfg_attr(not(feature = "console"), windows_subsystem = "windows")]

use bevy::{core_pipeline::bloom::Bloom, prelude::*, render::camera::ScalingMode};

mod audio;
mod edge;
mod level;
mod pause;
mod vertex;

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins,
            MeshPickingPlugin,
            audio::plugin,
            edge::plugin,
            level::plugin,
            pause::plugin,
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
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::AutoMin {
                min_width: 1440.0,
                min_height: 1440.0,
            },
            ..OrthographicProjection::default_2d()
        }),
        Camera {
            hdr: true,
            ..default()
        },
        Bloom::NATURAL,
    ));
}

#[derive(States, Default, Clone, Debug, PartialEq, Eq, Hash)]
#[states(scoped_entities)]
enum GameState {
    Playing,
    Paused,
    LevelSelect,
    Settings,
    #[default]
    LevelEnter,
    LevelExit,
}
