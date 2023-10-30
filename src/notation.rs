use crate::error::{Error, Result};
use crate::music_xml_types::{TimeModificationElement, TupletElement, TupletType, ArticulationValue};
use fraction::Fraction;
use log::{error, info, trace, warn};
use num::integer::lcm;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::convert::From;
use std::str::FromStr;
use strum::{EnumCount, EnumIter};

pub struct MeasureChecker {
    measure: Vec<MusicElement>,
    elems_since_backup: usize,
    quarter_division: u32,
    beats: Beats,
    beat_type: BeatType,
    measure_idx: usize,
}

impl MeasureChecker {
    pub const MAX_SUPPORTED_VOICES: usize = 4;
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
        //debug!("{:?}", elem);
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
        let mut current_voice = Voice::One;
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
                        // Chord notes do not directly impact duration
                        0
                    }
                }
                MusicElement::Tuplet(t) => {
                    time_mod = t.into();
                    0 // does not directly impact sum
                }
                _ => {
                    0 // does not impact sum
                }
            })
            .sum();

        match backup_duration.cmp(&duration_since_backup) {
            Ordering::Less => {
                let discrepancy = duration_since_backup - backup_duration;
                println!("M{} duration tally {} did not match the backup element's duration {backup_duration}, qtr_div: {} inserting rests to accommodate {discrepancy} discrepancy.", self.measure_idx, duration_since_backup, self.quarter_division);

                match NoteData::from_numeric_duration(discrepancy as u32, self.quarter_division) {
                    Some((duration, is_dotted, time_mod)) => {
                        if time_mod.is_some() {
                            warn!("time modification for rest is present, but not being used.")
                        }
                        // The new rest should begin on the next voice after the current one.
                        self.measure
                            .push(MusicElement::NoteRest(NoteData::new_default_rest(
                                duration,
                                is_dotted,
                                current_voice.next(),
                            )));
                    }
                    None => {
                        panic!(
                            "Could not convert {} in a rest duration value.",
                            discrepancy
                        );
                    }
                }
            }
            Ordering::Greater => {
                info!(
                    "Backup_duration {} was > duration_since_backup {} Assuming beginning of measure",
                    backup_duration, duration_since_backup
                );
            }
            Ordering::Equal => {
                // No additional action needed
            }
        }

        self.clear_elems_since_backup();
    }

    fn clear_elems_since_backup(&mut self) {
        self.elems_since_backup = 0;
    }

    pub fn as_inner(&mut self) -> &mut Vec<MusicElement> {
        &mut self.measure
    }

    pub fn remove_incomplete_voices(&mut self, voices: &BTreeSet<u8>) {
        let mut voice_durations: [u32; Self::MAX_SUPPORTED_VOICES] =
            [0; Self::MAX_SUPPORTED_VOICES];
        let mut voice_last_idx: [usize; Self::MAX_SUPPORTED_VOICES] =
            [0; Self::MAX_SUPPORTED_VOICES];

        if voices.len() > Self::MAX_SUPPORTED_VOICES {
            panic!(
                "Set of voices len {} exceeds max supported {}",
                voices.len(),
                Self::MAX_SUPPORTED_VOICES
            );
        }
        let mut time_mod = None;
        let mut prev_voice = 0;
        // TODO: enumerate the iterator and create mapping of the last element of each voice
        // so a rest can be inserted at that location later if the voice duration is insufficient
        for (idx, elem) in self.measure.iter().cloned().enumerate() {
            match elem {
                MusicElement::Tuplet(t) => time_mod = t.into(),
                MusicElement::NoteRest(n) => {
                    // Do not include chord notes or grace notes in the count, as they do not impact measure duration
                    if n.chord == Chord::NoChord && n.special_note == SpecialNote::None {
                        voice_durations[n.voice as usize] += n.get_duration_numeric(
                            self.quarter_division,
                            u32::from(self.beats),
                            u32::from(self.beat_type),
                            time_mod,
                        )
                    }
                    if n.voice as usize > prev_voice {
                        voice_last_idx[n.voice as usize] = idx - 1;
                    }
                    prev_voice = n.voice as usize;
                }
                _ => {
                    error!("Unhandled element case");
                }
            }
        }

        let first_voice_duration = voice_durations[0];
        for (voice_idx, _) in voices.iter().enumerate() {
            if voice_durations[voice_idx] != 0 && voice_durations[voice_idx] < first_voice_duration
            {
                let discrepancy = first_voice_duration - voice_durations[voice_idx];
                println!(
                    "M{} Voice Zero: {first_voice_duration} duration Voice {voice_idx}: {} duration {} discrepancy", self.measure_idx,
                    voice_durations[voice_idx],discrepancy
                );
                // insert rest of discrepancy length at index at measure[voice_last_idx[voice_idx]]
                if let Some((duration, is_dotted, time_mod)) =
                    NoteData::from_numeric_duration(discrepancy, self.quarter_division)
                {
                    if time_mod.is_some() {
                        warn!("time modification for rest is present, but not being used.")
                    }
                    // The new rest should begin on the next voice after the current one.
                    self.measure.insert(
                        voice_last_idx[voice_idx],
                        MusicElement::NoteRest(NoteData::new_default_rest(
                            duration,
                            is_dotted,
                            FromPrimitive::from_u8(voice_idx as u8).unwrap(),
                        )),
                    );
                } else {
                    panic!(
                        "Could not convert {} in a rest duration value.",
                        discrepancy
                    );
                }
            }
        }
    }
}

