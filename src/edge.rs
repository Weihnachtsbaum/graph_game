use bevy::prelude::*;

use crate::{level::CheckIfSolvedSystem, vertex::Vertex};

#[derive(Component)]
pub struct Edge(pub Entity, pub Entity);

impl Edge {
    pub const WIDTH: f32 = 10.0;
}

pub fn handle_edge_click(
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
