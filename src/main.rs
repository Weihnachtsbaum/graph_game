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
            MeshPickingPlugin,
            Material2dPlugin::<VertexMaterial>::default(),
        ))
        .add_systems(Startup, setup)
        .run()
}

#[derive(Component)]
struct Selected;

#[derive(Component)]
struct Vertex;

impl Vertex {
    fn spawn(
        self,
        pos: Vec2,
        mut commands: Commands,
        mut meshes: Mut<Assets<Mesh>>,
        mut materials: Mut<Assets<VertexMaterial>>,
    ) {
        commands
            .spawn((
                Vertex,
                Mesh2d(meshes.add(Circle::new(25.0))),
                MeshMaterial2d(materials.add(VertexMaterial {})),
                Text2d::new("1"),
                TextFont {
                    font_size: 35.0,
                    ..default()
                },
                TextColor(Color::BLACK),
                Transform::from_translation(pos.extend(0.0)),
            ))
            .observe(handle_vertex_click);
    }
}

#[derive(AsBindGroup, Debug, Clone, Asset, TypePath)]
struct VertexMaterial {}

impl Material2d for VertexMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/vertex.wgsl".into()
    }
}

#[derive(Component)]
struct Connection;

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VertexMaterial>>,
) {
    commands.spawn(Camera2d);
    Vertex.spawn(
        Vec2::new(-150.0, -25.0),
        commands.reborrow(),
        meshes.reborrow(),
        materials.reborrow(),
    );
    Vertex.spawn(
        Vec2::new(100.0, 50.0),
        commands.reborrow(),
        meshes.reborrow(),
        materials.reborrow(),
    );
}

fn handle_vertex_click(
    trigger: Trigger<Pointer<Click>>,
    selected_q: Query<(Entity, &Transform), With<Selected>>,
    transform_q: Query<&Transform, With<Vertex>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let Ok((selected_entity, selected_transform)) = selected_q.get_single() else {
        commands.entity(trigger.entity()).insert(Selected);
        return;
    };
    commands.entity(selected_entity).remove::<Selected>();
    let Ok(transform) = transform_q.get(trigger.entity()) else {
        return;
    };
    let dist = selected_transform
        .translation
        .xy()
        .distance(transform.translation.xy());
    commands
        .spawn((
            Connection,
            Mesh2d(meshes.add(Rectangle::new(dist, 5.0))),
            MeshMaterial2d(materials.add(Color::WHITE)),
            Transform {
                translation: (selected_transform.translation + transform.translation) / 2.0,
                rotation: {
                    let diff = transform.translation - selected_transform.translation;
                    Quat::from_rotation_z((diff.y / diff.x).asin())
                },
                ..default()
            },
        ))
        .observe(handle_connection_click);
}

fn handle_connection_click(trigger: Trigger<Pointer<Click>>, mut commands: Commands) {
    commands.entity(trigger.entity()).despawn();
}
