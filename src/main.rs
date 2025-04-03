#![cfg_attr(not(feature = "console"), windows_subsystem = "windows")]

use bevy::prelude::*;

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
            level::plugin,
            vertex::plugin,
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .run()
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
}