struct DivisionsVec {
    inner: Vec<u32>,
}

impl DivisionsVec {
    // Create a new, empty DivisionsVec
    pub fn new() -> Self {
        DivisionsVec { inner: vec![] }
    }

    // Add an item to the DivisionsVec, but only if it's not already present
    pub fn add(&mut self, value: u32) {
        if value != 0 && !self.inner.contains(&value) {
            self.inner.push(value);
        }
    }

    pub fn find_lcm(&mut self) -> u32 {
        self.inner.iter().fold(1, |acc, &n| lcm(acc, n))
    }

    // Allow direct access to the inner Vec<u32>
    pub fn inner(&self) -> &Vec<u32> {
        &self.inner
    }
}

pub fn calc_divisions_voices(music_elems_v: Vec<MusicElement>, dump_input: bool) -> (u32, usize) {
    let mut voices = BTreeSet::new();
    let mut integers_v = DivisionsVec::new();
    let mut time_mod = None;

    for elem in music_elems_v.iter() {
        if dump_input {
            trace!("{:?}", elem);
        }
        match elem {
            MusicElement::Tuplet(t) => {
                time_mod = (*t).into();
            }
            MusicElement::NoteRest(n) => {
                voices.insert(n.voice as u8);
                integers_v.add(n.get_note_multiple(time_mod).map_or_else(|| 0, |v| v));
            }
            _ => {}
        }
    }

    // for (idx, elem) in integers_v.inner().iter().enumerate() {
    //     println!("{idx},{elem}");
    // }

    (integers_v.find_lcm(), voices.len())
}

#[derive(Eq, PartialEq, Default, Debug, Copy, Clone)]
pub struct TimeModification {
    actual_notes: TupletActual,
    normal_notes: TupletNormal,
}

fn convert_time_modification(t_mod: &TimeModificationElement) -> TimeModification {
    let tup_ac = TupletActual::try_from(t_mod.actual_notes.as_ref())
        .expect("Cannot convert this TupletActual string.");
    let tup_norm = TupletNormal::try_from(t_mod.normal_notes.as_ref())
        .expect("Cannot convert this TupletNormal string.");

    TimeModification::new(tup_ac, tup_norm)
}

impl From<TimeModificationElement> for TimeModification {
    fn from(time_mod_elem: TimeModificationElement) -> Self {
        convert_time_modification(&time_mod_elem)
    }
}

