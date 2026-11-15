mod chords;
mod grid;
mod midi;
mod music;
mod note;

use bevy::prelude::*;
use chords::ChordsPlugin;
use grid::GridPlugin;
use midi::MidiPlugin;
use note::NotePlugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((MidiPlugin, GridPlugin, NotePlugin, ChordsPlugin))
        .add_systems(Startup, |mut commands: Commands| {
            commands.spawn(Camera2d);
        })
        .run();
}
