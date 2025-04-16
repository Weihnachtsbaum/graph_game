use bevy::prelude::*;

use crate::{
    level::CheckIfSolvedSystem,
    vertex::{Selected, Vertex, VertexMaterial},
};

pub fn plugin(app: &mut App) {
    app.add_systems(Update, handle_mouse_move);
}

#[derive(Component)]
pub struct Edge(pub Entity, pub Entity);

impl Edge {
    pub const WIDTH: f32 = 10.0;
    pub const MAX_LEN: f32 = 400.0;
}

fn handle_mouse_move(
    mut cursor_evr: EventReader<CursorMoved>,
    mut edge_q: Query<(&mut Transform, &Mesh2d), Without<Vertex>>,
    selected_q: Query<(&Selected, &Transform), With<Vertex>>,
    cam_q: Query<(&Camera, &GlobalTransform)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for ev in cursor_evr.read() {
        let Ok((selected, vertex_transform)) = selected_q.get_single() else {
            return;
        };
        let Ok((mut transform, mesh2d)) = edge_q.get_mut(selected.edge) else {
            return;
        };
        let Ok((cam, cam_transform)) = cam_q.get_single() else {
            return;
        };
        let Ok(pos) = cam.viewport_to_world_2d(cam_transform, ev.position) else {
            return;
        };
        let vertex_pos = vertex_transform.translation.xy();
        let dist = vertex_pos.distance(pos).min(Edge::MAX_LEN + Vertex::RADIUS);
        let Some(mesh) = meshes.get_mut(mesh2d) else {
            return;
        };
        *mesh = Rectangle::new(dist, Edge::WIDTH).into();

        transform.translation = (vertex_pos
            + (pos - vertex_pos).clamp_length_max(Edge::MAX_LEN + Vertex::RADIUS) / 2.0)
            .extend(-1.0);
        let diff = vertex_pos - pos;
        transform.rotation = Quat::from_rotation_z(diff.y.atan2(diff.x));
    }
}

pub fn handle_edge_click(
    trigger: Trigger<Pointer<Click>>,
    edge_q: Query<&Edge>,
    mut vertex_q: Query<(&mut Vertex, &MeshMaterial2d<VertexMaterial>, &Children)>,
    mut text_color_q: Query<&mut TextColor>,
    mut materials: ResMut<Assets<VertexMaterial>>,
    mut commands: Commands,
    check_if_solved_system: Res<CheckIfSolvedSystem>,
) {
    let Ok(edge) = edge_q.get(trigger.entity()) else {
        return;
    };
    if let Ok((mut vertex, handle, children)) = vertex_q.get_mut(edge.0) {
        vertex.edges.remove(&edge.1);
        let Ok(mut text_color) = text_color_q.get_mut(children[0]) else {
            return;
        };
        if let Some(material) = materials.get_mut(handle) {
            material.set_solved(vertex.edges.len() == vertex.required_edges, &mut text_color);
        }
    }
    if let Ok((mut vertex, handle, children)) = vertex_q.get_mut(edge.1) {
        vertex.edges.remove(&edge.0);
        let Ok(mut text_color) = text_color_q.get_mut(children[0]) else {
            return;
        };
        if let Some(material) = materials.get_mut(handle) {
            material.set_solved(vertex.edges.len() == vertex.required_edges, &mut text_color);
        }
    }
    commands.entity(trigger.entity()).despawn();
    commands.run_system(check_if_solved_system.0);
}
