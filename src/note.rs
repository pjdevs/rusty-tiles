use bevy::{prelude::*, window::PrimaryWindow};

use crate::{grid::NOTE_SPAWN_LINE_OFFSET, midi::MidiNoteMessage};

pub const MIN_NOTE: i64 = 21; // A0
pub const MAX_NOTE: i64 = 108; // C8
const NOTE_COUNT: f32 = (MAX_NOTE - MIN_NOTE + 1) as f32;
const SPEED: f32 = 120.0;
const MINT: Color = Color::srgb_u8(93, 207, 184);
const BLUE: Color = Color::srgb_u8(128, 184, 232);

pub struct NotePlugin;

impl Plugin for NotePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (handle_midi_notes, update_note_bars, destroy_bars).chain(),
        );
    }
}

#[derive(Component)]
#[require(Sprite)]
struct NoteBar {
    midi_note_id: i64,
    started_at: f64,
    ended_at: Option<f64>,
}

fn handle_midi_notes(
    time: Res<Time>,
    mut notes: Query<&mut NoteBar>,
    mut note_message: MessageReader<MidiNoteMessage>,
    mut commands: Commands,
) {
    for MidiNoteMessage { note } in note_message.read() {
        info!(
            "Received MIDI note: [{}] {} {}",
            note.channel,
            note.name(),
            if note.pressed {
                format!("DOWN {}%", note.pressure)
            } else {
                "UP".to_string()
            }
        );

        if note.pressed {
            commands.spawn((
                Sprite::from_color(MINT, Vec2::new(20.0, 1.0)),
                NoteBar {
                    midi_note_id: note.id(),
                    started_at: time.elapsed_secs_f64(),
                    ended_at: None,
                },
            ));
        } else {
            for mut bar in notes.iter_mut() {
                if bar.midi_note_id == note.id() && bar.ended_at.is_none() {
                    bar.ended_at = Some(time.elapsed_secs_f64());
                }
            }
        }
    }
}

fn update_note_bars(
    time: Res<Time>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut notes: Query<(&NoteBar, &mut Transform, &mut Sprite)>,
) {
    let now = time.elapsed_secs_f64();
    let width = window.width();
    let height = window.height();
    let key_width = width / NOTE_COUNT;
    let play_line_y = -height / 2.0 + NOTE_SPAWN_LINE_OFFSET;

    for (note, mut transform, mut sprite) in &mut notes {
        if !(MIN_NOTE..=MAX_NOTE).contains(&note.midi_note_id) {
            continue;
        }

        let index = (note.midi_note_id - MIN_NOTE) as f32;
        transform.translation.x = -width / 2.0 + (index + 0.5) * key_width;

        let start_y = play_line_y + (now - note.started_at) as f32 * SPEED;
        let end_y = play_line_y + (now - note.ended_at.unwrap_or(now)) as f32 * SPEED;
        transform.translation.y = (start_y + end_y) / 2.0;

        sprite.color = if note.ended_at.is_some() { BLUE } else { MINT };
        sprite.custom_size = Some(Vec2::new(
            (key_width - 1.0).max(1.0),
            (start_y - end_y).max(1.0),
        ));
    }
}

fn destroy_bars(
    window: Single<&Window, With<PrimaryWindow>>,
    notes: Query<(Entity, &NoteBar, &Transform, &Sprite)>,
    mut commands: Commands,
) {
    let top = window.height() / 2.0;

    for (entity, note, transform, sprite) in &notes {
        let height = sprite.custom_size.unwrap_or(Vec2::ZERO).y;
        let bottom = transform.translation.y - height / 2.0;

        if note.ended_at.is_some() && bottom > top {
            commands.entity(entity).despawn();
        }
    }
}
