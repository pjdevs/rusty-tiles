use bevy::{prelude::*, window::PrimaryWindow};

use crate::{midi::midi_note_id_to_name, note::{MAX_NOTE, MIN_NOTE}};

const WHITE_KEY_BG: Color = Color::srgb_u8(27, 40, 57);
const BLACK_KEY_BG: Color = Color::srgb_u8(35, 51, 72);
const BACKGROUND: Color = Color::srgb_u8(24, 34, 48);
const GRID_COLOR: Color = Color::srgba(0.502, 0.584, 0.663, 0.25);
const SPAWN_LINE_COLOR: Color = Color::srgb_u8(241, 206, 112);
pub(crate) const NOTE_SPAWN_LINE_OFFSET: f32 = 100.0;
const NOTE_COUNT: f32 = (MAX_NOTE - MIN_NOTE + 1) as f32;

pub struct GridPlugin;

impl Plugin for GridPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(BACKGROUND))
            .add_systems(Update, sync_piano_grid);
    }
}

#[derive(Component)]
struct PianoGrid;

fn sync_piano_grid(
    mut commands: Commands,
    window: Single<&Window, With<PrimaryWindow>>,
    mut last_size: Local<Option<Vec2>>,
    existing: Query<Entity, With<PianoGrid>>,
) {
    let width = window.width();
    let height = window.height();
    let size = Vec2::new(width, height);

    if *last_size == Some(size) {
        return;
    }
    *last_size = Some(size);

    for entity in &existing {
        commands.entity(entity).despawn();
    }

    let key_width = width / NOTE_COUNT;
    let play_y = -height / 2.0 + NOTE_SPAWN_LINE_OFFSET;

    for note in MIN_NOTE..=MAX_NOTE {
        let index = (note - MIN_NOTE) as f32;
        let x = -width / 2.0 + (index + 0.5) * key_width;
        let is_black_key = matches!(note % 12, 1 | 3 | 6 | 8 | 10);
        let color = if is_black_key { BLACK_KEY_BG } else { WHITE_KEY_BG };

        commands.spawn((
            PianoGrid,
            Sprite::from_color(color, Vec2::new(key_width, height)),
            Transform::from_xyz(x, 0.0, -2.0),
        ));
    }

    for i in 0..=NOTE_COUNT as usize {
        let x = -width / 2.0 + i as f32 * key_width;
        commands.spawn((
            PianoGrid,
            Sprite::from_color(GRID_COLOR, Vec2::new(1.0, height)),
            Transform::from_xyz(x, 0.0, -1.0),
        ));
    }

    commands.spawn((
        PianoGrid,
        Sprite::from_color(SPAWN_LINE_COLOR, Vec2::new(width, 2.0)),
        Transform::from_xyz(0.0, play_y, 1.0),
    ));

    for note in MIN_NOTE..=MAX_NOTE {
        if note % 12 != 0 {
            continue;
        }

        let index = (note - MIN_NOTE) as f32;
        let x = -width / 2.0 + (index + 0.5) * key_width;
        commands.spawn((
            PianoGrid,
            Text2d::new(midi_note_id_to_name(note)),
            TextFont {
                font_size: FontSize::Px(12.0),
                ..default()
            },
            TextColor(Color::WHITE),
            Transform::from_xyz(x, play_y - 15.0, 2.0),
        ));
    }
}
