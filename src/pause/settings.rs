use bevy::{
    prelude::*,
    window::{PrimaryWindow, WindowMode},
};

use crate::GameState;

pub fn plugin(app: &mut App) {
    app.add_systems(Update, update_fullscreen)
        .add_systems(OnEnter(GameState::Settings), setup);
}

#[derive(Component)]
struct FullscreenButton;

fn setup(mut commands: Commands, window: Single<&Window, With<PrimaryWindow>>) {
    use WindowMode::*;
    commands.spawn((
        StateScoped(GameState::Settings),
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
                Text::new("Settings"),
                TextFont {
                    font_size: 60.0,
                    ..default()
                },
                TextColor(Color::BLACK),
            ),
            (
                Button,
                FullscreenButton,
                Text(format!(
                    "Fullscreen [{}]",
                    match window.mode {
                        Windowed => " ",
                        BorderlessFullscreen(_) | Fullscreen(..) => "X",
                    }
                )),
                TextFont {
                    font_size: 50.0,
                    ..default()
                },
                TextColor(Color::BLACK),
                Node {
                    top: Val::Percent(35.0),
                    ..default()
                }
            ),
        ],
    ));
}

#[allow(clippy::type_complexity)]
fn update_fullscreen(
    mut q: Query<
        (&Interaction, &mut BackgroundColor, &mut Text),
        (Changed<Interaction>, With<FullscreenButton>),
    >,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
) {
    use Interaction::*;
    for (interaction, mut bg, mut text) in &mut q {
        match *interaction {
            None => {
                bg.0 = Color::NONE;
            }
            Hovered => {
                bg.0 = Color::srgb(0.8, 0.8, 0.8);
            }
            Pressed => {
                bg.0 = Color::srgb(0.6, 0.6, 0.6);
                use WindowMode::*;
                let (mode, str) = match window.mode {
                    Windowed => (BorderlessFullscreen(MonitorSelection::Current), "X"),
                    BorderlessFullscreen(_) | Fullscreen(..) => (Windowed, " "),
                };
                window.mode = mode;
                text.replace_range(12..13, str);
            }
        }
    }
}
