#![cfg_attr(not(feature = "console"), windows_subsystem = "windows")]

use bevy::prelude::*;

fn main() -> AppExit {
    App::new().add_plugins(DefaultPlugins).run()
}