impl From<&TimeModificationElement> for TimeModification {
    fn from(time_mod_elem: &TimeModificationElement) -> Self {
        convert_time_modification(time_mod_elem)
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
    None = 0,
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
    None = 0,
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
    Accent,
    StrongAccent,
    Stacatto,
    Staccatissimo,
    Tenuto,
    DetachedLegato,
    Stress,
}

impl From<Articulation> for ArticulationValue {
    fn from(t: Articulation) -> Self {
        match t {
            Articulation::None => ArticulationValue::None,
            Articulation::Accent => ArticulationValue::Accent,
            Articulation::StrongAccent => ArticulationValue::StrongAccent,
            Articulation::Stacatto => ArticulationValue::Stacatto,
            Articulation::Staccatissimo => ArticulationValue::Staccatissimo,
            Articulation::Tenuto => ArticulationValue::Tenuto,
            Articulation::DetachedLegato => ArticulationValue::DetachedLegato,
            Articulation::Stress => ArticulationValue::Stress,
        }
    }
}

impl ToString for Articulation {
    fn to_string(&self) -> String {
        match self {
            Articulation::None => "".to_string(),
            Articulation::Accent => "accent".to_string(),
            Articulation::StrongAccent => "strong-accent".to_string(),
            Articulation::Stacatto => "staccato".to_string(),
            Articulation::Staccatissimo => "staccatissimo".to_string(),
            Articulation::Tenuto => "tenuto".to_string(),
            Articulation::DetachedLegato => "detached-legato".to_string(),
            Articulation::Stress => "stress".to_string(),
        }
    }
}

impl FromStr for Articulation {
    type Err = Error;
    fn from_str(input: &str) -> Result<Articulation> {
        match input {
            "accent" => Ok(Articulation::Accent),
            "strong-accent" => Ok(Articulation::StrongAccent),
            "staccato" => Ok(Articulation::Stacatto),
            "tenuto" => Ok(Articulation::Tenuto),
            "detached-legato" => Ok(Articulation::DetachedLegato),
            "staccatissimo" => Ok(Articulation::Staccatissimo),
            "spiccato" => Ok(Articulation::Staccatissimo),
            "stress" => Ok(Articulation::Stress),
            _ => {
                // Unsupported articulation tag
                Ok(Articulation::None)
            }
        }
    }
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
pub enum Chord {
    #[default]
    NoChord,
    Chord,
}

// TupletNumber is used for tracking tuplets when they are nested
#[derive(Eq, PartialEq, Copy, Clone, FromPrimitive, EnumCount, EnumIter, Default, Debug)]
#[repr(u8)]
pub enum TupletNumber {
    #[default]
    One,
    Two,
    Three,
    Four,
}

impl ToString for TupletNumber {
    fn to_string(&self) -> String {
        match self {
            TupletNumber::One => "1".to_string(),
            TupletNumber::Two => "2".to_string(),
            TupletNumber::Three => "3".to_string(),
            TupletNumber::Four => "4".to_string(),
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, FromPrimitive, Default, Debug)]
#[repr(u8)]
pub enum TupletStartStop {
    #[default]
    None,
    TupletStart,
    TupletStop,
}

trait AsU32 {
    fn as_u32(&self) -> u32;
}

#[derive(Eq, PartialEq, Copy, Clone, FromPrimitive, Default, Debug)]
#[repr(u8)]
pub enum TupletActual {
    #[default]
    Two = 0,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Eleven,
    Thirteen,
    Fifteen,
    Sixteen,
    Seventeen,
    Eighteen,
    TwentyOne,
    TwentyFive,
}

impl AsU32 for TupletActual {
    fn as_u32(&self) -> u32 {
        match self {
            TupletActual::Two => 2,
            TupletActual::Three => 3,
            TupletActual::Four => 4,
            TupletActual::Five => 5,
            TupletActual::Six => 6,
            TupletActual::Seven => 7,
            TupletActual::Eight => 8,
            TupletActual::Nine => 9,
            TupletActual::Ten => 10,
            TupletActual::Eleven => 11,
            TupletActual::Thirteen => 13,
            TupletActual::Fifteen => 15,
            TupletActual::Sixteen => 16,
            TupletActual::Seventeen => 17,
            TupletActual::Eighteen => 18,
            TupletActual::TwentyOne => 21,
            TupletActual::TwentyFive => 25,
        }
    }
}

impl TryFrom<u32> for TupletActual {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self> {
        match value {
            2 => Ok(TupletActual::Two),
            3 => Ok(TupletActual::Three),
            4 => Ok(TupletActual::Four),
            5 => Ok(TupletActual::Five),
            6 => Ok(TupletActual::Six),
            7 => Ok(TupletActual::Seven),
            8 => Ok(TupletActual::Eight),
            9 => Ok(TupletActual::Nine),
            10 => Ok(TupletActual::Ten),
            11 => Ok(TupletActual::Eleven),
            13 => Ok(TupletActual::Thirteen),
            15 => Ok(TupletActual::Fifteen),
            16 => Ok(TupletActual::Sixteen),
            17 => Ok(TupletActual::Seventeen),
            18 => Ok(TupletActual::Eighteen),
            21 => Ok(TupletActual::TwentyOne),
            25 => Ok(TupletActual::TwentyFive),
            _ => Err(Error::Unit),
        }
    }
}

impl TryFrom<&str> for TupletActual {
    type Error = Error;
    fn try_from(inp_string: &str) -> Result<Self> {
        match inp_string {
            "2" => Ok(TupletActual::Two),
            "3" => Ok(TupletActual::Three),
            "4" => Ok(TupletActual::Four),
            "5" => Ok(TupletActual::Five),
            "6" => Ok(TupletActual::Six),
            "7" => Ok(TupletActual::Seven),
            "8" => Ok(TupletActual::Eight),
            "9" => Ok(TupletActual::Nine),
            "10" => Ok(TupletActual::Ten),
            "11" => Ok(TupletActual::Eleven),
            "13" => Ok(TupletActual::Thirteen),
            "15" => Ok(TupletActual::Fifteen),
            "16" => Ok(TupletActual::Sixteen),
            "17" => Ok(TupletActual::Seventeen),
            "18" => Ok(TupletActual::Eighteen),
            "21" => Ok(TupletActual::TwentyOne),
            "25" => Ok(TupletActual::TwentyFive),
            _ => Err(Error::Unit),
        }
    }
}

impl FromStr for TupletActual {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "2" => Ok(TupletActual::Two),
            "3" => Ok(TupletActual::Three),
            "4" => Ok(TupletActual::Four),
            "5" => Ok(TupletActual::Five),
            "6" => Ok(TupletActual::Six),
            "7" => Ok(TupletActual::Seven),
            "8" => Ok(TupletActual::Eight),
            "9" => Ok(TupletActual::Nine),
            "10" => Ok(TupletActual::Ten),
            "11" => Ok(TupletActual::Eleven),
            "13" => Ok(TupletActual::Thirteen),
            "15" => Ok(TupletActual::Fifteen),
            "16" => Ok(TupletActual::Sixteen),
            "17" => Ok(TupletActual::Seventeen),
            "18" => Ok(TupletActual::Eighteen),
            "21" => Ok(TupletActual::TwentyOne),
            "25" => Ok(TupletActual::TwentyFive),
            _ => Err(Error::Unit),
        }
    }
}

impl From<TupletActual> for String {
    fn from(val: TupletActual) -> Self {
        match val {
            TupletActual::Two => "2".to_string(),
            TupletActual::Three => "3".to_string(),
            TupletActual::Four => "4".to_string(),
            TupletActual::Five => "5".to_string(),
            TupletActual::Six => "6".to_string(),
            TupletActual::Seven => "7".to_string(),
            TupletActual::Eight => "8".to_string(),
            TupletActual::Nine => "9".to_string(),
            TupletActual::Ten => "10".to_string(),
            TupletActual::Eleven => "11".to_string(),
            TupletActual::Thirteen => "13".to_string(),
            TupletActual::Fifteen => "15".to_string(),
            TupletActual::Sixteen => "16".to_string(),
            TupletActual::Seventeen => "17".to_string(),
            TupletActual::Eighteen => "18".to_string(),
            TupletActual::TwentyOne => "21".to_string(),
            TupletActual::TwentyFive => "25".to_string(),
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, FromPrimitive, Default, Debug)]
#[repr(u8)]
pub enum TupletNormal {
    #[default]
    One = 0,
    Two,
    Three,
    Four,
    Six,
    Eight,
    Nine,
    Twelve,
    Sixteen,
}

impl AsU32 for TupletNormal {
    fn as_u32(&self) -> u32 {
        match self {
            TupletNormal::One => 1,
            TupletNormal::Two => 2,
            TupletNormal::Three => 3,
            TupletNormal::Four => 4,
            TupletNormal::Six => 6,
            TupletNormal::Eight => 8,
            TupletNormal::Nine => 9,
            TupletNormal::Twelve => 12,
            TupletNormal::Sixteen => 16,
        }
    }
}

impl TryFrom<u32> for TupletNormal {
    type Error = Error;

