use crate::error::{Error, Result};
use fraction::Fraction;
use log::error;
use mulib::pitch::{AccidentalSpelling, Alter, PitchOctave};
use muxml::muxml_types::{
    ChordElement, DotElement, DynamicsValue, GraceElement, NotationsElement, NoteElement,
    PitchElement, PitchRest, TimeModificationElement,
};
use num_derive::FromPrimitive;
use std::convert::From;
use std::str::FromStr;
use strum::{EnumCount, EnumIter};

#[derive(Eq, PartialEq, Default, Debug, Copy, Clone)]
pub struct TimeModification {
    actual_notes: TupletActual,
    normal_notes: TupletNormal,
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
    Staccato,
    Staccatissimo,
    Tenuto,
    DetachedLegato,
    Stress,
}

impl ToString for Articulation {
    fn to_string(&self) -> String {
        match self {
            Articulation::None => "".to_string(),
            Articulation::Accent => "accent".to_string(),
            Articulation::StrongAccent => "strong-accent".to_string(),
            Articulation::Staccato => "staccato".to_string(),
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
            "staccato" => Ok(Articulation::Staccato),
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

impl From<PhraseDynamics> for Option<DynamicsValue> {
    fn from(dynamics: PhraseDynamics) -> Option<DynamicsValue> {
        match dynamics {
            PhraseDynamics::None => None,
            PhraseDynamics::Pianississimo => Some(DynamicsValue::Ppp),
            PhraseDynamics::Pianissimo => Some(DynamicsValue::Pp),
            PhraseDynamics::Piano => Some(DynamicsValue::P),
            PhraseDynamics::Forte => Some(DynamicsValue::F),
            PhraseDynamics::Fortissimo => Some(DynamicsValue::Ff),
            PhraseDynamics::Fortississimo => Some(DynamicsValue::Fff),
            PhraseDynamics::MezzoPiano => Some(DynamicsValue::Mp),
            PhraseDynamics::MezzoForte => Some(DynamicsValue::Mf),
            _ => Some(DynamicsValue::P),
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
pub enum RhythmType {
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

impl FromStr for RhythmType {
    type Err = Error;
    fn from_str(input: &str) -> Result<RhythmType> {
        match input {
            "breve" => Ok(RhythmType::SemiBreve),
            "whole" => Ok(RhythmType::SemiBreve),
            "half" => Ok(RhythmType::Minim),
            "quarter" => Ok(RhythmType::Crochet),
            "eighth" => Ok(RhythmType::Quaver),
            "16th" => Ok(RhythmType::SemiQuaver),
            "32nd" => Ok(RhythmType::DemiSemiQuaver),
            "64th" => Ok(RhythmType::HemiDemiSemiQuaver),
            "128th" => Ok(RhythmType::SemiHemiDemiSemiQuaver),
            s => {
                error!("Unhandled Note type {}", s);
                Err(Error::Parse)
            }
        }
    }
}

impl RhythmType {
    pub fn get_type_string(self) -> String {
        match self {
            RhythmType::SemiHemiDemiSemiQuaver => String::from("128th"),
            RhythmType::HemiDemiSemiQuaver => String::from("64th"),
            RhythmType::DemiSemiQuaver => String::from("32nd"),
            RhythmType::SemiQuaver => String::from("16th"),
            RhythmType::Quaver => String::from("eighth"),
            RhythmType::Crochet => String::from("quarter"),
            RhythmType::Minim => String::from("half"),
            RhythmType::SemiBreve => String::from("whole"),
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
    pub note_rest: NumericPitchRest,
    pub phrase_dynamics: PhraseDynamics,
    pub note_type: RhythmType,
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

pub type IsDotted = bool;

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

    pub fn new_default_rest(note_type: RhythmType, dotted: IsDotted, voice: Voice) -> NoteData {
        NoteData {
            note_rest: NumericPitchRest::Rest,
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
            RhythmType::SemiBreve => Self::SEMIBREVE_DENOMINATOR,
            RhythmType::Minim => Self::MINIM_DENOMINATOR,
            RhythmType::Crochet => Self::CROCHET_DENOMINATOR,
            RhythmType::Quaver => Self::QUAVER_DENOMINATOR,
            RhythmType::SemiQuaver => Self::SEMI_QUAVER_DENOMINATOR,
            RhythmType::DemiSemiQuaver => Self::DEMI_SEMI_QUAVER_DENOMINATOR,
            RhythmType::HemiDemiSemiQuaver => Self::HEMI_DEMI_SEMI_QUAVER_DENOMINATOR,
            RhythmType::SemiHemiDemiSemiQuaver => Self::SEMI_HEMI_DEMI_SEMI_QUAVER_DENOMINATOR,
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
            RhythmType::SemiBreve => Self::MIDI_TICKS_SEMIBREVE * numerator / denominator,
            RhythmType::Minim => Self::MIDI_TICKS_MINIM * numerator / denominator,
            RhythmType::Crochet => Self::MIDI_TICKS_CROCHET * numerator / denominator,
            RhythmType::Quaver => Self::MIDI_TICKS_QUAVER * numerator / denominator,
            RhythmType::SemiQuaver => Self::MIDI_TICKS_SEMI_QUAVER * numerator / denominator,
            RhythmType::DemiSemiQuaver => {
                Self::MIDI_TICKS_DEMI_SEMI_QUAVER * numerator / denominator
            }
            RhythmType::HemiDemiSemiQuaver => {
                Self::MIDI_TICKS_HEMI_DEMI_SEMI_QUAVER * numerator / denominator
            }

            RhythmType::SemiHemiDemiSemiQuaver => {
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
        // chords should not contribute to the measure tally, but they must always
        // replicate the duration of their previous element
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
            RhythmType::SemiHemiDemiSemiQuaver => (divisions * numerator / 32) / denominator,
            RhythmType::HemiDemiSemiQuaver => (divisions * numerator / 16) / denominator,
            RhythmType::DemiSemiQuaver => (divisions * numerator / 8) / denominator,
            RhythmType::SemiQuaver => (divisions * numerator / 4) / denominator,
            RhythmType::Quaver => (divisions * numerator / 2) / denominator,
            RhythmType::Crochet => (divisions * numerator) / denominator,
            RhythmType::Minim => (divisions * 2 * numerator) / denominator,
            RhythmType::SemiBreve => {
                // The duration of a semi breve rest can differ based on time signature.
                // For example, in 4/4, it would be 4 crochets, but in 3/4, only 3 crochets
                if self.note_rest == NumericPitchRest::Rest {
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

    /// Converts a numeric duration to its corresponding musical `NoteType` and `IsDotted` representation.
    ///
    /// # Arguments
    ///
    /// * `numeric_duration` - The numeric value representing the duration of the note.
    /// * `quarter_division` - The number of divisions that represent a quarter note (crochet).
    ///
    /// # Returns
    ///
    /// Returns an `Option` containing a tuple of `NoteType` and `IsDotted` if the `numeric_duration`
    /// matches a standard musical note duration. Returns `None` if the `numeric_duration` doesn't fit
    /// standard note values.
    ///
    /// # Examples
    ///
    /// ```
    /// # use muxml::ir::notation::{NoteType, IsDotted, from_numeric_duration};
    /// let divisions = 480;
    /// let numeric = 720;
    ///
    /// assert_eq!(
    ///     from_numeric_duration(numeric, divisions),
    ///     Some((NoteType::QuarterNote, IsDotted::Dotted))
    /// );
    pub fn from_numeric_duration(
        numeric_duration: u32,
        quarter_division: u32,
    ) -> Option<(RhythmType, IsDotted, Option<TimeModification>)> {
        let note_types = [
            RhythmType::SemiBreve,
            RhythmType::Minim,
            RhythmType::Crochet,
            RhythmType::Quaver,
            RhythmType::SemiQuaver,
            RhythmType::DemiSemiQuaver,
            RhythmType::HemiDemiSemiQuaver,
            RhythmType::SemiHemiDemiSemiQuaver,
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

// The pitches in the binary format are the equivalent MIDI pitch numbers minus an offset of 11. MIDI Note 108 corresponds to 97 in this format. Note 12 -> 1
// The PitchOctave type from music lib uses the MIDI note number values
#[derive(Eq, PartialEq, Default, Clone, Copy, Debug)]
#[repr(u8)]
pub enum NumericPitchRest {
    #[default]
    Rest = 0,
    Pitch(u8),
}

impl NumericPitchRest {
    const MAX_NOTE_VALUE: i8 = 97;
    const MIN_NOTE_VALUE: i8 = 1;
    const REST_VALUE: u8 = 0;
    const MIDI_NOTE_OFFSET: i8 = 11;

    // fn get_octave(self) -> Option<Octave> {
    //     match self {
    //         NumericPitchRest::Rest => None,
    //         NumericPitchRest::Pitch(v) => {
    //             let num = (v - 1) / 12;
    //             FromPrimitive::from_u8(num)
    //         }
    //     }
    // }

    pub fn new_from_numeric(note_val: u8) -> Self {
        if note_val == 0 {
            NumericPitchRest::Rest
        } else {
            NumericPitchRest::Pitch(note_val)
        }
    }
    /// Encodes note data into numerical form for embedding. Supported note range is C0 to C8
    ///
    /// # Arguments
    ///
    /// * `pitch_octave`  -  Contains diatonic step, note accidental alterations, and octave
    pub fn from_pitch_octave(pitch_octave: PitchOctave) -> Result<NumericPitchRest> {
        let midi_numeric = i8::from(pitch_octave.pitch.step);
        let mut numeric_note = midi_numeric - Self::MIDI_NOTE_OFFSET;
        let numeric_alter = i8::from(pitch_octave.pitch.alter);
        let numeric_octave = pitch_octave.octave as i8; // MIDI C3 corresponds to C4 in MusicXML

        numeric_note += numeric_alter;
        numeric_note += (numeric_octave - 4) * 12;
        //println!("midi_numeric: {midi_numeric} numeric_octave: {numeric_octave}, numeric: {numeric_note}");
        if !(Self::MIN_NOTE_VALUE..=Self::MAX_NOTE_VALUE).contains(&numeric_note) {
            Err(Error::OutofBounds)
        } else {
            Ok(NumericPitchRest::Pitch(numeric_note as u8))
        }
    }

    pub fn get_pitch_octave(self) -> Option<PitchOctave> {
        match self {
            NumericPitchRest::Rest => None,
            NumericPitchRest::Pitch(v) => {
                let midi_note_numeric = (v as i8) + Self::MIDI_NOTE_OFFSET;
                Some(
                    PitchOctave::new_from_semitone(midi_note_numeric, AccidentalSpelling::Sharp)
                        .ok()?,
                )
            }
        }
    }

    pub fn get_numeric_value(self) -> u8 {
        match self {
            NumericPitchRest::Rest => NumericPitchRest::REST_VALUE,
            NumericPitchRest::Pitch(v) => v,
        }
    }
    pub fn get_midi_numeric_pitch_value(self) -> Option<u8> {
        match self {
            NumericPitchRest::Rest => None,
            NumericPitchRest::Pitch(v) => Some(v + 11),
        }
    }
}

impl From<NumericPitchRest> for PitchRest {
    fn from(note_data: NumericPitchRest) -> PitchRest {
        if note_data.get_numeric_value() == 0 {
            PitchRest::Rest
        } else if let Some(pabs) = note_data.get_pitch_octave() {
            // TODO: Make this logic for processing alter string more terse
            if pabs.pitch.alter == Alter::None {
                return PitchRest::Pitch(PitchElement {
                    step: pabs.pitch.step.to_string(),
                    octave: pabs.octave as i8 + 1,
                    alter: None,
                });
            } else {
                return PitchRest::Pitch(PitchElement {
                    step: pabs.pitch.step.to_string(),
                    octave: pabs.octave as i8 + 1,
                    alter: Some(pabs.pitch.alter.to_num_string()),
                });
            }
        } else {
            panic!("Decode composite note failed");
        }
    }
}

pub fn get_staff(voice: Voice, num_voices: usize) -> String {
    if num_voices < 3 {
        if voice == Voice::One {
            1.to_string()
        } else {
            2.to_string()
        }
    } else if voice == Voice::One || voice == Voice::Two {
        1.to_string()
    } else {
        2.to_string()
    }
}

pub struct NoteElementWrapper {
    note_element: NoteElement,
}

impl NoteElementWrapper {
    pub fn inner(&self) -> &NoteElement {
        &self.note_element
    }
    pub fn create_wrap(
        note: NoteData,
        divisions: u32,
        beats: Beats,
        beat_type: BeatType,
        t_modification: Option<TimeModificationElement>,
        notations: Option<NotationsElement>,
        num_voices: usize,
    ) -> Self {
        let note_element = NoteElement {
            chord: if note.chord.eq(&Chord::Chord) {
                Some(ChordElement {})
            } else {
                None
            },
            grace: if note.special_note != SpecialNote::None {
                Some(GraceElement {
                    slash: note.special_note.to_string(),
                })
            } else {
                None
            },
            pitch_or_rest: PitchRest::from(note.note_rest),
            duration: if note.special_note == SpecialNote::None {
                Some(note.get_duration_string(
                    divisions,
                    u32::from(beats),
                    u32::from(beat_type),
                    t_modification.as_ref().map(TimeModification::from),
                ))
            } else {
                None
            },
            beam: None,
            stem: None,
            dot: if note.dotted {
                Some(DotElement {})
            } else {
                None
            },
            voice: (note.voice as u8 + 1).to_string(),
            r#type: note.note_type.get_type_string(),
            time_modification: t_modification,
            staff: get_staff(note.voice, num_voices),
            notations,
        };
        Self { note_element }
    }
}

// #[cfg(test)]
// mod tests {
//     use super::{
//         Alter, NoteData, NumericPitchRest, Octave, RhythmType, Tempo, TimeModification,
//     };
//     use super::{TupletActual, TupletNormal};
//     use crate::error::Error;
//     #[test]
//     fn test_from_numeric_duration() {
//         let result = NoteData::from_numeric_duration(36, 24);
//         assert_eq!(result, Some((RhythmType::Crochet, true, None)));

//         let result = NoteData::from_numeric_duration(1440, 480);
//         assert_eq!(result, Some((RhythmType::Minim, true, None)));

//         let result = NoteData::from_numeric_duration(1920, 480);
//         assert_eq!(result, Some((RhythmType::SemiBreve, false, None)));

//         let result = NoteData::from_numeric_duration(720, 480);
//         assert_eq!(result, Some((RhythmType::Crochet, true, None)));

//         let result = NoteData::from_numeric_duration(96, 336);
//         assert_eq!(
//             result,
//             Some((
//                 RhythmType::Quaver,
//                 false,
//                 Some(TimeModification {
//                     actual_notes: TupletActual::Seven,
//                     normal_notes: TupletNormal::Four
//                 })
//             ))
//         );

//         let result = NoteData::from_numeric_duration(112, 336);
//         assert_eq!(
//             result,
//             Some((
//                 RhythmType::Quaver,
//                 false,
//                 Some(TimeModification {
//                     actual_notes: TupletActual::Three,
//                     normal_notes: TupletNormal::Two
//                 })
//             ))
//         );
//     }

//     #[test]
//     fn test_tempo_into() {
//         let value: Tempo = 30.into();
//         assert_eq!(value.0, 5);
//     }

//     // #[test]
//     // fn test_encode_note() {
//     //     let mut note = Step::C;
//     //     let mut alter = Alter::None;
//     //     let mut octave = Octave::Octave0;

//     //     assert_eq!(
//     //         NumericPitchRest::derive_numeric_note(note, alter, octave),
//     //         Ok(NumericPitchRest::Pitch(1))
//     //     );

//     //     alter = Alter::Sharp;
//     //     assert_eq!(
//     //         NumericPitchRest::derive_numeric_note(note, alter, octave),
//     //         Ok(NumericPitchRest::Pitch(2))
//     //     );

//     //     alter = Alter::None;
//     //     octave = Octave::Octave8;
//     //     assert_eq!(
//     //         NumericPitchRest::derive_numeric_note(note, alter, octave),
//     //         Ok(NumericPitchRest::Pitch(97))
//     //     );

//     //     alter = Alter::Flat;
//     //     assert_eq!(
//     //         NumericPitchRest::derive_numeric_note(note, alter, octave),
//     //         Ok(NumericPitchRest::Pitch(96))
//     //     );

//     //     note = Step::D;
//     //     alter = Alter::None;
//     //     octave = Octave::Octave8;
//     //     assert_eq!(
//     //         NumericPitchRest::derive_numeric_note(note, alter, octave),
//     //         Err(Error::OutofBounds)
//     //     );
//     //     note = Step::C;
//     //     alter = Alter::Sharp;
//     //     octave = Octave::Octave8;
//     //     assert_eq!(
//     //         NumericPitchRest::derive_numeric_note(note, alter, octave),
//     //         Err(Error::OutofBounds)
//     //     );
//     //     note = Step::C;
//     //     alter = Alter::Flat;
//     //     octave = Octave::Octave0;
//     //     assert_eq!(
//     //         NumericPitchRest::derive_numeric_note(note, alter, octave),
//     //         Err(Error::OutofBounds)
//     //     );
//     // }
// }
