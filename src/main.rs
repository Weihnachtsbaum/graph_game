#![cfg_attr(not(feature = "console"), windows_subsystem = "windows")]

use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin},
};

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins,
            Material2dPlugin::<VertexMaterial>::default(),
        ))
        .add_systems(Startup, setup)
        .run()
}

#[derive(Component)]
struct Vertex;

#[derive(AsBindGroup, Debug, Clone, Asset, TypePath)]
struct VertexMaterial {}

impl Material2d for VertexMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/vertex.wgsl".into()
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VertexMaterial>>,
) {
    commands.spawn(Camera2d);
    commands.spawn((
        Vertex,
        Mesh2d(meshes.add(Circle::new(25.0))),
        MeshMaterial2d(materials.add(VertexMaterial {})),
        Text2d::new("1"),
        TextFont {
            font_size: 35.0,
            ..default()
        },
        TextColor(Color::BLACK),
    ));
}
