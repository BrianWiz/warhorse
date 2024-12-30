mod warhorse;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(warhorse::BevyWarhorsePlugin)
        .add_systems(Startup,startup_system)
        .run();
}

fn startup_system(mut commands: Commands) {
    commands.spawn(Camera2d::default());
}
