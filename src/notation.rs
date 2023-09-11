use crate::error::{Error, Result};
use crate::music_xml_types::TimeModificationElement;
use log::{error, info, trace};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::collections::BTreeSet;
use std::convert::From;

use std::str::FromStr;

pub struct MeasureChecker {
    measure: Vec<MusicElement>,
    elems_since_backup: usize,
    quarter_division: u32,
    beats: Beats,
    beat_type: BeatType,
    measure_idx: usize,
}

impl MeasureChecker {
    const MAX_SUPPORTED_VOICES: usize = 4;
    pub fn new(
        quarter_division: u32,
        measure_init: &MeasureInitializer,
        measure_idx: usize,
    ) -> MeasureChecker {
        MeasureChecker {
            measure: vec![],
            elems_since_backup: 0,
            quarter_division,
            beats: measure_init.beats,
            beat_type: measure_init.beat_type,
            measure_idx,
        }
    }

    pub fn push_elem(&mut self, elem: MusicElement) {
        self.measure.push(elem);
        self.elems_since_backup += 1;
    }

    pub fn quarter_division(&self) -> u32 {
        self.quarter_division
    }

    pub fn measure_idx(&self) -> usize {
        self.measure_idx
    }

    pub fn conform_backup_placeholder_rests(&mut self, backup_duration: usize) {
        // Backup elements are only inserted when voice changes happen.
        // Calculate duration to current point, since previous voice began, based on notes in the measure, and accounting for corresponding
        // time modifying elements
        let last_backup_idx = self.measure.len() - self.elems_since_backup;
        let mut time_mod: Option<TimeModification> = None;
        let mut current_voice= Voice::One;
        let duration_since_backup: usize = self.measure[last_backup_idx..]
            .iter()
            .cloned()
            .map(|element| match element {
                MusicElement::NoteRest(n) => {
                    // chords have a duration in musicxml, but this duration is always identical to the previous note it's
                    // attached to. Chord duration shouldn't impact the total summation.
                    current_voice = n.voice;
                    if n.chord == Chord::NoChord {
                        n.get_duration_numeric(
                            self.quarter_division,
                            u32::from(self.beats),
                            u32::from(self.beat_type),
                            time_mod,
                        ) as usize
                    } else {
                        0
                    }
                }
                MusicElement::Tuplet(t) => {
                    match t.start_stop {
                        TupletStartStop::TupletStart => {
                            time_mod = Some(TimeModification::new(t.actual_notes, t.normal_notes));
                        }
                        TupletStartStop::TupletNone => {
                            time_mod = None;
                        }
                        TupletStartStop::TupletStop => {
                            time_mod = None;
                        }
                    }
                    0 // does not directly impact sum
                }
                _ => {
                    0 // does not impact sum
                }
            })
            .sum();

        if backup_duration < duration_since_backup {
            let discrepancy = duration_since_backup - backup_duration;
            println!("M{} duration tally {} did not match the backup element's duration {backup_duration}, inserting rests to accommodate {discrepancy} discrepancy.", self.measure_idx, duration_since_backup);
            if let Some((duration, is_dotted)) =
                NoteData::from_numeric_duration(discrepancy as u32, self.quarter_division)
            {
                // The new rest should begin on the next voice after the current one.
                self.measure.push(
                    MusicElement::NoteRest(NoteData::new_default_rest(duration, is_dotted, current_voice.next())),
                );
            } else {
                panic!(
                    "Could not convert {} in a rest duration value.",
                    discrepancy
                );
            }
        } else if backup_duration > duration_since_backup {
            info!("Backup_duration {} was > duration_since_backup {} Assuming beginning of measure", backup_duration, duration_since_backup);
        }
        self.clear_elems_since_backup();
    }

    fn clear_elems_since_backup(&mut self) {
        self.elems_since_backup = 0;
    }

    pub fn as_inner(&mut self) -> &mut Vec<MusicElement> {
        &mut self.measure
    }

    // pub fn remove_incomplete_voices(&mut self, voices: &BTreeSet<u8>) {
    //     let printout = self.measure_idx == 27;
    //     let mut voice_durations: [u32; Self::MAX_SUPPORTED_VOICES] =
    //         [0; Self::MAX_SUPPORTED_VOICES];

    //     let mut cur_tuplet_info: Option<TupletData> = None;

    //     for elem in self.measure.iter().cloned() {
    //         match elem {
    //             MusicElement::Tuplet(t) => match t.start_stop {
    //                 TupletStartStop::TupletStart => {
    //                     if printout {
    //                         // println!("TupletStart");
    //                     }
    //                     cur_tuplet_info = Some(t);
    //                 }
    //                 TupletStartStop::TupletNone => {
    //                     if printout {
    //                         // println!("TupletNone");
    //                     }
    //                     cur_tuplet_info = None;
    //                 }
    //                 TupletStartStop::TupletStop => {
    //                     if printout {
    //                         // println!("TupletStop");
    //                     }
    //                     cur_tuplet_info = None;
    //                 }
    //             },
    //             MusicElement::NoteRest(n) => {
    //                 // Do not include chord notes or grace notes in the count, as they do not impact measure duration
    //                 if n.chord == Chord::NoChord && n.special_note == SpecialNote::None {
    //                     if cur_tuplet_info.is_none() {
    //                         voice_durations[n.voice as usize] += n.get_duration_in_midi_ticks();
    //                     } else {
    //                         voice_durations[n.voice as usize] += n.get_duration_in_midi_ticks()
    //                             * cur_tuplet_info.unwrap().normal_notes as u32
    //                             / cur_tuplet_info.unwrap().actual_notes as u32;
    //                     }
    //                     if printout {
    //                         // println!(
    //                         //     "{:?} note: {:?} voice {}",
    //                         //     n.note_type, n.note_rest, n.voice as usize
    //                         // );
    //                     }
    //                 }
    //             }
    //             _ => {
    //                 error!("Unhandled element case");
    //             }
    //         }
    //     }

    //     let first_voice_duration = voice_durations[0];

    //     for (voice_idx, _) in voices.iter().enumerate() {
    //         if voice_durations[voice_idx] != first_voice_duration {
    //             // println!(
    //             //     "M{measure_idx} Voice Zero: {first_voice_duration} duration Voice {voice_idx}: {} duration",
    //             //     voice_durations[voice_idx]
    //             // );
    //         }
    //     }
    // }
}

