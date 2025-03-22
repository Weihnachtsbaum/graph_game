#![cfg_attr(not(feature = "console"), windows_subsystem = "windows")]

use bevy::{
    ecs::system::SystemId,
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin},
    utils::HashSet,
};

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins,
            MeshPickingPlugin,
            Material2dPlugin::<VertexMaterial>::default(),
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .add_systems(Startup, setup)
        .run()
}

#[derive(Resource)]
struct CheckIfSolvedSystem(SystemId);

#[derive(Component)]
struct Selected;

#[derive(Component)]
struct Vertex {
    edges: HashSet<Entity>,
    required_edges: usize,
}

impl Vertex {
    fn new(required_edges: usize) -> Self {
        Self {
            edges: HashSet::new(),
            required_edges,
        }
    }

    fn spawn(
        self,
        pos: Vec2,
        mut commands: Commands,
        mut meshes: Mut<Assets<Mesh>>,
        mut materials: Mut<Assets<VertexMaterial>>,
    ) {
        let required = self.required_edges;
        commands
            .spawn((
                self,
                Mesh2d(meshes.add(Circle::new(25.0))),
                MeshMaterial2d(materials.add(VertexMaterial { selected: 0 })),
                Text2d(format!("{}", required)),
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
struct VertexMaterial {
    #[uniform(0)]
    selected: u32,
}

impl Material2d for VertexMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/vertex.wgsl".into()
    }
}

#[derive(Component)]
struct Edge(Entity, Entity);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VertexMaterial>>,
) {
    commands.spawn(Camera2d);
    let id = commands.register_system(check_if_solved);
    commands.insert_resource(CheckIfSolvedSystem(id));
    Vertex::new(1).spawn(
        Vec2::new(-150.0, -25.0),
        commands.reborrow(),
        meshes.reborrow(),
        materials.reborrow(),
    );
    Vertex::new(1).spawn(
        Vec2::new(100.0, 50.0),
        commands.reborrow(),
        meshes.reborrow(),
        materials.reborrow(),
    );
}

#[allow(clippy::too_many_arguments)]
fn handle_vertex_click(
    trigger: Trigger<Pointer<Click>>,
    mut selected_q: Query<(Entity, &mut Vertex, &Transform), With<Selected>>,
    mut vertex_q: Query<(Entity, &mut Vertex, &Transform), Without<Selected>>,
    mesh_material_q: Query<&MeshMaterial2d<VertexMaterial>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    mut vertex_materials: ResMut<Assets<VertexMaterial>>,
    check_if_solved_system: Res<CheckIfSolvedSystem>,
) {
    let Ok((selected_entity, mut selected_vertex, selected_transform)) =
        selected_q.get_single_mut()
    else {
        commands.entity(trigger.entity()).insert(Selected);
        let Ok(handle) = mesh_material_q.get(trigger.entity()) else {
            return;
        };
        let Some(material) = vertex_materials.get_mut(handle) else {
            return;
        };
        material.selected = 1;
        return;
    };

    commands.entity(selected_entity).remove::<Selected>();
    let Ok(handle) = mesh_material_q.get(selected_entity) else {
        return;
    };
    let Some(material) = vertex_materials.get_mut(handle) else {
        return;
    };
    material.selected = 0;

    let Ok((entity, mut vertex, transform)) = vertex_q.get_mut(trigger.entity()) else {
        return;
    };
    if selected_vertex.edges.contains(&entity) {
        // Edge already exists.
        return;
    }
    let dist = selected_transform
        .translation
        .xy()
        .distance(transform.translation.xy());
    commands
        .spawn((
            Edge(selected_entity, entity),
            Mesh2d(meshes.add(Rectangle::new(dist, 5.0))),
            MeshMaterial2d(color_materials.add(Color::WHITE)),
            Transform {
                translation: (selected_transform.translation + transform.translation) / 2.0,
                rotation: {
                    let diff = transform.translation - selected_transform.translation;
                    Quat::from_rotation_z((diff.y / diff.x).asin())
                },
                ..default()
            },
        ))
        .observe(handle_edge_click);
    vertex.edges.insert(selected_entity);
    selected_vertex.edges.insert(entity);
    commands.run_system(check_if_solved_system.0);
}

fn handle_edge_click(
    trigger: Trigger<Pointer<Click>>,
    edge_q: Query<&Edge>,
    mut vertex_q: Query<&mut Vertex>,
    mut commands: Commands,
    check_if_solved_system: Res<CheckIfSolvedSystem>,
) {
    let Ok(edge) = edge_q.get(trigger.entity()) else {
        return;
    };
    if let Ok(mut vertex) = vertex_q.get_mut(edge.0) {
        vertex.edges.remove(&edge.1);
    }
    if let Ok(mut vertex) = vertex_q.get_mut(edge.1) {
        vertex.edges.remove(&edge.0);
    }
    commands.entity(trigger.entity()).despawn();
    commands.run_system(check_if_solved_system.0);
}

fn check_if_solved(vertex_q: Query<&Vertex>) {
    let solved = vertex_q
        .iter()
        .all(|vertex| vertex.edges.len() == vertex.required_edges);
    info!("Solved: {}", solved);
}
