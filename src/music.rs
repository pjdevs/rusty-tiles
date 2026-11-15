use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum Note {
    C,
    CSharp,
    D,
    DSharp,
    E,
    F,
    FSharp,
    G,
    GSharp,
    A,
    ASharp,
    B,
}

impl Note {
    pub const ALL: [Self; 12] = [
        Self::C,
        Self::CSharp,
        Self::D,
        Self::DSharp,
        Self::E,
        Self::F,
        Self::FSharp,
        Self::G,
        Self::GSharp,
        Self::A,
        Self::ASharp,
        Self::B,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Self::C => "C",
            Self::CSharp => "C#",
            Self::D => "D",
            Self::DSharp => "D#",
            Self::E => "E",
            Self::F => "F",
            Self::FSharp => "F#",
            Self::G => "G",
            Self::GSharp => "G#",
            Self::A => "A",
            Self::ASharp => "A#",
            Self::B => "B",
        }
    }
}

impl fmt::Display for Note {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NoteId(pub u8);

impl NoteId {
    pub fn note(self) -> Note {
        Note::ALL[(self.0 % 12) as usize]
    }

    pub fn octave(self) -> i8 {
        (self.0 / 12) as i8 - 1
    }
}

impl fmt::Display for NoteId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.note(), self.octave())
    }
}

pub const CHORDS: &[(&str, &[u8])] = &[
    ("", &[0, 4, 7]),
    ("m", &[0, 3, 7]),
    ("dim", &[0, 3, 6]),
    ("aug", &[0, 4, 8]),
    ("7", &[0, 4, 7, 10]),
    ("maj7", &[0, 4, 7, 11]),
    ("m7", &[0, 3, 7, 10]),
    ("m7b5", &[0, 3, 6, 10]),
    ("dim7", &[0, 3, 6, 9]),
    ("sus2", &[0, 2, 7]),
    ("sus4", &[0, 5, 7]),
    ("6", &[0, 4, 7, 9]),
    ("m6", &[0, 3, 7, 9]),
    ("9", &[0, 2, 4, 7, 10]),
    ("maj9", &[0, 2, 4, 7, 11]),
    ("m9", &[0, 2, 3, 7, 10]),
    ("add9", &[0, 2, 4, 7]),
    ("madd9", &[0, 2, 3, 7]),
];

pub fn detect_chords(notes: &[NoteId]) -> Vec<String> {
    let Some(bass) = notes.iter().min().map(|n| n.note()) else {
        return Vec::new();
    };

    let mut pitches: Vec<u8> = notes.iter().map(|n| n.note() as u8).collect();

    pitches.sort_unstable();
    pitches.dedup();

    let mut results = Vec::new();

    for root in Note::ALL {
        let mut intervals: Vec<u8> = pitches
            .iter()
            .map(|&pitch| (pitch + 12 - root as u8) % 12)
            .collect();

        intervals.sort_unstable();

        for &(suffix, pattern) in CHORDS {
            if intervals == pattern {
                let name = format!("{root}{suffix}");

                results.push(if root == bass {
                    name
                } else {
                    format!("{name}/{bass}")
                });
            }
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use crate::music::{Note, NoteId, detect_chords};

    #[test]
    fn empty_or_single_pitch_has_no_chord() {
        assert!(detect_chords(&[]).is_empty());
        assert!(detect_chords(&[NoteId(60)]).is_empty());
        assert!(detect_chords(&[NoteId(48), NoteId(60), NoteId(72)]).is_empty());
    }

    #[test]
    fn duplicates_and_octaves_do_not_change_chord() {
        assert_eq!(
            detect_chords(&[NoteId(48), NoteId(60), NoteId(64), NoteId(67), NoteId(67)]),
            vec!["C"]
        );
    }

    #[test]
    fn input_order_does_not_change_chords_or_bass() {
        let notes = [NoteId(48), NoteId(57), NoteId(64), NoteId(67)];
        let expected = vec!["C6", "Am7/C"];
        for &a in &notes {
            for &b in &notes {
                for &c in &notes {
                    for &d in &notes {
                        if a != b && a != c && a != d && b != c && b != d && c != d {
                            assert_eq!(detect_chords(&[a, b, c, d]), expected);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn inversion_names_the_lowest_note_as_bass() {
        assert_eq!(
            detect_chords(&[NoteId(52), NoteId(60), NoteId(67)]),
            vec!["C/E"]
        );
    }

    #[test]
    fn augmented_chord_keeps_all_interpretations() {
        assert_eq!(
            detect_chords(&[NoteId(60), NoteId(64), NoteId(68)]),
            vec!["Caug", "Eaug/C", "G#aug/C"]
        );
    }

    #[test]
    fn unmatched_notes_have_no_chord() {
        assert!(detect_chords(&[NoteId(60), NoteId(61), NoteId(62)]).is_empty());
    }

    #[test]
    fn note_ids_have_expected_pitch_octave_and_name() {
        for (id, note, octave, name) in [
            (NoteId(0), Note::C, -1, "C-1"),
            (NoteId(60), Note::C, 4, "C4"),
            (NoteId(127), Note::G, 9, "G9"),
        ] {
            assert_eq!(id.note(), note);
            assert_eq!(id.octave(), octave);
            assert_eq!(id.to_string(), name);
        }
    }

    #[test]
    fn test_detect_chords() {
        assert_eq!(
            detect_chords(&[NoteId(60), NoteId(64), NoteId(67)]),
            vec!["C"]
        );

        assert_eq!(
            detect_chords(&[NoteId(48), NoteId(57), NoteId(64)]),
            vec!["Am/C"]
        );

        assert_eq!(
            detect_chords(&[NoteId(45), NoteId(60), NoteId(64), NoteId(67)]),
            vec!["C6/A", "Am7"]
        );

        assert_eq!(
            detect_chords(&[NoteId(48), NoteId(57), NoteId(64), NoteId(67)]),
            vec!["C6", "Am7/C"]
        );
    }
}