struct DivisionsVec {
    inner: Vec<u32>,
}

impl DivisionsVec {
    const DIVISION_BASE: u32 = 960;
    // Create a new, empty DivisionsVec
    pub fn new() -> Self {
        DivisionsVec {
            inner: vec![Self::DIVISION_BASE],
        }
    }

    pub fn can_divide_by(&self, d: u32) -> bool {
        for elem in self.inner.iter() {
            if (elem <= &1) || ((elem % d) != 0) {
                return false;
            }
        }
        true
    }

    pub fn divide_by(&mut self, d: u32) {
        for elem in self.inner.iter_mut() {
            *elem /= d;
        }
    }

    // Add an item to the DivisionsVec, but only if it's not already present
    pub fn add(&mut self, value: u32) {
        if !self.inner.contains(&value) {
            self.inner.push(value);
        }
    }

    pub fn factor_by(&mut self, val: u32) {
        while self.can_divide_by(val) {
            self.divide_by(val);
        }
    }

    pub fn get_divisions(&self) -> u32 {
        Self::DIVISION_BASE / (Self::DIVISION_BASE / self.inner[0])
    }

    // Allow direct access to the inner Vec<u32>
    pub fn inner(&self) -> &Vec<u32> {
        &self.inner
    }
}

pub fn calc_divisions_voices(music_elems_v: Vec<MusicElement>, dump_input: bool) -> (u32, usize) {
    let mut voices = BTreeSet::new();
    let primes_v: Vec<u32> = vec![2, 3, 5];
    let mut integers_v = DivisionsVec::new();

    let mut tuplet_paren = false;
    let mut tuplet_actual = 1u32;
    let mut tuplet_normal = 1u32;
    for elem in music_elems_v.iter() {
        if dump_input {
            trace!("{:?}", elem);
        }
        match elem {
            MusicElement::Tuplet(t) => {
                if t.start_stop == TupletStartStop::TupletStart {
                    // Tuplet_paren indicates if a block of tuplets is being processed (Tuplet Parenthesis)
                    tuplet_paren = true;
                    tuplet_actual = t.actual_notes as u32;
                    tuplet_normal = t.normal_notes as u32;
                    // debug!(
                    //     "tuplet actual {} tup normal {}",
                    //     tuplet_actual, tuplet_normal
                    // );
                } else if t.start_stop == TupletStartStop::TupletStop {
                    tuplet_paren = false;
                }
            }
            MusicElement::NoteRest(n) => {
                voices.insert(n.voice as u8);
                let std_duration = n.get_duration_in_midi_ticks();
                if tuplet_paren {
                    integers_v.add(std_duration * tuplet_normal / tuplet_actual);
                } else {
                    integers_v.add(std_duration);
                }
            }
            _ => {}
        }
    }
    // Factor the values
    for prime in primes_v.iter() {
        integers_v.factor_by(*prime);
    }

    (integers_v.get_divisions(), voices.len())
}

#[derive(Eq, PartialEq, Default, Debug, Copy, Clone)]
pub struct TimeModification {
    actual_notes: TupletActual,
    normal_notes: TupletNormal,
}

impl From<TimeModificationElement> for TimeModification {
    fn from(time_mod_elem: TimeModificationElement) -> Self {
        TimeModification::new(time_mod_elem.actual_notes, time_mod_elem.normal_notes)
    }
}

impl TimeModification {
    pub fn new(actual_notes: TupletActual, normal_notes: TupletNormal) -> TimeModification {
        TimeModification {
            actual_notes,
            normal_notes,
        }
    }
    pub fn get_actual(&self) -> TupletActual {
        self.actual_notes
    }
    pub fn get_normal(&self) -> TupletNormal {
        self.normal_notes
    }
}
#[derive(Copy, FromPrimitive, Clone)]
#[repr(u8)]
pub enum Note {
    NoteC = 49,
    NoteCsharp = 50,
    NoteD = 51,
    NoteDsharp = 52,
    NoteE = 53,
    NoteF = 54,
    NoteFsharp = 55,
    NoteG = 56,
    NoteGsharp = 57,
    NoteA = 58,
    NoteAsharp = 59,
    NoteB = 60,
}

impl Note {
    pub fn get_note_alter_tuple(self) -> (Note, Alter) {
        match self {
            Self::NoteC => (Self::NoteC, Alter::None),
            Self::NoteCsharp => (Self::NoteC, Alter::Sharp),
            Self::NoteD => (Self::NoteD, Alter::None),
            Self::NoteDsharp => (Self::NoteD, Alter::Sharp),
            Self::NoteE => (Self::NoteE, Alter::None),
            Self::NoteF => (Self::NoteF, Alter::None),
            Self::NoteFsharp => (Self::NoteF, Alter::Sharp),
            Self::NoteG => (Self::NoteG, Alter::None),
            Self::NoteGsharp => (Self::NoteG, Alter::Sharp),
            Self::NoteA => (Self::NoteA, Alter::None),
            Self::NoteAsharp => (Self::NoteA, Alter::Sharp),
            Self::NoteB => (Self::NoteB, Alter::None),
        }
    }
}

impl ToString for Note {
    fn to_string(&self) -> String {
        match self {
            Self::NoteC => String::from("C"),
            Self::NoteCsharp => String::from("C#"),
            Self::NoteD => String::from("D"),
            Self::NoteDsharp => String::from("D#"),
            Self::NoteE => String::from("E"),
            Self::NoteF => String::from("F"),
            Self::NoteFsharp => String::from("F#"),
            Self::NoteG => String::from("G"),
            Self::NoteGsharp => String::from("G#"),
            Self::NoteA => String::from("A"),
            Self::NoteAsharp => String::from("A#"),
            Self::NoteB => String::from("B"),
        }
    }
}

#[derive(Eq, PartialEq, Default, FromPrimitive, Debug, Copy, Clone)]
#[repr(u8)]
pub enum KeySignature {
    #[default]
    CMajorAminor = 0,
    GMajorEminor = 1,
    DMajorBminor = 2,
    AMajorFsminor = 3,
    EMajorCsminor = 4,
    BMajorGsminor = 5,
    GbMajorEbminor = 6,
    DbMajorBbminor = 7,
    AbMajorFminor = 8,
    EbMajorCminor = 9,
    BbMajorGminor = 10,
    FMajorDminor = 11,
}