    fn try_from(value: u32) -> Result<Self> {
        match value {
            1 => Ok(TupletNormal::One),
            2 => Ok(TupletNormal::Two),
            3 => Ok(TupletNormal::Three),
            4 => Ok(TupletNormal::Four),
            6 => Ok(TupletNormal::Six),
            8 => Ok(TupletNormal::Eight),
            9 => Ok(TupletNormal::Nine),
            12 => Ok(TupletNormal::Twelve),
            16 => Ok(TupletNormal::Sixteen),
            _ => Err(Error::Unit),
        }
    }
}

impl TryFrom<&str> for TupletNormal {
    type Error = Error;
    fn try_from(inp_string: &str) -> Result<Self> {
        match inp_string {
            "1" => Ok(TupletNormal::One),
            "2" => Ok(TupletNormal::Two),
            "3" => Ok(TupletNormal::Three),
            "4" => Ok(TupletNormal::Four),
            "6" => Ok(TupletNormal::Six),
            "8" => Ok(TupletNormal::Eight),
            "9" => Ok(TupletNormal::Nine),
            "12" => Ok(TupletNormal::Twelve),
            "16" => Ok(TupletNormal::Sixteen),
            _ => Err(Error::Unit),
        }
    }
}

impl FromStr for TupletNormal {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "1" => Ok(TupletNormal::One),
            "2" => Ok(TupletNormal::Two),
            "3" => Ok(TupletNormal::Three),
            "4" => Ok(TupletNormal::Four),
            "6" => Ok(TupletNormal::Six),
            "8" => Ok(TupletNormal::Eight),
            "9" => Ok(TupletNormal::Nine),
            "12" => Ok(TupletNormal::Twelve),
            "16" => Ok(TupletNormal::Sixteen),
            _ => Err(Error::Unit),
        }
    }
}

