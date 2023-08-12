use failure::Error;
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use std::convert::From;
use std::num::ParseIntError;
use std::str::FromStr;

#[derive(Fail, PartialEq, Debug)]
pub enum NotationError {
    #[fail(display = "Unnamed Error")]
    Unit,
    #[fail(display = "Data Out of Bounds")]
    OutofBounds,
    #[fail(display = "Parsing Error")]
    Parse,
    #[fail(display = "Encoding Error")]
    EncodingError,
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
            Self::NoteA => (Self::NoteG, Alter::None),
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
    type Err = NotationError;
    fn from_str(input: &str) -> Result<KeySignature, NotationError> {
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
            _ => Err(NotationError::Unit),
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
#[derive(Copy, Clone)]
#[repr(i8)]
pub enum Alter {
    Flat = -1,
    None = 0,
    Sharp = 1,
}

impl FromStr for Alter {
    type Err = NotationError;
    fn from_str(input: &str) -> Result<Alter, NotationError> {
        match input {
            "-1" => Ok(Alter::Flat),
            "0" => Ok(Alter::None),
            "1" => Ok(Alter::Sharp),
            _ => Err(NotationError::Unit),
        }
    }
}
impl ToString for Alter {
    fn to_string(&self) -> String {
        match self {
            Alter::Flat => String::from("-1"),
            Alter::None => String::from("0"),
            Alter::Sharp => String::from("1"),
        }
    }
}
impl FromStr for Octave {
    type Err = NotationError;
    fn from_str(input: &str) -> Result<Octave, NotationError> {
        match input {
            "0" => Ok(Octave::Octave0),
            "1" => Ok(Octave::Octave1),
            "2" => Ok(Octave::Octave2),
            "3" => Ok(Octave::Octave3),
            "4" => Ok(Octave::Octave4),
            "5" => Ok(Octave::Octave5),
            "6" => Ok(Octave::Octave6),
            "7" => Ok(Octave::Octave7),
            _ => Err(NotationError::Unit),
        }
    }
}

impl FromStr for Note {
    type Err = NotationError;
    fn from_str(input: &str) -> Result<Note, NotationError> {
        match input {
            "C" => Ok(Note::NoteC),
            "D" => Ok(Note::NoteD),
            "E" => Ok(Note::NoteE),
            "F" => Ok(Note::NoteF),
            "G" => Ok(Note::NoteG),
            "A" => Ok(Note::NoteA),
            "B" => Ok(Note::NoteB),
            _ => Err(NotationError::Unit),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, FromPrimitive, Default, Debug)]
#[repr(u8)]
pub enum Dynamics {
    Pianississimo = 0,
    Pianissimo,
    #[default]
    Piano,
    MezzoPiano,
    MezzoForte,
    Forte,
    Fortissimo,
    Fortississimo,
}

impl FromStr for Dynamics {
    type Err = NotationError;
    fn from_str(input: &str) -> Result<Dynamics, NotationError> {
        match input {
            "ppp" => Ok(Dynamics::Pianississimo),
            "pp" => Ok(Dynamics::Pianissimo),
            "p" => Ok(Dynamics::Piano),
            "mp" => Ok(Dynamics::MezzoPiano),
            "mf" => Ok(Dynamics::MezzoForte),
            "f" => Ok(Dynamics::Forte),
            "ff" => Ok(Dynamics::Fortissimo),
            "fff" => Ok(Dynamics::Fortississimo),
            _ => Err(NotationError::Parse),
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
    type Err = NotationError;
    fn from_str(input: &str) -> Result<NoteConnection, NotationError> {
        match input {
            "start" => Ok(NoteConnection::StartTie),
            "stop" => Ok(NoteConnection::EndTie),
            _ => Err(NotationError::Parse),
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, FromPrimitive, Default, Debug)]
#[repr(u8)]
pub enum MeasureStartEnd {
    #[default]
    MeasureStart = 0,
    MeasureEnd,
}

impl From<MeasureStartEnd> for bool {
    fn from(f: MeasureStartEnd) -> bool {
        match f {
            MeasureStartEnd::MeasureStart => false,
            MeasureStartEnd::MeasureEnd => true,
        }
    }
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

type TupletActual = u8;
type TupletNormal = u8;
type TupletDotted = bool;

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
}
#[derive(Eq, PartialEq, Copy, Clone, FromPrimitive, Default, Debug)]
#[repr(u8)]
pub enum Repeats {
    #[default]
    NoRepeat = 0,
    Repeat,
}

impl From<Repeats> for bool {
    fn from(f: Repeats) -> bool {
        match f {
            Repeats::NoRepeat => false,
            Repeats::Repeat => true,
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
    fn from_str(input: &str) -> Result<Tempo, Error> {
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

// impl From<f32> for Tempo {
//     fn from(real_tempo: f32) -> Self {
//         let assign_tempo;
//         if real_tempo > Self::MAX_SUPPORTED_REAL_TEMPO as f32 {
//             assign_tempo = Self::MAX_SUPPORTED_REAL_TEMPO as f32;
//         } else if real_tempo < Self::MIN_SUPPORTED_REAL_TEMPO as f32 {
//             assign_tempo = Self::MIN_SUPPORTED_REAL_TEMPO as f32;
//         } else {
//             assign_tempo = real_tempo;
//         }
//         Tempo(((assign_tempo - 20.0) / 2.0) as u8)
//     }
// }

// impl Tempo {
//     pub fn as_float(&self) -> f32 {
//         match self {
//             Tempo::Larghissimo => 24.0,
//             Tempo::Grave => 40.0,
//             Tempo::Lento => 45.0,
//             Tempo::Largo => 50.0,
//             Tempo::Adagio => 65.0,
//             Tempo::Adagietto => 69.0,
//             Tempo::Andante => 77.0,
//             Tempo::Moderato => 97.0,
//             Tempo::Allegretto => 120.0,
//             Tempo::Allegro => 150.0,
//             Tempo::Vivace => 176.0,
//             Tempo::Presto => 200.0,
//             Tempo::Prestissimo => 210.0,
//             Tempo::PiuPrestissimo => 220.0,
//         }
//     }
// }

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
    fn from_str(input: &str) -> Result<DescriptiveTempo, Error> {
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

#[derive(Eq, PartialEq, Copy, Clone, FromPrimitive, Default, Debug)]
#[repr(u8)]
pub enum TrebleBassClef {
    #[default]
    TrebleClef,
    BassClef,
}

impl From<TrebleBassClef> for bool {
    fn from(f: TrebleBassClef) -> bool {
        match f {
            TrebleBassClef::TrebleClef => false,
            TrebleBassClef::BassClef => true,
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, FromPrimitive, Default, Debug)]
#[repr(u8)]
pub enum Duration {
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

impl FromStr for Duration {
    type Err = NotationError;
    fn from_str(input: &str) -> Result<Duration, NotationError> {
        match input {
            "whole" => Ok(Duration::SemiBreve),
            "half" => Ok(Duration::Minim),
            "quarter" => Ok(Duration::Crochet),
            "eighth" => Ok(Duration::Quaver),
            "16th" => Ok(Duration::SemiQuaver),
            "32nd" => Ok(Duration::DemiSemiQuaver),
            "64th" => Ok(Duration::HemiDemiSemiQuaver),
            "128th" => Ok(Duration::SemiHemiDemiSemiQuaver),
            _ => Err(NotationError::Parse),
        }
    }
}

impl Duration {
    pub fn get_type_string(self) -> String {
        match self {
            Duration::SemiHemiDemiSemiQuaver => String::from("128th"),
            Duration::HemiDemiSemiQuaver => String::from("64th"),
            Duration::DemiSemiQuaver => String::from("32nd"),
            Duration::SemiQuaver => String::from("16th"),
            Duration::Quaver => String::from("eighth"),
            Duration::Crochet => String::from("quarter"),
            Duration::Minim => String::from("half"),
            Duration::SemiBreve => String::from("whole"),
        }
    }

    pub fn get_duration_string(self) -> String {
        String::from("4")
        // match self {
        //     Duration::SemiHemiDemiSemiQuaver => ,
        //     Duration::HemiDemiSemiQuaver => ,
        //     Duration::DemiSemiQuaver => ,
        //     Duration::SemiQuaver => ,
        //     Duration::Quaver => ,
        //     Duration::Crochet => ,
        //     Duration::Minim => ,
        //     Duration::SemiBreve => ,
        // }
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
    type Err = NotationError;
    fn from_str(input: &str) -> Result<Beats, NotationError> {
        match input {
            "2" => Ok(Beats::Two),
            "3" => Ok(Beats::Three),
            "4" => Ok(Beats::Four),
            "5" => Ok(Beats::Five),
            "6" => Ok(Beats::Six),
            "9" => Ok(Beats::Nine),
            "12" => Ok(Beats::Twelve),
            _ => Err(NotationError::Parse),
        }
    }
}

impl FromStr for BeatType {
    type Err = NotationError;
    fn from_str(input: &str) -> Result<BeatType, NotationError> {
        match input {
            "2" => Ok(BeatType::Two),
            "4" => Ok(BeatType::Four),
            "8" => Ok(BeatType::Eight),
            "16" => Ok(BeatType::Sixteen),
            _ => Err(NotationError::Parse),
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
    pub treble_dynamics: Dynamics,
    pub bass_dynamics: Dynamics,
    pub tempo: Tempo,
}

#[derive(Eq, PartialEq, Default, Clone, Copy, Debug)]
pub struct MeasureMetaData {
    pub start_end: MeasureStartEnd,
    pub repeat: Repeats,
    pub dal_segno: DalSegno,
}

#[derive(Eq, PartialEq, Default, Clone, Copy, Debug)]
pub struct NoteData {
    pub note_rest: NoteRestValue,
    pub phrase_dynamics: PhraseDynamics,
    pub rhythm_value: Duration,
    pub arpeggiate: Arpeggiate,
    pub special_note: SpecialNote,
    pub articulation: Articulation,
    pub trill: Trill,
    pub ties: NoteConnection,
    pub treble_bass: TrebleBassClef,
    pub stress: Stress,
    pub chord: Chord,
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
    const REST_VALUE: i8 = 0;

    fn get_octave(self) -> Option<Octave> {
        match self {
            NoteRestValue::Rest => None,
            NoteRestValue::Pitch(v) => {
                let num = (v / 12) - 1;
                FromPrimitive::from_u8(num)
            }
        }
    }

    fn get_note(self) -> Option<Note> {
        match self {
            NoteRestValue::Rest => None,
            NoteRestValue::Pitch(v) => {
                let num = (v / 12) - 1;
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
    pub fn derive_numeric_note(
        note: Note,
        alter: Alter,
        octave: Octave,
    ) -> Result<NoteRestValue, NotationError> {
        let mut numeric_note = note as i8;
        let numeric_alter = alter as i8;
        let numeric_octave = octave as i8;

        numeric_note += numeric_alter;
        numeric_note += (numeric_octave - 4) * 12;
        if numeric_note > Self::MAX_NOTE_VALUE {
            Err(NotationError::OutofBounds)
        } else if numeric_note < Self::MIN_NOTE_VALUE {
            Err(NotationError::OutofBounds)
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
                    let note_numeric =
                        (((oct_u8 as i32) - 4) * 12).abs() + (self.get_numeric_value() as i32);
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
            NoteRestValue::Rest => 0,
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
            Err(NotationError::OutofBounds)
        );
        note = Note::NoteC;
        alter = Alter::Sharp;
        octave = Octave::Octave8;
        assert_eq!(
            NoteRestValue::derive_numeric_note(note, alter, octave),
            Err(NotationError::OutofBounds)
        );
        note = Note::NoteC;
        alter = Alter::Flat;
        octave = Octave::Octave0;
        assert_eq!(
            NoteRestValue::derive_numeric_note(note, alter, octave),
            Err(NotationError::OutofBounds)
        );
    }
}