impl ToString for KeySignature {
    fn to_string(&self) -> String {
        match self {
            KeySignature::DbMajorBbminor => String::from("-5"),
            KeySignature::AbMajorFminor => String::from("-4"),
            KeySignature::EbMajorCminor => String::from("-3"),
            KeySignature::BbMajorGminor => String::from("-2"),
            KeySignature::FMajorDminor => String::from("-1"),
            KeySignature::CMajorAminor => String::from("0"),
            KeySignature::GMajorEminor => String::from("1"),
            KeySignature::DMajorBminor => String::from("2"),
            KeySignature::AMajorFsminor => String::from("3"),
            KeySignature::EMajorCsminor => String::from("4"),
            KeySignature::BMajorGsminor => String::from("5"),
            KeySignature::GbMajorEbminor => String::from("6"),
        }
    }
}

impl FromStr for KeySignature {
    type Err = Error;
    fn from_str(input: &str) -> Result<KeySignature> {
        match input {
            "-7" => Ok(KeySignature::BMajorGsminor),
            "-6" => Ok(KeySignature::GbMajorEbminor),
            "-5" => Ok(KeySignature::DbMajorBbminor),
            "-4" => Ok(KeySignature::AbMajorFminor),
            "-3" => Ok(KeySignature::EbMajorCminor),
            "-2" => Ok(KeySignature::BbMajorGminor),
            "-1" => Ok(KeySignature::FMajorDminor),
            "0" => Ok(KeySignature::CMajorAminor),
            "1" => Ok(KeySignature::GMajorEminor),
            "2" => Ok(KeySignature::DMajorBminor),
            "3" => Ok(KeySignature::AMajorFsminor),
            "4" => Ok(KeySignature::EMajorCsminor),
            "5" => Ok(KeySignature::BMajorGsminor),
            "6" => Ok(KeySignature::GbMajorEbminor),
            "7" => Ok(KeySignature::DbMajorBbminor),
            _ => Err(Error::Unit),
        }
    }
}

#[derive(Copy, Clone, FromPrimitive)]
#[repr(u8)]
pub enum Octave {
    Octave0 = 0,
    Octave1 = 1,
    Octave2 = 2,
    Octave3 = 3,
    Octave4 = 4,
    Octave5 = 5,
    Octave6 = 6,
    Octave7 = 7,
    Octave8 = 8,
}

#[derive(PartialEq, Eq, Copy, Clone)]
#[repr(i8)]
pub enum Alter {
    Flat = -1,
    None = 0,
    Sharp = 1,
    DoubleSharp = 2,
}

impl FromStr for Alter {
    type Err = Error;
    fn from_str(input: &str) -> Result<Alter> {
        match input {
            "-1" => Ok(Alter::Flat),
            "0" => Ok(Alter::None),
            "1" => Ok(Alter::Sharp),
            "2" => Ok(Alter::DoubleSharp),
            _ => Err(Error::Unit),
        }
    }
}

impl ToString for Alter {
    fn to_string(&self) -> String {
        match self {
            Alter::Flat => String::from("-1"),
            Alter::None => String::from("0"),
            Alter::Sharp => String::from("1"),
            Alter::DoubleSharp => String::from("2"),
        }
    }
}
impl FromStr for Octave {
    type Err = Error;
    fn from_str(input: &str) -> Result<Octave> {
        match input {
            "0" => Ok(Octave::Octave0),
            "1" => Ok(Octave::Octave1),
            "2" => Ok(Octave::Octave2),
            "3" => Ok(Octave::Octave3),
            "4" => Ok(Octave::Octave4),
            "5" => Ok(Octave::Octave5),
            "6" => Ok(Octave::Octave6),
            "7" => Ok(Octave::Octave7),
            "8" => Ok(Octave::Octave8),
            _ => Err(Error::Unit),
        }
    }
}

