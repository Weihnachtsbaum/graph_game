use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin},
    utils::HashSet,
};

use crate::{
    audio::{PlaceAudioHandle, SelectAudioHandle},
    edge::{Edge, handle_edge_click},
    level::CheckIfSolvedSystem,
};

pub fn plugin(app: &mut App) {
    app.add_plugins(Material2dPlugin::<VertexMaterial>::default());
}

#[derive(Component)]
struct Selected;

#[derive(Component)]
pub struct Vertex {
    pub edges: HashSet<Entity>,
    pub required_edges: usize,
    start_pos: Vec2,
}

impl Vertex {
    pub fn new(required_edges: usize, start_pos: Vec2) -> Self {
        Self {
            edges: HashSet::new(),
            required_edges,
            start_pos,
        }
    }

    pub fn spawn(
        self,
        mut commands: Commands,
        mut meshes: Mut<Assets<Mesh>>,
        mut materials: Mut<Assets<VertexMaterial>>,
    ) {
        let required = self.required_edges;
        let pos = self.start_pos.extend(0.0);
        commands
            .spawn((
                self,
                Mesh2d(meshes.add(Circle::new(50.0))),
                MeshMaterial2d(materials.add(VertexMaterial { selected: 0 })),
                Transform::from_translation(pos),
            ))
            .with_child((
                Text2d(format!("{}", required)),
                TextFont {
                    font_size: 70.0,
                    ..default()
                },
                TextColor(Color::BLACK),
            ))
            .observe(handle_vertex_click)
            .observe(handle_vertex_drag);
    }
}

#[derive(AsBindGroup, Debug, Clone, Asset, TypePath)]
pub struct VertexMaterial {
    #[uniform(0)]
    selected: u32,
}

impl Material2d for VertexMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/vertex.wgsl".into()
    }
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
    select_audio: Res<SelectAudioHandle>,
    place_audio: Res<PlaceAudioHandle>,
    check_if_solved_system: Res<CheckIfSolvedSystem>,
) {
    let Ok((selected_entity, mut selected_vertex, selected_transform)) =
        selected_q.get_single_mut()
    else {
        commands.entity(trigger.entity()).insert((
            Selected,
            AudioPlayer(select_audio.0.clone()),
            PlaybackSettings::REMOVE,
        ));
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
            Mesh2d(meshes.add(Rectangle::new(dist, Edge::WIDTH))),
            MeshMaterial2d(color_materials.add(Color::WHITE)),
            Transform {
                translation: ((selected_transform.translation.xy() + transform.translation.xy())
                    / 2.0)
                    .extend(-1.0),
                rotation: {
                    let diff = transform.translation - selected_transform.translation;
                    Quat::from_rotation_z(diff.y.atan2(diff.x))
                },
                ..default()
            },
            AudioPlayer(place_audio.0.clone()),
            PlaybackSettings::REMOVE,
        ))
        .observe(handle_edge_click);
    vertex.edges.insert(selected_entity);
    selected_vertex.edges.insert(entity);
    commands.run_system(check_if_solved_system.0);
}

fn handle_vertex_drag(
    trigger: Trigger<Pointer<Drag>>,
    mut vertex_q: Query<(&Vertex, &mut Transform)>,
    mut edge_q: Query<(&Edge, &mut Transform, &Mesh2d), Without<Vertex>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let entity = trigger.entity();
    let Ok((vertex, transform)) = vertex_q.get(entity) else {
        return;
    };

    let delta = trigger.event().delta / 2.0;
    let new_pos = Vec2::new(
        transform.translation.x + delta.x,
        transform.translation.y - delta.y,
    );
    if new_pos.distance_squared(vertex.start_pos) > 10000.0 {
        return;
    }

    if !vertex.edges.is_empty() {
        for (edge, mut edge_transform, mesh2d) in &mut edge_q {
            let other = if edge.0 == entity {
                edge.1
            } else if edge.1 == entity {
                edge.0
            } else {
                continue;
            };

            let Some(mesh) = meshes.get_mut(mesh2d) else {
                return;
            };
            let Ok((_, other_transform)) = vertex_q.get(other) else {
                return;
            };
            let other_pos = other_transform.translation.xy();
            let dist = new_pos.distance(other_pos);
            *mesh = Rectangle::new(dist, Edge::WIDTH).into();

            edge_transform.translation = ((new_pos + other_pos) / 2.0).extend(-1.0);
            let diff = new_pos - other_pos;
            edge_transform.rotation = Quat::from_rotation_z(diff.y.atan2(diff.x));
        }
    }

    let (_, mut transform) = vertex_q.get_mut(entity).unwrap();
    transform.translation.x = new_pos.x;
    transform.translation.y = new_pos.y;
}
