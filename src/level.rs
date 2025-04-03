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
        Text::new("Level 1"),
        Node {
            justify_self: JustifySelf::Center,
            ..default()
        },
        TextFont {
            font_size: 30.0,
            ..default()
        },
    ));
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
        Vertex::new(vertex.0 as usize, generate_pos(&mut rng)).spawn(
            commands.reborrow(),
            meshes.reborrow(),
            materials.reborrow(),
        );
    }
}

fn generate_pos(rng: &mut impl Rng) -> Vec2 {
    Vec2::new(rng.gen_range(-250.0..250.0), rng.gen_range(-250.0..250.0))
}

fn check_if_solved(
    vertex_q: Query<(Entity, &Vertex)>,
    edge_q: Query<Entity, With<Edge>>,
    mut level_text_q: Query<&mut Text, With<LevelText>>,
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