impl FromStr for Note {
    type Err = Error;
    fn from_str(input: &str) -> Result<Note> {
        match input {
            "C" => Ok(Note::NoteC),
            "D" => Ok(Note::NoteD),
            "E" => Ok(Note::NoteE),
            "F" => Ok(Note::NoteF),
            "G" => Ok(Note::NoteG),
            "A" => Ok(Note::NoteA),
            "B" => Ok(Note::NoteB),
            _ => Err(Error::Unit),
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, FromPrimitive, Default, Debug)]
#[repr(u8)]
pub enum NoteConnection {
    #[default]
    NoTie = 0,
    StartTie,
    EndTie,
}

impl FromStr for NoteConnection {
    type Err = Error;
    fn from_str(input: &str) -> Result<NoteConnection> {
        match input {
            "start" => Ok(NoteConnection::StartTie),
            "stop" => Ok(NoteConnection::EndTie),
            _ => Err(Error::Parse),
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, FromPrimitive, Default, Debug)]
#[repr(u8)]
pub enum SlurConnection {
    #[default]
    NoSlur = 0,
    StartSlur,
    EndSlur,
}

impl FromStr for SlurConnection {
    type Err = Error;
    fn from_str(input: &str) -> Result<SlurConnection> {
        match input {
            "start" => Ok(SlurConnection::StartSlur),
            "stop" => Ok(SlurConnection::EndSlur),
            _ => Err(Error::Parse),
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, FromPrimitive, Default, Debug)]
#[repr(u8)]
pub enum MeasureStartEnd {
    #[default]
    MeasureStart = 0,
    MeasureEnd,
    RepeatStart,
    RepeatEnd,
}

#[derive(Eq, PartialEq, Copy, Clone, FromPrimitive, Default, Debug)]
#[repr(u8)]
pub enum Articulation {
    #[default]
    None,
    Marcato,
    Stacatto,
    Legato,
}

#[derive(Eq, PartialEq, Copy, Clone, FromPrimitive, Default, Debug)]
#[repr(u8)]
pub enum Arpeggiate {
    #[default]
    NoArpeggiation,
    Arpeggiate,
}

impl From<Arpeggiate> for bool {
    fn from(f: Arpeggiate) -> bool {
        match f {
            Arpeggiate::NoArpeggiation => false,
            Arpeggiate::Arpeggiate => true,
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, FromPrimitive, Default, Debug)]
#[repr(u8)]
pub enum Stress {
    #[default]
    NotAccented,
    Accented,
}

#[derive(Eq, PartialEq, Copy, Clone, FromPrimitive, Default, Debug)]
#[repr(u8)]
pub enum Chord {
    #[default]
    NoChord,
    Chord,
}
#[derive(Eq, PartialEq, Copy, Clone, FromPrimitive, Default, Debug)]
#[repr(u8)]
pub enum TupletNumber {
    #[default]
    TupletOne,
    TupletTwo,
}

#[derive(Eq, PartialEq, Copy, Clone, FromPrimitive, Default, Debug)]
#[repr(u8)]
pub enum TupletStartStop {
    #[default]
    TupletNone,
    TupletStart,
    TupletStop,
}

pub type TupletActual = u8;
pub type TupletNormal = u8;
pub type TupletDotted = bool;

#[derive(Eq, PartialEq, Copy, Clone, Default, Debug)]
pub struct TupletData {
    pub start_stop: TupletStartStop,
    pub tuplet_number: TupletNumber,
    pub actual_notes: TupletActual,
    pub normal_notes: TupletNormal,
    pub dotted: TupletDotted,
}

impl From<Chord> for bool {
    fn from(f: Chord) -> bool {
        match f {
            Chord::NoChord => false,
            Chord::Chord => true,
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, FromPrimitive, Default, Debug)]
#[repr(u8)]
pub enum SpecialNote {
    #[default]
    None,
    Acciatura,
    Appogiatura,
    Fermata,
}

impl FromStr for SpecialNote {
    type Err = Error;
    fn from_str(input: &str) -> Result<SpecialNote> {
        match input {
            "yes" => Ok(SpecialNote::Acciatura),
            "no" => Ok(SpecialNote::Appogiatura),
            _ => Err(Error::Parse),
        }
    }
}

impl ToString for SpecialNote {
    fn to_string(&self) -> String {
        match self {
            SpecialNote::Acciatura => "yes".to_string(),
            SpecialNote::Appogiatura => "no".to_string(),
            _ => "".to_string(),
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, FromPrimitive, Default, Debug)]
#[repr(u8)]
pub enum PhraseDynamics {
    #[default]
    None,
    Sforzando,
    Fortepiano,
    Crescendo,
    Diminuendo,
    Niente,
    Rinforzando,
    Tenuto,
    Pianississimo,
    Pianissimo,
    Piano,
    MezzoPiano,
    MezzoForte,
    Forte,
    Fortissimo,
    Fortississimo,
}

impl FromStr for PhraseDynamics {
    type Err = Error;
    fn from_str(input: &str) -> Result<PhraseDynamics> {
        match input {
            "ppp" => Ok(PhraseDynamics::Pianississimo),
            "pp" => Ok(PhraseDynamics::Pianissimo),
            "p" => Ok(PhraseDynamics::Piano),
            "mp" => Ok(PhraseDynamics::MezzoPiano),
            "mf" => Ok(PhraseDynamics::MezzoForte),
            "f" => Ok(PhraseDynamics::Forte),
            "ff" => Ok(PhraseDynamics::Fortissimo),
            "fff" => Ok(PhraseDynamics::Fortississimo),
            "sf" => Ok(PhraseDynamics::Sforzando),
            "sfz" => Ok(PhraseDynamics::Sforzando),
            "fz" => Ok(PhraseDynamics::Sforzando),
            s => {
                println!("Dynamic type {}", s);
                Err(Error::Parse)
            }
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, FromPrimitive, Default, Debug)]
#[repr(u8)]
pub enum Ending {
    #[default]
    None = 0,
    One,
    Two,
    Three,
}

impl FromStr for Ending {
    type Err = Error;
    fn from_str(input: &str) -> Result<Ending> {
        match input {
            "" => Ok(Ending::None),
            "1" => Ok(Ending::One),
            "2" => Ok(Ending::Two),
            "3" => Ok(Ending::Three),
            _ => Err(Error::Unit),
        }
    }
}

impl ToString for Ending {
    fn to_string(&self) -> String {
        match self {
            Ending::None => "".to_string(),
            Ending::One => "1".to_string(),
            Ending::Two => "2".to_string(),
            Ending::Three => "3".to_string(),
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, FromPrimitive, Default, Debug)]
#[repr(u8)]
pub enum Trill {
    #[default]
    None = 0,
    Diatonic,
    Chromatic,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Tempo(u8);

impl Default for Tempo {
    fn default() -> Self {
        Tempo::from(Tempo::DEFAULT_REAL_TEMPO)
    }
}

impl Tempo {
    const MAX_SUPPORTED_RAW_TEMPO: u8 = 127;
    const MAX_SUPPORTED_REAL_TEMPO: i32 = 274;
    const MIN_SUPPORTED_REAL_TEMPO: i32 = 20;
    const DEFAULT_REAL_TEMPO: i32 = 120;

    pub fn new(real_tempo: i32) -> Tempo {
        let assign_tempo: i32;
        if real_tempo > Self::MAX_SUPPORTED_REAL_TEMPO {
            assign_tempo = Self::MAX_SUPPORTED_REAL_TEMPO;
        } else if real_tempo < Self::MIN_SUPPORTED_REAL_TEMPO {
            assign_tempo = Self::MIN_SUPPORTED_REAL_TEMPO;
        } else {
            assign_tempo = real_tempo;
        }
        Tempo(((assign_tempo - 20) / 2) as u8)
    }

    pub fn new_from_raw(raw_tempo: u8) -> Tempo {
        let assign_tempo: u8;
        if raw_tempo > Self::MAX_SUPPORTED_RAW_TEMPO {
            assign_tempo = Self::MAX_SUPPORTED_RAW_TEMPO
        } else {
            assign_tempo = raw_tempo;
        }

        Tempo(assign_tempo)
    }

    pub fn get_raw(self) -> u8 {
        self.0
    }

    pub fn get_actual(self) -> i32 {
        (self.0 as i32 * 2) + 20
    }

