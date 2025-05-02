use bevy::{input::common_conditions::input_just_pressed, prelude::*, window::PrimaryWindow};

use crate::GameState::{self, *};

mod levels;
mod settings;

pub fn plugin(app: &mut App) {
    app.add_plugins((levels::plugin, settings::plugin))
        .add_systems(
            Update,
            (
                pause.run_if(input_just_pressed(KeyCode::Escape)),
                update_ui_scale,
                update_buttons,
            ),
        )
        .add_systems(OnEnter(Paused), setup);
}

fn pause(state: Res<State<GameState>>, mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(match state.get() {
        Playing | LevelSelect | Settings => Paused,
        Paused => Playing,
        LevelTransition => return,
    });
}

#[derive(Component)]
enum ButtonType {
    Levels,
    Settings,
    Exit,
}

fn setup(mut commands: Commands) {
    commands.spawn((
        StateScoped(Paused),
        Node {
            width: Val::Percent(30.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(Color::WHITE),
        children![
            (
                Text::new("Paused"),
                TextFont {
                    font_size: 60.0,
                    ..default()
                },
                TextColor(Color::BLACK),
            ),
            (
                Button,
                ButtonType::Levels,
                Text::new("Levels"),
                TextFont {
                    font_size: 50.0,
                    ..default()
                },
                TextColor(Color::BLACK),
                Node {
                    top: Val::Percent(30.0),
                    ..default()
                }
            ),
            (
                Button,
                ButtonType::Settings,
                Text::new("Settings"),
                TextFont {
                    font_size: 50.0,
                    ..default()
                },
                TextColor(Color::BLACK),
                Node {
                    top: Val::Percent(40.0),
                    ..default()
                }
            ),
            (
                Button,
                ButtonType::Exit,
                Text::new("Exit"),
                TextFont {
                    font_size: 50.0,
                    ..default()
                },
                TextColor(Color::BLACK),
                Node {
                    top: Val::Percent(50.0),
                    ..default()
                }
            )
        ],
    ));
}

fn update_buttons(
    mut q: Query<(&Interaction, &ButtonType, &mut BackgroundColor), Changed<Interaction>>,
    mut exit_evw: EventWriter<AppExit>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    use ButtonType::*;
    use Interaction::*;
    for (interaction, button_type, mut bg) in &mut q {
        match *interaction {
            None => {
                bg.0 = Color::NONE;
            }
            Hovered => {
                bg.0 = Color::srgb(0.8, 0.8, 0.8);
            }
            Pressed => {
                bg.0 = Color::srgb(0.6, 0.6, 0.6);
                match *button_type {
                    Levels => next_state.set(GameState::LevelSelect),
                    Settings => next_state.set(GameState::Settings),
                    Exit => {
                        exit_evw.write(AppExit::Success);
                    }
                };
            }
        }
    }
}

fn update_ui_scale(mut scale: ResMut<UiScale>, window: Single<&Window, With<PrimaryWindow>>) {
    **scale = window.resolution.size().min_element() / 1440.0;
}
