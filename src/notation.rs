use failure::Error;
use num_derive::FromPrimitive;
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

#[derive(Copy, Clone)]
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

#[derive(Copy, Clone)]
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

#[derive(Copy, Clone, Eq, FromPrimitive, PartialEq, Default, Debug)]
#[repr(u8)]
pub enum Tempo {
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
    PiuPrestissimo,
}
impl Tempo {
    pub fn as_float(&self) -> f32 {
        match self {
            Tempo::Larghissimo => 24.0,
            Tempo::Grave => 40.0,
            Tempo::Lento => 45.0,
            Tempo::Largo => 50.0,
            Tempo::Adagio => 65.0,
            Tempo::Adagietto => 69.0,
            Tempo::Andante => 77.0,
            Tempo::Moderato => 97.0,
            Tempo::Allegretto => 120.0,
            Tempo::Allegro => 150.0,
            Tempo::Vivace => 176.0,
            Tempo::Presto => 200.0,
            Tempo::Prestissimo => 210.0,
            Tempo::PiuPrestissimo => 220.0,
        }
    }
}
impl ToString for Tempo {
    fn to_string(&self) -> String {
        match self {
            Tempo::Larghissimo => String::from("Larghissimo"),
            Tempo::Grave => String::from("Grave"),
            Tempo::Lento => String::from("Lento"),
            Tempo::Largo => String::from("Largo"),
            Tempo::Adagio => String::from("Adagio"),
            Tempo::Adagietto => String::from("Adagietto"),
            Tempo::Andante => String::from("Andante"),
            Tempo::Moderato => String::from("Moderato"),
            Tempo::Allegretto => String::from("Allegretto"),
            Tempo::Allegro => String::from("Allegro"),
            Tempo::Vivace => String::from("Vivace"),
            Tempo::Presto => String::from("Presto"),
            Tempo::Prestissimo => String::from("Prestissimo"),
            Tempo::PiuPrestissimo => String::from("PiuPrestissimo"),
        }
    }
}

impl FromStr for Tempo {
    type Err = Error;
    fn from_str(input: &str) -> Result<Tempo, Error> {
        let val = u32::from_str(input)?;
        if val <= 24 {
            Ok(Tempo::Larghissimo)
        } else if val <= 40 {
            Ok(Tempo::Grave)
        } else if val <= 45 {
            Ok(Tempo::Lento)
        } else if val <= 50 {
            Ok(Tempo::Largo)
        } else if val <= 65 {
            Ok(Tempo::Adagio)
        } else if val <= 69 {
            Ok(Tempo::Adagietto)
        } else if val <= 77 {
            Ok(Tempo::Andante)
        } else if val <= 97 {
            Ok(Tempo::Moderato)
        } else if val <= 120 {
            Ok(Tempo::Allegretto)
        } else if val <= 150 {
            Ok(Tempo::Allegro)
        } else if val <= 176 {
            Ok(Tempo::Vivace)
        } else if val <= 200 {
            Ok(Tempo::Presto)
        } else if val <= 210 {
            Ok(Tempo::Prestissimo)
        } else {
            Ok(Tempo::PiuPrestissimo)
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
pub enum RightHandLeftHand {
    #[default]
    RightHand,
    LeftHand,
}

impl From<RightHandLeftHand> for bool {
    fn from(f: RightHandLeftHand) -> bool {
        match f {
            RightHandLeftHand::RightHand => false,
            RightHandLeftHand::LeftHand => true,
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
#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum MusicElement {
    MeasureInit(MeasureInitializer),
    MeasureMeta(MeasureMetaData),
    NoteRest(NoteData),
}

#[derive(Eq, PartialEq, Copy, Clone, Default, Debug)]
pub struct MeasureInitializer {
    pub tempo: Tempo,
    pub beats: Beats,
    pub beat_type: BeatType,
    pub key_sig: KeySignature,
    pub treble_dynamics: Dynamics,
    pub bass_dynamics: Dynamics,
}

#[derive(Eq, PartialEq, Default, Clone, Copy, Debug)]
pub struct MeasureMetaData {
    pub start_end: MeasureStartEnd,
    pub repeat: Repeats,
    pub dal_segno: DalSegno,
}

type NoteRestValue = u8;

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
    pub rh_lh: RightHandLeftHand,
    pub stress: Stress,
}

impl NoteData {
    const MAX_NOTE_VALUE: i8 = 97;
    const MIN_NOTE_VALUE: i8 = 1;
    /// Encodes note data into numerical form for embedding. Supported note range is C0 to C8
    ///
    /// # Arguments
    ///
    /// * `note`  -  An enum indicating the step of the 12 tone equal temperament diatonic scale
    /// * `alter` -  An enum indicating if note should be unchanged, flatted, or sharped
    /// * `octave` - An enum indicating the octave from 0 to 8
    ///
    pub fn encode_numeric_note(
        note: Note,
        alter: Alter,
        octave: Octave,
    ) -> Result<NoteRestValue, NotationError> {
        let mut numeric_note = note as i8;
        let numeric_alter = alter as i8;
        let numeric_octave = octave as i8;

        numeric_note += numeric_alter;
        numeric_note += (numeric_octave - 4) * 12;
        if numeric_note > Self::MAX_NOTE_VALUE || numeric_note < Self::MIN_NOTE_VALUE {
            Err(NotationError::OutofBounds)
        } else {
            Ok(numeric_note as u8)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_note() {
        let mut note = Note::NoteC;
        let mut alter = Alter::None;
        let mut octave = Octave::Octave0;

        assert_eq!(NoteData::encode_numeric_note(note, alter, octave), Ok(1));

        alter = Alter::Sharp;
        assert_eq!(NoteData::encode_numeric_note(note, alter, octave), Ok(2));

        alter = Alter::None;
        octave = Octave::Octave8;
        assert_eq!(NoteData::encode_numeric_note(note, alter, octave), Ok(97));

        alter = Alter::Flat;
        assert_eq!(NoteData::encode_numeric_note(note, alter, octave), Ok(96));

        note = Note::NoteD;
        alter = Alter::None;
        octave = Octave::Octave8;
        assert_eq!(
            NoteData::encode_numeric_note(note, alter, octave),
            Err(NotationError::OutofBounds)
        );
        note = Note::NoteC;
        alter = Alter::Sharp;
        octave = Octave::Octave8;
        assert_eq!(
            NoteData::encode_numeric_note(note, alter, octave),
            Err(NotationError::OutofBounds)
        );
        note = Note::NoteC;
        alter = Alter::Flat;
        octave = Octave::Octave0;
        assert_eq!(
            NoteData::encode_numeric_note(note, alter, octave),
            Err(NotationError::OutofBounds)
        );
    }
}
