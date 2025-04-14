use bevy::prelude::*;

use crate::{
    level::CheckIfSolvedSystem,
    vertex::{Vertex, VertexMaterial},
};

#[derive(Component)]
pub struct Edge(pub Entity, pub Entity);

impl Edge {
    pub const WIDTH: f32 = 10.0;
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
