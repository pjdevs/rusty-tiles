mod grid;
mod midi;
mod note;

use bevy::prelude::*;
use grid::GridPlugin;
use midi::MidiPlugin;
use note::NotePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((MidiPlugin, GridPlugin, NotePlugin))
        .add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Camera2d);
        })
        .run();
}
