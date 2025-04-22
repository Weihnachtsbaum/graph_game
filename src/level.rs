use std::f32::consts::PI;

use bevy::{
    ecs::system::SystemId,
    math::bounding::{BoundingCircle, RayCast2d},
    prelude::*,
};
use rand::{Rng, SeedableRng, rngs::StdRng};

use crate::{
    GameState,
    audio::BeatLevelAudioHandle,
    edge::Edge,
    vertex::{Vertex, VertexMaterial},
};

pub fn plugin(app: &mut App) {
    app.insert_resource(Level(1))
        .init_resource::<NextLevelTimer>()
        .add_systems(Startup, (setup, generate_level))
        .add_systems(
            FixedUpdate,
            tick_next_level_timer.run_if(in_state(GameState::LevelTransition)),
        )
        .add_systems(
            OnExit(GameState::LevelTransition),
            (switch_level, generate_level).chain(),
        );
}

#[derive(Resource)]
pub struct CheckIfSolvedSystem(pub SystemId);

#[derive(Resource)]
struct Level(u64);

#[derive(Component)]
struct LevelText;

fn setup(mut commands: Commands) {
    let id = commands.register_system(check_if_solved);
    commands.insert_resource(CheckIfSolvedSystem(id));

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
        positions.push(loop {
            // TODO: limit number of iterations
            let pos = (positions[rng.gen_range(0..positions.len())]
                + rng.gen_range(Vertex::RADIUS * 2.0..Edge::MAX_LEN + Vertex::RADIUS * 2.0)
                    * Vec2::from_angle(rng.gen_range(-PI..PI)))
            .clamp(Vec2::new(-620.0, -620.0), Vec2::new(620.0, 620.0));
            const MIN_DIST: f32 = Vertex::RADIUS * 2.0 + 40.0;
            if positions
                .iter()
                .all(|p| p.distance_squared(pos) > MIN_DIST * MIN_DIST)
            {
                break pos;
            }
        });
    }

    let mut required_edges = vec![0; vertex_count];
    const EDGE_PROBABILITY: f32 = 0.5;

    for (i1, pos1) in positions.iter().enumerate() {
        for i2 in i1 + 1..vertex_count {
            let pos2 = positions[i2];
            let dist = pos1.distance(pos2);
            if dist < Edge::MAX_LEN + Vertex::RADIUS * 2.0
                && (required_edges[i1] == 0 || rng.r#gen::<f32>() < EDGE_PROBABILITY)
                && {
                    let dir = Dir2::new(pos2 - pos1).unwrap_or(Dir2::X);
                    let ray_cast = RayCast2d::new(
                        *pos1 + (Vertex::RADIUS + 0.1) * dir,
                        dir,
                        dist - 2.0 * (Vertex::RADIUS + 0.1),
                    );
                    let mut clear = true;
                    for obstacle_pos in &positions {
                        let circle = BoundingCircle::new(*obstacle_pos, Vertex::RADIUS);
                        if ray_cast.circle_intersection_at(&circle).is_some() {
                            clear = false;
                            break;
                        }
                    }
                    clear
                }
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
    mut next_state: ResMut<NextState<GameState>>,
    beat_level_audio: Res<BeatLevelAudioHandle>,
    mut commands: Commands,
) {
    let solved = vertex_q
        .iter()
        .all(|(_, vertex)| vertex.edges.len() == vertex.required_edges);
    if solved {
        next_state.set(GameState::LevelTransition);
        commands.spawn((
            AudioPlayer(beat_level_audio.0.clone()),
            PlaybackSettings::DESPAWN,
        ));
    }
}

#[derive(Resource)]
struct NextLevelTimer(Timer);

impl Default for NextLevelTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(1.0, TimerMode::Once))
    }
}

fn tick_next_level_timer(
    mut timer: ResMut<NextLevelTimer>,
    time: Res<Time>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    timer.0.tick(time.delta());
    if timer.0.finished() {
        next_state.set(GameState::Playing);
        timer.0.reset();
    }
}

#[allow(clippy::type_complexity)]
fn switch_level(
    despawn_q: Query<Entity, Or<(With<Vertex>, With<Edge>)>>,
    mut level: ResMut<Level>,
    mut level_text_q: Query<&mut Text2d, With<LevelText>>,
    mut commands: Commands,
) {
    for entity in &despawn_q {
        commands.entity(entity).despawn_recursive();
    }
    level.0 += 1;
    if let Ok(mut level_text) = level_text_q.get_single_mut() {
        level_text.0 = format!("Level {}", level.0);
    }
}