impl From<TupletNormal> for String {
    fn from(val: TupletNormal) -> Self {
        match val {
            TupletNormal::One => "1".to_string(),
            TupletNormal::Two => "2".to_string(),
            TupletNormal::Three => "3".to_string(),
            TupletNormal::Four => "4".to_string(),
            TupletNormal::Six => "6".to_string(),
            TupletNormal::Eight => "8".to_string(),
            TupletNormal::Nine => "9".to_string(),
            TupletNormal::Twelve => "12".to_string(),
            TupletNormal::Sixteen => "16".to_string(),
        }
    }
}

pub type TupletDotted = bool;

#[derive(Eq, PartialEq, Copy, Clone, Default, Debug)]
pub struct TupletData {
    pub start_stop: TupletStartStop,
    pub tuplet_number: TupletNumber,
    pub actual_notes: TupletActual,
    pub normal_notes: TupletNormal,
    pub dotted: TupletDotted,
}

impl From<TupletData> for Option<TupletElement> {
    fn from(t: TupletData) -> Self {
        match t.start_stop {
            TupletStartStop::TupletStart => Some(TupletElement {
                r#type: TupletType::Start,
                number: t.tuplet_number.to_string(),
            }),
            TupletStartStop::None => None,
            TupletStartStop::TupletStop => None,
        }
    }
}

