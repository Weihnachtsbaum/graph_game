use std::f32::consts::PI;

use bevy::{ecs::system::SystemId, prelude::*};
use rand::{Rng, SeedableRng, rngs::StdRng};

use crate::{
    audio::BeatLevelAudioHandle,
    edge::Edge,
    vertex::{Vertex, VertexMaterial},
};

pub fn plugin(app: &mut App) {
    app.insert_resource(Level(1))
        .add_systems(Startup, (setup, generate_level));
}

#[derive(Resource)]
pub struct CheckIfSolvedSystem(pub SystemId);

#[derive(Resource)]
struct GenerateLevelSystem(SystemId);

#[derive(Resource)]
struct Level(u64);

#[derive(Component)]
struct LevelText;

fn setup(mut commands: Commands) {
    let id = commands.register_system(check_if_solved);
    commands.insert_resource(CheckIfSolvedSystem(id));

    let id = commands.register_system(generate_level);
    commands.insert_resource(GenerateLevelSystem(id));

    commands.spawn((
        LevelText,
        Text2d::new("Level 1"),
        TextFont {
            font_size: 60.0,
            ..default()
        },
        Transform::from_xyz(0.0, 690.0, -2.0),
    ));
}

fn generate_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<VertexMaterial>>,
    level: Res<Level>,
) {
    let mut rng = StdRng::seed_from_u64(level.0);

    let vertex_count =
        rng.gen_range((1 + level.0 as usize).min(4)..=(1 + level.0 as usize).min(25));
    let mut positions = Vec::with_capacity(vertex_count);
    positions.push(Vec2::ZERO);

    for _ in 0..vertex_count {
        positions.push(
            (positions[rng.gen_range(0..positions.len())]
                + rng.gen_range(Vertex::RADIUS * 2.0..Edge::MAX_LEN + Vertex::RADIUS * 2.0)
                    * Vec2::from_angle(rng.gen_range(-PI..PI)))
            .clamp(Vec2::new(-620.0, -620.0), Vec2::new(620.0, 620.0)),
        );
    }

    let mut required_edges = vec![0; vertex_count];
    const EDGE_PROBABILITY: f32 = 0.3;

    for (i1, pos1) in positions.iter().enumerate() {
        for i2 in i1 + 1..vertex_count {
            if pos1.distance_squared(positions[i2])
                < (Edge::MAX_LEN + Vertex::RADIUS * 2.0) * (Edge::MAX_LEN + Vertex::RADIUS * 2.0)
                && (required_edges[i1] == 0 || rng.r#gen::<f32>() < EDGE_PROBABILITY)
            {
                required_edges[i1] += 1;
                required_edges[i2] += 1;
            }
        }
    }

    for (i, (pos, required_edges)) in positions.iter().zip(required_edges).enumerate() {
        if required_edges == 0 {
            continue;
        }
        Vertex::new(required_edges, *pos).spawn(
            i as f32 / vertex_count as f32,
            commands.reborrow(),
            meshes.reborrow(),
            materials.reborrow(),
        );
    }
}

fn check_if_solved(
    vertex_q: Query<(Entity, &Vertex)>,
    edge_q: Query<Entity, With<Edge>>,
    mut level_text_q: Query<&mut Text2d, With<LevelText>>,
    mut level: ResMut<Level>,
    generate_level_system: Res<GenerateLevelSystem>,
    mut commands: Commands,
    beat_level_audio: Res<BeatLevelAudioHandle>,
) {
    let solved = vertex_q
        .iter()
        .all(|(_, vertex)| vertex.edges.len() == vertex.required_edges);
    if solved {
        for (entity, _) in &vertex_q {
            commands.entity(entity).despawn_recursive();
        }
        for entity in &edge_q {
            commands.entity(entity).despawn();
        }
        level.0 += 1;
        if let Ok(mut level_text) = level_text_q.get_single_mut() {
            level_text.0 = format!("Level {}", level.0);
        }
        commands.spawn((
            AudioPlayer(beat_level_audio.0.clone()),
            PlaybackSettings::DESPAWN,
        ));
        commands.run_system(generate_level_system.0);
    }
}
