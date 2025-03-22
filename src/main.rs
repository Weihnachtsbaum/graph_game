#![cfg_attr(not(feature = "console"), windows_subsystem = "windows")]

use bevy::{
    ecs::system::SystemId,
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin},
    utils::HashSet,
};
use rand::{Rng, SeedableRng, rngs::StdRng};

fn main() -> AppExit {
    App::new()
        .add_plugins((
            DefaultPlugins,
            MeshPickingPlugin,
            Material2dPlugin::<VertexMaterial>::default(),
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Level(1))
        .add_systems(Startup, (setup, generate_level))
        .run()
}

#[derive(Resource)]
struct CheckIfSolvedSystem(SystemId);

#[derive(Resource)]
struct GenerateLevelSystem(SystemId);

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

#[derive(Resource)]
struct Level(u64);

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);

    let id = commands.register_system(check_if_solved);
    commands.insert_resource(CheckIfSolvedSystem(id));

    let id = commands.register_system(generate_level);
    commands.insert_resource(GenerateLevelSystem(id));
}

fn generate_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VertexMaterial>>,
    level: Res<Level>,
) {
    let mut rng = StdRng::seed_from_u64(level.0);

    let mut vertices: Vec<(u8, u8)> = vec![];

    loop {
        let degree = rng.gen_range(1..=4);
        let mut current = 0;

        for vertex in &mut vertices {
            if current == degree {
                break;
            }
            if vertex.0 > vertex.1 {
                vertex.1 += 1;
                current += 1;
            }
        }

        vertices.push((degree, current));

        if degree == current && vertices.iter().all(|v| v.0 == v.1) {
            break;
        }
    }

    for vertex in vertices {
        Vertex::new(vertex.0 as usize).spawn(
            generate_pos(&mut rng),
            commands.reborrow(),
            meshes.reborrow(),
            materials.reborrow(),
        );
    }
}

fn generate_pos(rng: &mut impl Rng) -> Vec2 {
    Vec2::new(rng.gen_range(-250.0..250.0), rng.gen_range(-250.0..250.0))
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
                    Quat::from_rotation_z(diff.y.atan2(diff.x))
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

fn check_if_solved(
    vertex_q: Query<(Entity, &Vertex)>,
    edge_q: Query<Entity, With<Edge>>,
    mut level: ResMut<Level>,
    generate_level_system: Res<GenerateLevelSystem>,
    mut commands: Commands,
) {
    let solved = vertex_q
        .iter()
        .all(|(_, vertex)| vertex.edges.len() == vertex.required_edges);
    if solved {
        for (entity, _) in &vertex_q {
            commands.entity(entity).despawn();
        }
        for entity in &edge_q {
            commands.entity(entity).despawn();
        }
        level.0 += 1;
        commands.run_system(generate_level_system.0);
    }
}