impl From<TupletData> for Option<TimeModificationElement> {
    fn from(t: TupletData) -> Self {
        match t.start_stop {
            TupletStartStop::TupletStart => Some(TimeModificationElement {
                actual_notes: t.actual_notes.into(),
                normal_notes: t.normal_notes.into(),
            }),
            TupletStartStop::None => None,
            TupletStartStop::TupletStop => None,
        }
    }
}

impl From<TupletData> for Option<TimeModification> {
    fn from(t: TupletData) -> Self {
        match t.start_stop {
            TupletStartStop::TupletStart => {
                Some(TimeModification::new(t.actual_notes, t.normal_notes))
            }
            TupletStartStop::None => None,
            TupletStartStop::TupletStop => None,
        }
    }
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
        let assign_tempo: u8 = if raw_tempo > Self::MAX_SUPPORTED_RAW_TEMPO {
            Self::MAX_SUPPORTED_RAW_TEMPO
        } else {
            raw_tempo
        };

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
    pub chord: Chord,
    pub slur: SlurConnection,
    pub voice: Voice,
}

type IsDotted = bool;

impl NoteData {
    const SEMIBREVE_DENOMINATOR: u32 = 1;
    const MINIM_DENOMINATOR: u32 = 2;
    const CROCHET_DENOMINATOR: u32 = 4;
    const QUAVER_DENOMINATOR: u32 = 8;
    const SEMI_QUAVER_DENOMINATOR: u32 = 16;
    const DEMI_SEMI_QUAVER_DENOMINATOR: u32 = 32;
    const HEMI_DEMI_SEMI_QUAVER_DENOMINATOR: u32 = 64;
    const SEMI_HEMI_DEMI_SEMI_QUAVER_DENOMINATOR: u32 = 128;
    const IS_DOTTED_DENOMINATOR: u32 = 2;
    const IS_DOTTED_NUMERATOR: u32 = 3;

    const MIDI_TICKS_SEMI_HEMI_DEMI_SEMI_QUAVER: u32 = Self::MIDI_TICKS_CROCHET / 32;
    const MIDI_TICKS_HEMI_DEMI_SEMI_QUAVER: u32 = Self::MIDI_TICKS_CROCHET / 16;
    const MIDI_TICKS_DEMI_SEMI_QUAVER: u32 = Self::MIDI_TICKS_CROCHET / 8;
    const MIDI_TICKS_SEMI_QUAVER: u32 = Self::MIDI_TICKS_CROCHET / 4;
    const MIDI_TICKS_QUAVER: u32 = Self::MIDI_TICKS_CROCHET / 2;
    const MIDI_TICKS_CROCHET: u32 = 960;
    const MIDI_TICKS_MINIM: u32 = Self::MIDI_TICKS_CROCHET * 2;
    const MIDI_TICKS_SEMIBREVE: u32 = Self::MIDI_TICKS_CROCHET * 4;

    pub fn new_default_rest(note_type: NoteType, dotted: IsDotted, voice: Voice) -> NoteData {
        NoteData {
            note_rest: NoteRestValue::Rest,
            note_type,
            dotted,
            voice,
            ..Default::default()
        }
    }

