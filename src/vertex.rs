use bevy::{
    prelude::*,
    render::render_resource::{AsBindGroup, ShaderRef},
    sprite::{Material2d, Material2dPlugin},
    utils::HashSet,
};

use crate::{
    Wall,
    audio::{PlaceAudioHandle, SelectAudioHandle},
    edge::{Edge, get_obstacle_pos, handle_edge_click},
    level::CheckIfSolvedSystem,
};

pub fn plugin(app: &mut App) {
    app.add_plugins(Material2dPlugin::<VertexMaterial>::default());
}

#[derive(Component)]
pub struct Selected {
    pub edge: Entity,
}

#[derive(Component)]
pub struct Vertex {
    pub edges: HashSet<Entity>,
    pub required_edges: usize,
    start_pos: Vec2,
}

impl Vertex {
    pub const RADIUS: f32 = 50.0;

    pub fn new(required_edges: usize, start_pos: Vec2) -> Self {
        Self {
            edges: HashSet::new(),
            required_edges,
            start_pos,
        }
    }

    pub fn spawn(
        self,
        z: f32,
        mut commands: Commands,
        mut meshes: Mut<Assets<Mesh>>,
        mut materials: Mut<Assets<VertexMaterial>>,
    ) {
        let required = self.required_edges;
        let pos = self.start_pos.extend(z);
        commands
            .spawn((
                self,
                Mesh2d(meshes.add(Circle::new(Self::RADIUS))),
                MeshMaterial2d(materials.add(VertexMaterial { bits: 0 })),
                Transform::from_translation(pos),
            ))
            .with_child((
                Text2d(format!("{}", required)),
                TextFont {
                    font_size: 70.0,
                    ..default()
                },
            ))
            .observe(handle_vertex_click)
            .observe(handle_vertex_drag);
    }
}

#[derive(AsBindGroup, Debug, Clone, Asset, TypePath)]
pub struct VertexMaterial {
    /// 1 << 0: selected
    /// 1 << 1: solved
    #[uniform(0)]
    bits: u32,
}

impl Material2d for VertexMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/vertex.wgsl".into()
    }
}

impl VertexMaterial {
    fn set_selected(&mut self, v: bool) {
        if v {
            self.bits |= 1;
        } else {
            self.bits &= !1;
        }
    }

    pub fn set_solved(&mut self, v: bool, text_color: &mut TextColor) {
        if v {
            self.bits |= 2;
            text_color.0 = Color::BLACK;
        } else {
            self.bits &= !2;
            text_color.0 = Color::WHITE;
        }
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn handle_vertex_click(
    trigger: Trigger<Pointer<Click>>,
    mut selected_q: Query<
        (Entity, &mut Vertex, &mut Transform, &Children, &Selected),
        Without<Edge>,
    >,
    mut vertex_q: Query<
        (Entity, &mut Vertex, &mut Transform, &Children),
        (Without<Selected>, Without<Edge>),
    >,
    wall_q: Query<&Transform, (With<Wall>, Without<Vertex>)>,
    mesh_material_q: Query<&MeshMaterial2d<VertexMaterial>>,
    mut text_color_q: Query<&mut TextColor>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    mut vertex_materials: ResMut<Assets<VertexMaterial>>,
    select_audio: Res<SelectAudioHandle>,
    place_audio: Res<PlaceAudioHandle>,
    check_if_solved_system: Res<CheckIfSolvedSystem>,
) {
    let Some(pointer_pos) = trigger.event().hit.position else {
        return;
    };
    let Ok((
        selected_entity,
        mut selected_vertex,
        mut selected_transform,
        selected_children,
        selected,
    )) = selected_q.get_single_mut()
    else {
        let Ok(handle) = mesh_material_q.get(trigger.entity()) else {
            return;
        };
        let Some(material) = vertex_materials.get_mut(handle) else {
            return;
        };
        material.set_selected(true);

        let Ok((entity, _, mut transform, _)) = vertex_q.get_mut(trigger.entity()) else {
            return;
        };
        transform.translation.z += 1.0;

        let dist = transform.translation.xy().distance(pointer_pos.xy());
        let edge = commands
            .spawn((
                Edge(entity, Entity::PLACEHOLDER),
                Mesh2d(meshes.add(Rectangle::new(dist, Edge::WIDTH))),
                MeshMaterial2d(color_materials.add(Color::WHITE)),
                Transform {
                    translation: ((transform.translation.xy() + pointer_pos.xy()) / 2.0)
                        .extend(-1.0),
                    rotation: {
                        let diff = transform.translation - pointer_pos;
                        Quat::from_rotation_z(diff.y.atan2(diff.x))
                    },
                    ..default()
                },
                AudioPlayer(select_audio.0.clone()),
                PlaybackSettings::REMOVE,
            ))
            .id();

        commands.entity(trigger.entity()).insert(Selected { edge });
        return;
    };

    commands.entity(selected_entity).remove::<Selected>();
    let Ok(handle) = mesh_material_q.get(selected_entity) else {
        return;
    };
    let Some(selected_material) = vertex_materials.get_mut(handle) else {
        return;
    };
    selected_material.set_selected(false);
    selected_transform.translation.z -= 1.0;

    commands.entity(selected.edge).despawn();

    if get_obstacle_pos(
        selected_transform.translation.xy(),
        pointer_pos.xy(),
        vertex_q.iter().filter_map(|(e, _, transform, _)| {
            if e == trigger.entity() {
                None
            } else {
                Some(transform)
            }
        }),
        wall_q.iter(),
    ) != pointer_pos.xy()
    {
        return;
    }

    let Ok((entity, mut vertex, transform, children)) = vertex_q.get_mut(trigger.entity()) else {
        // Unselect vertex.
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

    if dist > Edge::MAX_LEN + Vertex::RADIUS * 2.0 {
        return;
    }
    // Despawning `selected.edge` and spawning new edge to avoid bug with removing edges.
    // See bug in commit f650d38.
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

    selected_vertex.edges.insert(entity);
    let Ok(mut text_color) = text_color_q.get_mut(selected_children[0]) else {
        return;
    };
    selected_material.set_solved(
        selected_vertex.edges.len() == selected_vertex.required_edges,
        &mut text_color,
    );

    vertex.edges.insert(selected_entity);
    let Ok(handle) = mesh_material_q.get(entity) else {
        return;
    };
    let Some(material) = vertex_materials.get_mut(handle) else {
        return;
    };
    let Ok(mut text_color) = text_color_q.get_mut(children[0]) else {
        return;
    };
    material.set_solved(vertex.edges.len() == vertex.required_edges, &mut text_color);

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
