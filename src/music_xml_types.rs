use crate::notation::{Dynamics, NoteData, NoteRestValue, TupletNumber};
use failure::Error;
use serde::Serialize;

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct WorkElement {
    pub work_title: String,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct IdentificationElement {
    pub creator: CreatorElement,
    pub encoding: EncodingElement,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct ScorePart {
    #[serde(rename(serialize = "@id"))]
    pub id: String,
    pub part_name: String,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct PartListElement {
    pub score_part: ScorePart,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct KeyElement {
    pub fifths: String,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct TimeElement {
    pub beats: String,
    pub beat_type: String,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct ClefElement {
    #[serde(rename(serialize = "@number"))]
    pub number: String,
    pub sign: String,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum MXmlDynamics {
    Ppp,
    Pp,
    P,
    F,
    Ff,
    Fff,
    Mp,
    Mf,
    Sf,
    Rf,
    N,
}

impl MXmlDynamics {
    pub fn from_dynamics(dynamics: Dynamics) -> MXmlDynamics {
        match dynamics {
            Dynamics::Pianississimo => MXmlDynamics::Ppp,
            Dynamics::Pianissimo => MXmlDynamics::Pp,
            Dynamics::Piano => MXmlDynamics::P,
            Dynamics::Forte => MXmlDynamics::F,
            Dynamics::Fortissimo => MXmlDynamics::Ff,
            Dynamics::Fortississimo => MXmlDynamics::Fff,
            Dynamics::MezzoPiano => MXmlDynamics::Mp,
            Dynamics::MezzoForte => MXmlDynamics::Mf,
        }
    }
}
#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct WordsElement {
    #[serde(rename = "$value")]
    pub value: String,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum DirectionType {
    Dynamics(MXmlDynamics),
    Segno,
    Coda,
    Words(WordsElement),
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct DirectionTypeElement {
    pub direction_type: DirectionType,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct SoundElement {
    #[serde(
        rename(serialize = "@dynamics"),
        skip_serializing_if = "Option::is_none"
    )]
    pub dynamics: Option<f32>,
    #[serde(rename(serialize = "@tempo"), skip_serializing_if = "Option::is_none")]
    pub tempo: Option<f32>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct DirectionElement {
    pub direction_type: DirectionTypeElement,
    pub staff: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sound: Option<SoundElement>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct AttributesElement {
    pub divisions: String,
    pub key: KeyElement,
    pub time: TimeElement,
    pub staves: String,
    pub clef: Vec<ClefElement>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct PitchElement {
    pub step: String,
    pub octave: i8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alter: Option<String>,
}
#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum TupletType {
    Start,
    Stop,
}

#[derive(Copy, Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
#[repr(u8)]
pub enum MXmlTupletNumber {
    TupletOne = 1,
    TupletTwo = 2,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct TupletElement {
    #[serde(rename(serialize = "@type"))]
    pub r#type: TupletType,
    #[serde(rename(serialize = "@number"))]
    pub number: MXmlTupletNumber,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Notations {
    Tied,
    Slur,
    Tuplet(TupletElement),
    Articulations,
    Dynamics,
    Fermata,
    Arpeggiate,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum PitchRest {
    Pitch(PitchElement),
    Rest,
}
impl From<NoteRestValue> for PitchRest {
    fn from(note_data: NoteRestValue) -> PitchRest {
        if note_data.get_numeric_value() == 0 {
            return PitchRest::Rest;
        } else {
            if let Some((step, alter, octave)) = note_data.decode_composite_note() {
                // TODO: Make this logic for processing alter string more terse
                let alter_string = alter.to_string();
                if alter_string.eq("0") {
                    return PitchRest::Pitch(PitchElement {
                        step: step.to_string(),
                        octave: octave as i8,
                        alter: None,
                    });
                } else {
                    return PitchRest::Pitch(PitchElement {
                        step: step.to_string(),
                        octave: octave as i8,
                        alter: Some(alter_string),
                    });
                }
            } else {
                return PitchRest::Rest;
            }
        }
    }
}

#[derive(Copy, Clone, Debug, Default, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct TimeModificationElement {
    pub actual_notes: u8,
    pub normal_notes: u8,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename = "chord")]
#[serde(rename_all = "kebab-case")]
pub struct ChordElement();

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct NoteElement {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chord: Option<ChordElement>,
    pub pitch: PitchRest,
    pub duration: String,
    pub r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_modification: Option<TimeModificationElement>,
    pub staff: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notations: Option<Vec<Notations>>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum MeasureDirectionNote {
    None,
    Direction(DirectionElement),
    Note(NoteElement),
}
#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct Measure {
    #[serde(rename(serialize = "@number"))]
    pub number: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<AttributesElement>,
    pub direction_note: Vec<MeasureDirectionNote>,
}

impl Default for Measure {
    fn default() -> Measure {
        Measure {
            number: "1".to_string(),
            attributes: None,
            direction_note: vec![],
        }
    }
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct Part {
    #[serde(rename(serialize = "@id"))]
    pub id: String,
    pub measure: Vec<Measure>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename = "score-partwise")]
#[serde(rename_all = "kebab-case")]
pub struct ScorePartWise {
    #[serde(rename(serialize = "@version"))]
    pub version: String,
    pub work: WorkElement,
    pub identification: IdentificationElement,
    pub part_list: Vec<PartListElement>,
    pub part: Vec<Part>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct SupportsElement {
    #[serde(rename(serialize = "@element"))]
    pub element: String,
    #[serde(rename(serialize = "@type"))]
    pub r#type: String,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct EncodingElement {
    pub software: String,
    pub encoding_date: String,
    pub supports: Vec<SupportsElement>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
pub struct CreatorElement {
    #[serde(rename(serialize = "@type"))]
    pub r#type: String,
    #[serde(rename = "$value")]
    pub value: String,
}
