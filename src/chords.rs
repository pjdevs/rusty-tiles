use crate::{
    music::{NoteId, detect_chords},
    note::NoteBar,
};
use bevy::{prelude::*, sprite::Anchor, window::PrimaryWindow};

pub struct ChordsPlugin;

impl Plugin for ChordsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_chord_text)
            .add_systems(Update, draw_current_chord);
    }
}

#[derive(Component)]
struct ChordText;

fn spawn_chord_text(window: Single<&Window, With<PrimaryWindow>>, mut commands: Commands) {
    const MARGIN: f32 = 20.0;

    commands.spawn((
        ChordText,
        Text2d::new(""),
        TextFont {
            font_size: FontSize::Px(48.0),
            ..default()
        },
        TextColor(Color::WHITE),
        Anchor::TOP_RIGHT,
        Transform::from_xyz(
            window.width() / 2.0 - MARGIN,
            window.height() / 2.0 - MARGIN,
            5.0,
        ),
    ));
}

fn draw_current_chord(notes: Query<&NoteBar>, chord: Single<&mut Text2d, With<ChordText>>) {
    let active_notes: Vec<NoteId> = notes
        .iter()
        .filter(|note| note.is_pressed())
        .map(|note| note.note())
        .collect();

    let mut chord_text = chord.into_inner();
    chord_text.clear();

    let chord_names = detect_chords(&active_notes);
    chord_text.push_str(&chord_names.join(" / "));
}