    pub fn get_note_multiple(&self, time_mods: Option<TimeModification>) -> Option<u32> {
        let mut numer: u32 = 1;
        if self.special_note != SpecialNote::None {
            // Some notes have no duration
            return None;
        }

        let mut denom = match self.note_type {
            NoteType::SemiBreve => Self::SEMIBREVE_DENOMINATOR,
            NoteType::Minim => Self::MINIM_DENOMINATOR,
            NoteType::Crochet => Self::CROCHET_DENOMINATOR,
            NoteType::Quaver => Self::QUAVER_DENOMINATOR,
            NoteType::SemiQuaver => Self::SEMI_QUAVER_DENOMINATOR,
            NoteType::DemiSemiQuaver => Self::DEMI_SEMI_QUAVER_DENOMINATOR,
            NoteType::HemiDemiSemiQuaver => Self::HEMI_DEMI_SEMI_QUAVER_DENOMINATOR,
            NoteType::SemiHemiDemiSemiQuaver => Self::SEMI_HEMI_DEMI_SEMI_QUAVER_DENOMINATOR,
        };

        if self.dotted {
            numer *= Self::IS_DOTTED_NUMERATOR;
            denom *= Self::IS_DOTTED_DENOMINATOR;
        }

        if let Some(val) = time_mods {
            numer *= val.normal_notes.as_u32();
            denom *= val.actual_notes.as_u32();
        }
        let f = Fraction::new(numer, denom);
        //println!("{}",f);
        f.denom().map(|inner| *inner as u32)
    }

