use std::{
    sync::{
        Mutex,
        mpsc::{Receiver, Sender, channel},
    },
    time::Duration,
};

use bevy::{prelude::*, time::common_conditions::on_real_timer};
use midir::{Ignore, MidiInput, MidiInputConnection};

use crate::music::{Note, NoteId};

pub struct MidiPlugin;

impl Plugin for MidiPlugin {
    fn build(&self, app: &mut App) {
        #[cfg(target_os = "macos")]
        coremidi_hotplug_notification::receive_device_updates(|| {})
            .expect("Failed to initialize MIDI hotplug.");

        app.add_message::<MidiNoteMessage>()
            .add_systems(
                Update,
                try_connect_input_midi_system
                    .run_if(not(resource_exists::<MidiInputResource>))
                    .run_if(on_real_timer(Duration::from_millis(500))),
            )
            .add_systems(
                Update,
                check_midi_disconnection
                    .run_if(resource_exists::<MidiInputResource>)
                    .run_if(on_real_timer(Duration::from_millis(500))),
            )
            .add_systems(
                Update,
                poll_midi_messages.run_if(resource_exists::<MidiInputResource>),
            );
    }
}

#[derive(Message)]
pub struct MidiNoteMessage {
    pub note: MidiNote,
}

fn try_connect_input_midi_system(mut commands: Commands) {
    let Ok(mut midi_input) = MidiInput::new("MIDI Input") else {
        error!("MIDI driver not available");
        return;
    };
    midi_input.ignore(Ignore::TimeAndActiveSense);

    let mut ports = midi_input.ports();
    if ports.is_empty() {
        return;
    }

    let midi_input_port = ports.get_mut(0).expect("There should be some ports");
    let input_port_name = midi_input
        .port_name(midi_input_port)
        .unwrap_or_else(|_| "Unknown MIDI Port".to_string());
    info!(
        "Using MIDI input: {} (ID: \"{}\")",
        input_port_name,
        midi_input_port.id()
    );

    let (sender, receiver) = channel();
    let midi_input_connection = midi_input.connect(
        midi_input_port,
        "midir-read-input",
        handle_midi_input,
        sender,
    );

    if midi_input_connection.is_err() {
        error!("Failed to connect to MIDI input '{}'", input_port_name);
        return;
    }

    info!(
        "Connection opened reading input from '{}'.",
        input_port_name
    );

    commands.insert_resource(MidiInputResource {
        midi_port_id: midi_input_port.id(),
        _midi_input_connection: Mutex::new(
            midi_input_connection.expect("Connection should be established successfully"),
        ),
        midi_note_receiver: Mutex::new(receiver),
    });
}

#[derive(Event)]
pub struct MidiPortDisconnectedEvent;

fn check_midi_disconnection(midi: Res<MidiInputResource>, mut commands: Commands) {
    let Ok(input) = MidiInput::new("MIDI Monitor") else {
        return;
    };

    if input.find_port_by_id(&midi.midi_port_id).is_none() {
        info!("MIDI device disconnected: {}", midi.midi_port_id);

        commands.remove_resource::<MidiInputResource>();
        commands.trigger(MidiPortDisconnectedEvent);
    }
}

#[derive(Resource)]
struct MidiInputResource {
    midi_port_id: String,
    _midi_input_connection: Mutex<MidiInputConnection<Sender<MidiNote>>>,
    midi_note_receiver: Mutex<Receiver<MidiNote>>,
}

fn poll_midi_messages(
    midi_input_resource: Res<MidiInputResource>,
    mut note_message: MessageWriter<MidiNoteMessage>,
) {
    if let Ok(receiver) = midi_input_resource.midi_note_receiver.lock() {
        while let Ok(note) = receiver.try_recv() {
            note_message.write(MidiNoteMessage { note });
        }
    }
}

fn handle_midi_input(_stamp: u64, message: &[u8], receiver: &mut Sender<MidiNote>) {
    if let Ok(note) = decode_midi_note(message)
        && let Err(e) = receiver.send(note)
    {
        error!("Failed to send MIDI note: {}", e);
    }
}

fn decode_midi_note(message: &[u8]) -> Result<MidiNote, DecodeMidiError> {
    if message.len() < 3 {
        return Err(DecodeMidiError::PayloadError);
    }

    let status = message[0];
    let note = message[1];
    let velocity = message[2];
    let channel = (status & 0x0F) + 1;
    let command = status & 0xF0;

    if command != 0x90 && command != 0x80 {
        return Err(DecodeMidiError::CommandTypeError);
    }

    let pressed = command == 0x90 && velocity > 0;
    let pressure = velocity as f32 / 127.0 * 100.0;
    let octave = (note / 12) as i8 - 1;
    let note = NoteId(note).note();

    Ok(MidiNote {
        note,
        octave,
        pressed,
        pressure,
        channel,
    })
}

#[derive(Copy, Clone)]
pub struct MidiNote {
    pub note: Note,
    pub octave: i8,
    pub pressed: bool,
    pub pressure: f32,
    pub channel: u8,
}

impl MidiNote {
    pub fn id(&self) -> NoteId {
        let id = (self.octave as i16 + 1) * 12 + self.note as i16;

        NoteId(id.rem_euclid(128) as u8)
    }

    pub fn name(&self) -> String {
        format!("{}{}", self.note, self.octave)
    }
}

enum DecodeMidiError {
    PayloadError,
    CommandTypeError,
}

#[cfg(test)]
mod tests {
    use super::{MidiNote, decode_midi_note};
    use crate::music::NoteId;

    #[test]
    fn all_midi_note_ids_round_trip() {
        for value in 0..=127 {
            let id = NoteId(value);
            let note = MidiNote {
                note: id.note(),
                octave: id.octave(),
                pressed: true,
                pressure: 100.0,
                channel: 1,
            };
            assert_eq!(note.id(), id);
            assert_eq!(note.name(), id.to_string());
        }
    }

    #[test]
    fn decoding_preserves_all_midi_note_ids() {
        for value in 0..=127 {
            let Ok(note) = decode_midi_note(&[0x90, value, 127]) else {
                panic!("Failed to decode MIDI note {value}");
            };
            assert_eq!(note.id(), NoteId(value));
        }
    }
}
