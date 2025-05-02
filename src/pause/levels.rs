use bevy::prelude::*;

use crate::{
    GameState,
    edge::Edge,
    level::{Level, LevelText, generate_level},
    vertex::{Vertex, VertexMaterial},
};

pub fn plugin(app: &mut App) {
    app.add_systems(
        Update,
        handle_arrows.run_if(in_state(GameState::LevelSelect)),
    )
    .add_systems(OnEnter(GameState::LevelSelect), setup);
}

#[derive(Component)]
struct LevelVertex(u64);

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut color_materials: ResMut<Assets<ColorMaterial>>,
    mut vertex_materials: ResMut<Assets<VertexMaterial>>,
    level: Res<Level>,
) {
    commands
        .spawn((
            StateScoped(GameState::LevelSelect),
            Mesh2d(meshes.add(Rectangle::new(1440.0, 1440.0))),
            MeshMaterial2d(color_materials.add(Color::BLACK)),
            Transform::from_xyz(0.0, 0.0, 1.0),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text2d::new("Levels"),
                TextFont {
                    font_size: 60.0,
                    ..default()
                },
                Transform::from_xyz(0.0, 690.0, 0.0),
            ));

            const DIST: f32 = 330.0;
            fn i_to_pos(i: u64) -> Vec2 {
                let column = (i - 1) / 2;
                let is_first_in_column = (i - 1) % 2 == 0;
                let row = if column % 2 == 0 {
                    is_first_in_column as u8
                } else {
                    !is_first_in_column as u8
                };
                Vec2::new(-670.0 + column as f32 * DIST, row as f32 * DIST)
            }

            for i in 1..=10 {
                let pos = i_to_pos(i);
                parent
                    .spawn((
                        LevelVertex(i),
                        Mesh2d(meshes.add(Circle::new(Vertex::RADIUS))),
                        MeshMaterial2d(vertex_materials.add(VertexMaterial {
                            bits: if i < level.0 { 2 } else { 0 },
                        })),
                        Transform::from_translation(pos.extend(2.0)),
                    ))
                    .with_child((
                        Text2d(format!("{}", i)),
                        TextFont {
                            font_size: 70.0,
                            ..default()
                        },
                        TextColor(if i < level.0 {
                            Color::BLACK
                        } else {
                            Color::WHITE
                        }),
                    ))
                    .observe(handle_vertex_click);
                if i == 10 {
                    continue;
                }
                let next_pos = i_to_pos(i + 1);
                parent.spawn((
                    Mesh2d(meshes.add(Rectangle::new(DIST, Edge::WIDTH))),
                    MeshMaterial2d(color_materials.add(Color::WHITE)),
                    Transform {
                        translation: ((pos + next_pos) / 2.0).extend(1.0),
                        rotation: {
                            let diff = next_pos - pos;
                            Quat::from_rotation_z(diff.y.atan2(diff.x))
                        },
                        ..default()
                    },
                ));
            }
        });
}

fn handle_arrows(
    kb: Res<ButtonInput<KeyCode>>,
    mut level_vertex_q: Query<(&mut LevelVertex, &MeshMaterial2d<VertexMaterial>, &Children)>,
    mut text_q: Query<(&mut Text2d, &mut TextColor)>,
    mut vertex_materials: ResMut<Assets<VertexMaterial>>,
    level: Res<Level>,
) -> Result {
    let dir = if kb.just_pressed(KeyCode::ArrowLeft) {
        -1
    } else if kb.just_pressed(KeyCode::ArrowRight) {
        1
    } else {
        return Ok(());
    };
    for (mut level_vertex, mesh_material, children) in &mut level_vertex_q {
        if level_vertex.0 <= 10 && dir < 0 {
            return Ok(());
        }
        level_vertex.0 = (level_vertex.0 as i32 + dir * 10) as u64;
        let (mut text, mut color) = text_q.get_mut(children[0])?;
        text.0 = format!("{}", level_vertex.0);
        let material = vertex_materials
            .get_mut(mesh_material)
            .ok_or("Invalid vertex material handle")?;
        material.set_solved(level_vertex.0 < level.0, &mut color);
    }
    Ok(())
}

#[allow(clippy::type_complexity)]
fn handle_vertex_click(
    trigger: Trigger<Pointer<Click>>,
    despawn_q: Query<Entity, Or<(With<Vertex>, With<Edge>)>>,
    level_vertex_q: Query<&LevelVertex>,
    mut level: ResMut<Level>,
    mut level_text: Single<&mut Text2d, With<LevelText>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
) -> Result {
    for entity in &despawn_q {
        commands.entity(entity).despawn();
    }
    level.0 = level_vertex_q.get(trigger.target())?.0;
    level_text.0 = format!("Level {}", level.0);
    next_state.set(GameState::Playing);
    commands.run_system_cached(generate_level);
    Ok(())
}