    pub fn get_duration_in_midi_ticks(&self, time_mods: Option<TimeModification>) -> u32 {
        let mut numerator: u32 = 1;
        let mut denominator: u32 = 1;

        if self.special_note != SpecialNote::None {
            // Some notes have no duration
            return 0;
        }

        if self.dotted {
            numerator = Self::IS_DOTTED_NUMERATOR;
            denominator = Self::IS_DOTTED_DENOMINATOR;
        }

        if let Some(val) = time_mods {
            numerator *= val.normal_notes.as_u32();
            denominator *= val.actual_notes.as_u32();
        }

        match self.note_type {
            NoteType::SemiBreve => Self::MIDI_TICKS_SEMIBREVE * numerator / denominator,
            NoteType::Minim => Self::MIDI_TICKS_MINIM * numerator / denominator,
            NoteType::Crochet => Self::MIDI_TICKS_CROCHET * numerator / denominator,
            NoteType::Quaver => Self::MIDI_TICKS_QUAVER * numerator / denominator,
            NoteType::SemiQuaver => Self::MIDI_TICKS_SEMI_QUAVER * numerator / denominator,
            NoteType::DemiSemiQuaver => Self::MIDI_TICKS_DEMI_SEMI_QUAVER * numerator / denominator,
            NoteType::HemiDemiSemiQuaver => {
                Self::MIDI_TICKS_HEMI_DEMI_SEMI_QUAVER * numerator / denominator
            }

            NoteType::SemiHemiDemiSemiQuaver => {
                Self::MIDI_TICKS_SEMI_HEMI_DEMI_SEMI_QUAVER * numerator / denominator
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

        let mut numerator: u32 = 1;
        let mut denominator: u32 = 1;

        if self.dotted {
            numerator = 3;
            denominator = 2;
        }

        if let Some(val) = time_mods {
            numerator *= val.normal_notes.as_u32();
            denominator *= val.actual_notes.as_u32();
            if denominator == 0 {
                panic!("time_mod denominator cannot be zero.");
            }
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
    ) -> Option<(NoteType, IsDotted, Option<TimeModification>)> {
        let note_types = [
            NoteType::SemiBreve,
            NoteType::Minim,
            NoteType::Crochet,
            NoteType::Quaver,
            NoteType::SemiQuaver,
            NoteType::DemiSemiQuaver,
            NoteType::HemiDemiSemiQuaver,
            NoteType::SemiHemiDemiSemiQuaver,
        ];

        let is_dotted = |base: u32, duration| (3 * base) / 2 == duration;

        // Start from a quarter note and expand in both directions.
        let mut base_duration = quarter_division;
        let mut exponent = 2; // index for quarter in note_types

        if is_dotted(base_duration, numeric_duration) {
            return Some((note_types[exponent], true, None));
        }

        while base_duration > numeric_duration && exponent < note_types.len() - 1 {
            base_duration /= 2;
            exponent += 1;
            if is_dotted(base_duration, numeric_duration) {
                return Some((note_types[exponent], true, None));
            }
        }

        while base_duration < numeric_duration && exponent > 0 {
            base_duration *= 2;
            exponent -= 1;
            if is_dotted(base_duration, numeric_duration) {
                return Some((note_types[exponent], true, None));
            }
        }

        // Check for time modification representation (tuplets)
        let mut tuplet_representation = None;
        for nn in 2..=16 {
            if nn == 5 || nn == 7 || nn == 10 || nn == 11 || nn == 13 || nn == 14 {
                // The TupletNormal type does not support these numerators
                continue;
            }
            for an in 2..=25 {
                if an == 12 || an == 14 || an == 19 || an == 20 || an == 22 || an == 23 || an == 24
                {
                    // The TupletActual type does not support these divisors
                    continue;
                }
                if (base_duration * nn) == numeric_duration * an {
                    if an != nn {
                        tuplet_representation = Some(TimeModification {
                            actual_notes: TupletActual::try_from(an).unwrap_or_else(|_e| {
                                panic!("Couldn't create TupletActual from u32 value {an}")
                            }),
                            normal_notes: TupletNormal::try_from(nn).unwrap_or_else(|_e| {
                                panic!("Couldn't create TupletNormal from u32 value {nn}")
                            }),
                        });
                    }
                    break;
                }
            }
            if tuplet_representation.is_some() {
                break;
            }
        }

        let note_type = if exponent < note_types.len() {
            note_types[exponent]
        } else {
            // exponent >= note_types.len() case
            *note_types.last().unwrap()
        };
        if let Some(val) = tuplet_representation {
            println!("TBC. Qtr{quarter_division} Duration {numeric_duration} NoteType: {:?} TimeMod: {:?}", note_type, val);
        }
        // else {
        //     println!("TBC. Qtr{quarter_division} Duration {numeric_duration} NoteType: {:?} isDotted: {}", note_type, is_dotted);
        // }

        Some((note_type, false, tuplet_representation))
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
        if !(Self::MIN_NOTE_VALUE..=Self::MAX_NOTE_VALUE).contains(&numeric_note) {
            Err(Error::OutofBounds)
        } else {
            Ok(NoteRestValue::Pitch(numeric_note as u8))
        }
    }

    pub fn decode_composite_note(self) -> Option<(Note, Alter, Octave)> {
        if let Some(octave) = self.get_octave() {
            match self {
                NoteRestValue::Rest => None,
                NoteRestValue::Pitch(v) => {
                    let oct_u8 = octave as u8;
                    let note_numeric = (v as i32) - (((oct_u8 as i32) - 4) * 12);
                    let note: Option<Note> = FromPrimitive::from_u8(note_numeric as u8);
                    match note {
                        None => None,
                        Some(n) => {
                            let (somenote, somealter) = n.get_note_alter_tuple();
                            Some((somenote, somealter, octave))
                        }
                    }
                }
            }
        } else {
            None
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
    fn test_from_numeric_duration() {
        let result = NoteData::from_numeric_duration(36, 24);
        assert_eq!(result, Some((NoteType::Crochet, true, None)));

        let result = NoteData::from_numeric_duration(1440, 480);
        assert_eq!(result, Some((NoteType::Minim, true, None)));

        let result = NoteData::from_numeric_duration(1920, 480);
        assert_eq!(result, Some((NoteType::SemiBreve, false, None)));

        let result = NoteData::from_numeric_duration(720, 480);
        assert_eq!(result, Some((NoteType::Crochet, true, None)));

        let result = NoteData::from_numeric_duration(96, 336);
        assert_eq!(
            result,
            Some((
                NoteType::Quaver,
                false,
                Some(TimeModification {
                    actual_notes: 7,
                    normal_notes: 4
                })
            ))
        );

        let result = NoteData::from_numeric_duration(112, 336);
        assert_eq!(
            result,
            Some((
                NoteType::Quaver,
                false,
                Some(TimeModification {
                    actual_notes: 3,
                    normal_notes: 2
                })
            ))
        );
    }

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