    pub fn get_actual_f(self) -> f32 {
        (self.0 as f32 * 2.0) + 20.0
    }
}

impl ToString for Tempo {
    fn to_string(&self) -> String {
        self.get_actual().to_string()
    }
}

impl FromStr for Tempo {
    type Err = Error;
    fn from_str(input: &str) -> Result<Tempo> {
        let parsed_num = input.parse::<i32>()?;
        Ok(Tempo::new(parsed_num))
    }
}

impl From<i32> for Tempo {
    fn from(real_tempo: i32) -> Self {
        let assign_tempo;
        if real_tempo > Self::MAX_SUPPORTED_REAL_TEMPO {
            assign_tempo = Self::MAX_SUPPORTED_REAL_TEMPO;
        } else if real_tempo < Self::MIN_SUPPORTED_REAL_TEMPO {
            assign_tempo = Self::MIN_SUPPORTED_REAL_TEMPO;
        } else {
            assign_tempo = real_tempo;
        }
        Tempo(((assign_tempo - 20) / 2) as u8)
    }
}

#[derive(Copy, Clone, Eq, FromPrimitive, PartialEq, Default, Debug)]
#[repr(u8)]
pub enum DescriptiveTempo {
    Larghissimo = 0,
    Grave,
    Lento,
    Largo,
    Adagio,
    Adagietto,
    Andante,
    Moderato,
    Allegretto,
    #[default]
    Allegro,
    Vivace,
    Presto,
    Prestissimo,
}

impl From<Tempo> for DescriptiveTempo {
    fn from(tempo: Tempo) -> Self {
        let val = tempo.get_actual();
        if val <= 24 {
            DescriptiveTempo::Larghissimo
        } else if val <= 40 {
            DescriptiveTempo::Grave
        } else if val <= 45 {
            DescriptiveTempo::Lento
        } else if val <= 50 {
            DescriptiveTempo::Largo
        } else if val <= 65 {
            DescriptiveTempo::Adagio
        } else if val <= 69 {
            DescriptiveTempo::Adagietto
        } else if val <= 77 {
            DescriptiveTempo::Andante
        } else if val <= 97 {
            DescriptiveTempo::Moderato
        } else if val <= 120 {
            DescriptiveTempo::Allegretto
        } else if val <= 150 {
            DescriptiveTempo::Allegro
        } else if val <= 176 {
            DescriptiveTempo::Vivace
        } else if val <= 200 {
            DescriptiveTempo::Presto
        } else {
            DescriptiveTempo::Prestissimo
        }
    }
}

impl ToString for DescriptiveTempo {
    fn to_string(&self) -> String {
        match self {
            DescriptiveTempo::Larghissimo => String::from("Larghissimo"),
            DescriptiveTempo::Grave => String::from("Grave"),
            DescriptiveTempo::Lento => String::from("Lento"),
            DescriptiveTempo::Largo => String::from("Largo"),
            DescriptiveTempo::Adagio => String::from("Adagio"),
            DescriptiveTempo::Adagietto => String::from("Adagietto"),
            DescriptiveTempo::Andante => String::from("Andante"),
            DescriptiveTempo::Moderato => String::from("Moderato"),
            DescriptiveTempo::Allegretto => String::from("Allegretto"),
            DescriptiveTempo::Allegro => String::from("Allegro"),
            DescriptiveTempo::Vivace => String::from("Vivace"),
            DescriptiveTempo::Presto => String::from("Presto"),
            DescriptiveTempo::Prestissimo => String::from("Prestissimo"),
        }
    }
}

impl FromStr for DescriptiveTempo {
    type Err = Error;
    fn from_str(input: &str) -> Result<DescriptiveTempo> {
        let val = u32::from_str(input)?;
        if val <= 24 {
            Ok(DescriptiveTempo::Larghissimo)
        } else if val <= 40 {
            Ok(DescriptiveTempo::Grave)
        } else if val <= 45 {
            Ok(DescriptiveTempo::Lento)
        } else if val <= 50 {
            Ok(DescriptiveTempo::Largo)
        } else if val <= 65 {
            Ok(DescriptiveTempo::Adagio)
        } else if val <= 69 {
            Ok(DescriptiveTempo::Adagietto)
        } else if val <= 77 {
            Ok(DescriptiveTempo::Andante)
        } else if val <= 97 {
            Ok(DescriptiveTempo::Moderato)
        } else if val <= 120 {
            Ok(DescriptiveTempo::Allegretto)
        } else if val <= 150 {
            Ok(DescriptiveTempo::Allegro)
        } else if val <= 176 {
            Ok(DescriptiveTempo::Vivace)
        } else if val <= 200 {
            Ok(DescriptiveTempo::Presto)
        } else {
            Ok(DescriptiveTempo::Prestissimo)
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, FromPrimitive, Default, Debug)]
#[repr(u8)]
pub enum DalSegno {
    #[default]
    None = 0,
    SegnoMarker,
    CodaMarker,
    DaSegno,
    DaCapo,
    DaCapoalSegno,
    DaCapoAlCoda,
    DaCapoAlFine,
}

// #[derive(Eq, PartialEq, Copy, Clone, FromPrimitive, Default, Debug)]
// #[repr(u8)]
// pub enum TrebleBassClef {
//     #[default]
//     TrebleClef,
//     BassClef,
// }

// impl From<TrebleBassClef> for bool {
//     fn from(f: TrebleBassClef) -> bool {
//         match f {
//             TrebleBassClef::TrebleClef => false,
//             TrebleBassClef::BassClef => true,
//         }
//     }
// }

#[derive(Eq, PartialEq, PartialOrd, Ord, Copy, Clone, FromPrimitive, Default, Debug)]
#[repr(u8)]
pub enum NoteType {
    SemiHemiDemiSemiQuaver,
    HemiDemiSemiQuaver,
    DemiSemiQuaver,
    SemiQuaver,
    Quaver,
    #[default]
    Crochet,
    Minim,
    SemiBreve,
}

impl FromStr for NoteType {
    type Err = Error;
    fn from_str(input: &str) -> Result<NoteType> {
        match input {
            "breve" => Ok(NoteType::SemiBreve),
            "whole" => Ok(NoteType::SemiBreve),
            "half" => Ok(NoteType::Minim),
            "quarter" => Ok(NoteType::Crochet),
            "eighth" => Ok(NoteType::Quaver),
            "16th" => Ok(NoteType::SemiQuaver),
            "32nd" => Ok(NoteType::DemiSemiQuaver),
            "64th" => Ok(NoteType::HemiDemiSemiQuaver),
            "128th" => Ok(NoteType::SemiHemiDemiSemiQuaver),
            s => {
                error!("Unhandled Note type {}", s);
                Err(Error::Parse)
            }
        }
    }
}

impl NoteType {
    pub fn get_type_string(self) -> String {
        match self {
            NoteType::SemiHemiDemiSemiQuaver => String::from("128th"),
            NoteType::HemiDemiSemiQuaver => String::from("64th"),
            NoteType::DemiSemiQuaver => String::from("32nd"),
            NoteType::SemiQuaver => String::from("16th"),
            NoteType::Quaver => String::from("eighth"),
            NoteType::Crochet => String::from("quarter"),
            NoteType::Minim => String::from("half"),
            NoteType::SemiBreve => String::from("whole"),
        }
    }
}

#[derive(Copy, Clone, Eq, FromPrimitive, PartialEq, Default, Debug)]
#[repr(u8)]
pub enum Beats {
    Two = 0,
    Three,
    #[default]
    Four,
    Five,
    Six,
    Nine,
    Twelve,
}

impl From<Beats> for u32 {
    fn from(b: Beats) -> Self {
        match b {
            Beats::Two => 2,
            Beats::Three => 3,
            Beats::Four => 4,
            Beats::Five => 5,
            Beats::Six => 6,
            Beats::Nine => 9,
            Beats::Twelve => 12,
        }
    }
}

#[derive(Copy, Clone, Debug, Default, FromPrimitive, PartialEq)]
#[repr(u8)]
pub enum Staff {
    #[default]
    TrebleClef = 1,
    BassClef = 2,
}

#[derive(Copy, Clone, Eq, FromPrimitive, PartialEq, Default, Debug)]
#[repr(u8)]
pub enum Voice {
    #[default]
    One = 0,
    Two,
    Three,
    Four,
}

impl Voice {
    pub fn next(&self) -> Voice {
        match self {
            Voice::One => Voice::Two,
            Voice::Two => Voice::Three,
            Voice::Three => Voice::Four,
            Voice::Four => Voice::One,
        }
    }
}

impl ToString for Beats {
    fn to_string(&self) -> String {
        match self {
            Beats::Two => String::from("2"),
            Beats::Three => String::from("3"),
            Beats::Four => String::from("4"),
            Beats::Five => String::from("5"),
            Beats::Six => String::from("6"),
            Beats::Nine => String::from("9"),
            Beats::Twelve => String::from("12"),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, FromPrimitive, Default, Debug)]
#[repr(u8)]
pub enum BeatType {
    Two = 0,
    #[default]
    Four,
    Eight,
    Sixteen,
}

impl From<BeatType> for u32 {
    fn from(b: BeatType) -> Self {
        match b {
            BeatType::Two => 2,
            BeatType::Four => 4,
            BeatType::Eight => 8,
            BeatType::Sixteen => 16,
        }
    }
}

impl ToString for BeatType {
    fn to_string(&self) -> String {
        match self {
            BeatType::Two => String::from("2"),
            BeatType::Four => String::from("4"),
            BeatType::Eight => String::from("8"),
            BeatType::Sixteen => String::from("16"),
        }
    }
}

impl FromStr for Beats {
    type Err = Error;
    fn from_str(input: &str) -> Result<Beats> {
        match input {
            "2" => Ok(Beats::Two),
            "3" => Ok(Beats::Three),
            "4" => Ok(Beats::Four),
            "5" => Ok(Beats::Five),
            "6" => Ok(Beats::Six),
            "9" => Ok(Beats::Nine),
            "12" => Ok(Beats::Twelve),
            _ => Err(Error::Parse),
        }
    }
}

impl FromStr for BeatType {
    type Err = Error;
    fn from_str(input: &str) -> Result<BeatType> {
        match input {
            "2" => Ok(BeatType::Two),
            "4" => Ok(BeatType::Four),
            "8" => Ok(BeatType::Eight),
            "16" => Ok(BeatType::Sixteen),
            _ => Err(Error::Parse),
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum MusicElement {
    MeasureInit(MeasureInitializer),
    MeasureMeta(MeasureMetaData),
    NoteRest(NoteData),
    Tuplet(TupletData),
}

#[derive(Eq, PartialEq, Copy, Clone, Default, Debug)]
pub struct MeasureInitializer {
    pub beats: Beats,
    pub beat_type: BeatType,
    pub key_sig: KeySignature,
    pub tempo: Tempo,
}

#[derive(Eq, PartialEq, Default, Clone, Copy, Debug)]
pub struct MeasureMetaData {
    pub start_end: MeasureStartEnd,
    pub ending: Ending,
    pub dal_segno: DalSegno,
}

impl MeasureMetaData {
    pub fn new(measure_type: MeasureStartEnd) -> MeasureMetaData {
        MeasureMetaData {
            start_end: measure_type,
            ending: Ending::default(),
            dal_segno: DalSegno::default(),
        }
    }
}
#[derive(Eq, PartialEq, Default, Clone, Copy, Debug)]
pub struct NoteData {
    pub note_rest: NoteRestValue,
    pub phrase_dynamics: PhraseDynamics,
    pub note_type: NoteType,
    pub dotted: bool,
    pub arpeggiate: Arpeggiate,
    pub special_note: SpecialNote,
    pub articulation: Articulation,
    pub trill: Trill,
    pub ties: NoteConnection,
    pub stress: Stress,
    pub chord: Chord,
    pub slur: SlurConnection,
    pub voice: Voice,
}

type IsDotted = bool;

impl NoteData {
    const MIDI_TICKS_SEMI_HEMI_DEMI_SEMI_QUAVER: u32 = 30;
    const MIDI_TICKS_HEMI_DEMI_SEMI_QUAVER: u32 = 60;
    const MIDI_TICKS_DEMI_SEMI_QUAVER: u32 = 120;
    const MIDI_TICKS_SEMI_QUAVER: u32 = 240;
    const MIDI_TICKS_QUAVER: u32 = 480;
    const MIDI_TICKS_CROCHET: u32 = 960;
    const MIDI_TICKS_MINIM: u32 = 1920;
    const MIDI_TICKS_SEMIBREVE: u32 = 3840;

    pub fn new_default_rest(note_type: NoteType, is_dotted: IsDotted, voice: Voice) -> NoteData {
        let mut note = NoteData::default();
        note.note_rest = NoteRestValue::Rest;
        note.note_type = note_type;
        note.dotted = is_dotted;
        note.voice = voice;
        note
    }

    pub fn get_duration_in_midi_ticks(&self) -> u32 {
        match self.note_type {
            NoteType::SemiBreve => {
                Self::MIDI_TICKS_SEMIBREVE
                    + match self.dotted {
                        true => Self::MIDI_TICKS_SEMIBREVE / 2,
                        false => 0,
                    }
            }
            NoteType::Minim => {
                Self::MIDI_TICKS_MINIM
                    + match self.dotted {
                        true => Self::MIDI_TICKS_MINIM / 2,
                        false => 0,
                    }
            }
            NoteType::Crochet => {
                Self::MIDI_TICKS_CROCHET
                    + match self.dotted {
                        true => Self::MIDI_TICKS_CROCHET / 2,
                        false => 0,
                    }
            }
            NoteType::Quaver => {
                Self::MIDI_TICKS_QUAVER
                    + match self.dotted {
                        true => Self::MIDI_TICKS_QUAVER / 2,
                        false => 0,
                    }
            }
            NoteType::SemiQuaver => {
                Self::MIDI_TICKS_SEMI_QUAVER
                    + match self.dotted {
                        true => Self::MIDI_TICKS_SEMI_QUAVER / 2,
                        false => 0,
                    }
            }
            NoteType::DemiSemiQuaver => {
                Self::MIDI_TICKS_DEMI_SEMI_QUAVER
                    + match self.dotted {
                        true => Self::MIDI_TICKS_DEMI_SEMI_QUAVER / 2,
                        false => 0,
                    }
            }
            NoteType::HemiDemiSemiQuaver => {
                Self::MIDI_TICKS_HEMI_DEMI_SEMI_QUAVER
                    + match self.dotted {
                        true => Self::MIDI_TICKS_HEMI_DEMI_SEMI_QUAVER / 2,
                        false => 0,
                    }
            }
            NoteType::SemiHemiDemiSemiQuaver => {
                Self::MIDI_TICKS_SEMI_HEMI_DEMI_SEMI_QUAVER
                    + match self.dotted {
                        true => Self::MIDI_TICKS_SEMI_HEMI_DEMI_SEMI_QUAVER / 2,
                        false => 0,
                    }
            }
        }
    }

    pub fn get_duration_numeric(
        &self,
        divisions: u32,
        beats: u32,
        beat_type: u32,
        time_mods: Option<TimeModification>,
    ) -> u32 {
        if self.special_note != SpecialNote::None {
            // Some notes have no duration
            return 0;
        }
        let mut numerator: u32 = 2;
        let mut denominator: u32 = 2;
        if self.dotted {
            numerator = 3;
        }
        if time_mods.is_some() {
            numerator *= time_mods.unwrap().normal_notes as u32;
            denominator *= time_mods.unwrap().actual_notes as u32;
        }

        match self.note_type {
            NoteType::SemiHemiDemiSemiQuaver => (divisions * numerator / 32) / denominator,
            NoteType::HemiDemiSemiQuaver => (divisions * numerator / 16) / denominator,
            NoteType::DemiSemiQuaver => (divisions * numerator / 8) / denominator,
            NoteType::SemiQuaver => (divisions * numerator / 4) / denominator,
            NoteType::Quaver => (divisions * numerator / 2) / denominator,
            NoteType::Crochet => (divisions * numerator) / denominator,
            NoteType::Minim => (divisions * 2 * numerator) / denominator,
            NoteType::SemiBreve => {
                // The duration of a semi breve rest can differ based on time signature.
                // For example, in 4/4, it would be 4 crochets, but in 3/4, only 3 crochets
                if self.note_rest == NoteRestValue::Rest {
                    ((divisions * numerator * beats * 10) / (beat_type * 10)) / denominator
                } else {
                    (divisions * 4 * numerator) / denominator
                }
            }
        }
    }

    pub fn get_duration_string(
        &self,
        divisions: u32,
        beats: u32,
        beat_type: u32,
        time_mod: Option<TimeModification>,
    ) -> String {
        self.get_duration_numeric(divisions, beats, beat_type, time_mod)
            .to_string()
    }

    /// Converts a numeric duration to its corresponding musical `Duration` and `IsDotted` representation.
    ///
    /// # Arguments
    ///
    /// * `numeric_duration` - The numeric value representing the duration of the note.
    /// * `quarter_division` - The number of divisions that represent a quarter note (crochet).
    ///
    /// # Returns
    ///
    /// Returns an `Option` containing a tuple of `Duration` and `IsDotted` if the `numeric_duration`
    /// matches a standard musical note duration. Returns `None` if the `numeric_duration` doesn't fit
    /// standard note values.
    ///
    /// # Examples
    ///
    /// ```
    /// # use your_crate_name::{Duration, IsDotted, from_numeric_duration};
    /// let divisions = 480;
    /// let numeric = 720;
    ///
    /// assert_eq!(
    ///     from_numeric_duration(numeric, divisions),
    ///     Some((Duration::QuarterNote, IsDotted::Dotted))
    /// );
    pub fn from_numeric_duration(
        numeric_duration: u32,
        quarter_division: u32,
    ) -> Option<(NoteType, IsDotted)> {
        let is_dotted = |standard_duration: u32| {
            if (standard_duration * 3) % 2 != 0 {
                panic!("Invalid {standard_duration}");
            }
            numeric_duration == standard_duration * 3 / 2
        };
        match numeric_duration {
            _ if is_dotted(4 * quarter_division) => Some((NoteType::SemiBreve, true)),
            _ if numeric_duration == 4 * quarter_division => Some((NoteType::SemiBreve, false)),
            _ if is_dotted(2 * quarter_division) => Some((NoteType::Minim, true)),
            _ if numeric_duration == 2 * quarter_division => Some((NoteType::Minim, false)),
            _ if is_dotted(quarter_division) => Some((NoteType::Crochet, true)),
            _ if numeric_duration == quarter_division => Some((NoteType::Crochet, false)),
            _ if is_dotted(quarter_division / 2) => Some((NoteType::Quaver, true)),
            _ if numeric_duration == quarter_division / 2 => Some((NoteType::Quaver, false)),
            _ if is_dotted(quarter_division / 4) => Some((NoteType::SemiQuaver, true)),
            _ if numeric_duration == quarter_division / 4 => Some((NoteType::SemiQuaver, false)),
            //_ if numeric_duration == quarter_division / 6 => Some((Duration::SemiQuaver, false)),
            _ if is_dotted(quarter_division / 8) => Some((NoteType::DemiSemiQuaver, true)),
            _ if numeric_duration == quarter_division / 8 => {
                Some((NoteType::DemiSemiQuaver, false))
            }
            _ if is_dotted(quarter_division / 16) => Some((NoteType::HemiDemiSemiQuaver, true)),
            _ if numeric_duration == quarter_division / 16 => {
                Some((NoteType::HemiDemiSemiQuaver, false))
            }
            _ if is_dotted(quarter_division / 32) => Some((NoteType::SemiHemiDemiSemiQuaver, true)),
            _ if numeric_duration == quarter_division / 32 => {
                Some((NoteType::SemiHemiDemiSemiQuaver, false))
            }
            _ => {
                error!("Unsupported combination: Numeric_Duration: {numeric_duration} Qtr_Note_Divisions: {quarter_division}");
                None
            } // Note can't be represented using this quarter_division
        }
    }
}
#[derive(Eq, PartialEq, Default, Clone, Copy, Debug)]
#[repr(u8)]
pub enum NoteRestValue {
    #[default]
    Rest = 0,
    Pitch(u8),
}

impl NoteRestValue {
    const MAX_NOTE_VALUE: i8 = 97;
    const MIN_NOTE_VALUE: i8 = 1;
    const REST_VALUE: u8 = 0;

    fn get_octave(self) -> Option<Octave> {
        match self {
            NoteRestValue::Rest => None,
            NoteRestValue::Pitch(v) => {
                let num = (v - 1) / 12;
                FromPrimitive::from_u8(num)
            }
        }
    }

    pub fn new_from_numeric(note_val: u8) -> NoteRestValue {
        if note_val == 0 {
            NoteRestValue::Rest
        } else {
            NoteRestValue::Pitch(note_val)
        }
    }
    /// Encodes note data into numerical form for embedding. Supported note range is C0 to C8
    ///
    /// # Arguments
    ///
    /// * `note`  -  An enum indicating the step of the 12 tone equal temperament diatonic scale
    /// * `alter` -  An enum indicating if note should be unchanged, flatted, or sharped
    /// * `octave` - An enum indicating the octave from 0 to 8
    ///
    pub fn derive_numeric_note(note: Note, alter: Alter, octave: Octave) -> Result<NoteRestValue> {
        let mut numeric_note = note as i8;
        let numeric_alter = alter as i8;
        let numeric_octave = octave as i8;

        numeric_note += numeric_alter;
        numeric_note += (numeric_octave - 4) * 12;
        if numeric_note > Self::MAX_NOTE_VALUE {
            Err(Error::OutofBounds)
        } else if numeric_note < Self::MIN_NOTE_VALUE {
            Err(Error::OutofBounds)
        } else {
            Ok(NoteRestValue::Pitch(numeric_note as u8))
        }
    }

    pub fn decode_composite_note(self) -> Option<(Note, Alter, Octave)> {
        if let Some(octave) = self.get_octave() {
            match self {
                NoteRestValue::Rest => return None,
                NoteRestValue::Pitch(v) => {
                    let oct_u8 = octave as u8;
                    let note_numeric = (v as i32) - (((oct_u8 as i32) - 4) * 12);
                    let note: Option<Note> = FromPrimitive::from_u8(note_numeric as u8);
                    match note {
                        None => return None,
                        Some(n) => {
                            let (somenote, somealter) = n.get_note_alter_tuple();
                            return Some((somenote, somealter, octave));
                        }
                    }
                }
            }
        } else {
            return None;
        }
    }

    pub fn get_numeric_value(self) -> u8 {
        match self {
            NoteRestValue::Rest => NoteRestValue::REST_VALUE,
            NoteRestValue::Pitch(v) => v,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tempo_into() {
        let value: Tempo = 30.into();
        assert_eq!(value.0, 5);
    }

    #[test]
    fn test_encode_note() {
        let mut note = Note::NoteC;
        let mut alter = Alter::None;
        let mut octave = Octave::Octave0;

        assert_eq!(
            NoteRestValue::derive_numeric_note(note, alter, octave),
            Ok(NoteRestValue::Pitch(1))
        );

        alter = Alter::Sharp;
        assert_eq!(
            NoteRestValue::derive_numeric_note(note, alter, octave),
            Ok(NoteRestValue::Pitch(2))
        );

        alter = Alter::None;
        octave = Octave::Octave8;
        assert_eq!(
            NoteRestValue::derive_numeric_note(note, alter, octave),
            Ok(NoteRestValue::Pitch(97))
        );

        alter = Alter::Flat;
        assert_eq!(
            NoteRestValue::derive_numeric_note(note, alter, octave),
            Ok(NoteRestValue::Pitch(96))
        );

        note = Note::NoteD;
        alter = Alter::None;
        octave = Octave::Octave8;
        assert_eq!(
            NoteRestValue::derive_numeric_note(note, alter, octave),
            Err(Error::OutofBounds)
        );
        note = Note::NoteC;
        alter = Alter::Sharp;
        octave = Octave::Octave8;
        assert_eq!(
            NoteRestValue::derive_numeric_note(note, alter, octave),
            Err(Error::OutofBounds)
        );
        note = Note::NoteC;
        alter = Alter::Flat;
        octave = Octave::Octave0;
        assert_eq!(
            NoteRestValue::derive_numeric_note(note, alter, octave),
            Err(Error::OutofBounds)
        );
    }
}
