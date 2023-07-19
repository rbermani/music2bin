use crate::notation::Dynamics;
use serde::Serialize;

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct WorkElement {
    pub work_title: String,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct IdentificationElement {
    pub creator: CreatorElement,
    pub encoding: EncodingElement,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct ScorePart {
    #[serde(rename(serialize = "@id"))]
    pub id: String,
    pub part_name: String,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct PartListElement {
    pub score_part: ScorePart,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct KeyElement {
    pub fifths: String,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct TimeElement {
    pub beats: String,
    pub beat_type: String,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct ClefElement {
    #[serde(rename(serialize = "@number"))]
    pub number: String,
    pub sign: String,
}

#[derive(Debug, Serialize, PartialEq)]
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
#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct WordsElement {
    #[serde(rename = "$value")]
    pub value: String,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum DirectionType {
    Dynamics(MXmlDynamics),
    Segno,
    Coda,
    Words(WordsElement),
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct DirectionTypeElement {
    pub direction_type: DirectionType,
}

#[derive(Debug, Serialize, PartialEq)]
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

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct DirectionElement {
    pub direction_type: DirectionTypeElement,
    pub staff: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sound: Option<SoundElement>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct AttributesElement {
    pub divisions: String,
    pub key: KeyElement,
    pub time: TimeElement,
    pub staves: String,
    pub clef: Vec<ClefElement>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct PitchElement {
    pub step: String,
    pub octave: i8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alter: Option<f32>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Notations {
    Tied,
    Slur,
    Tuplet,
    Articulations,
    Dynamics,
    Fermata,
    Arpeggiate,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum PitchRest {
    Pitch(PitchElement),
    Rest,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct NoteElement {
    pub pitch: PitchRest,
    pub duration: i8,
    pub r#type: String,
    pub staff: i8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notations: Option<Vec<Notations>>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct Measure {
    #[serde(rename(serialize = "@number"))]
    pub number: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<AttributesElement>,
    pub direction: Vec<DirectionElement>,
    pub note: Vec<NoteElement>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct Part {
    #[serde(rename(serialize = "@id"))]
    pub id: String,
    pub measure: Vec<Measure>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename = "score-part-wise")]
#[serde(rename_all = "kebab-case")]
pub struct ScorePartWise {
    #[serde(rename(serialize = "@version"))]
    pub version: String,
    pub work: WorkElement,
    pub identification: IdentificationElement,
    pub part_list: Vec<PartListElement>,
    pub part: Vec<Part>,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct SupportsElement {
    #[serde(rename(serialize = "@element"))]
    pub element: String,
    #[serde(rename(serialize = "@type"))]
    pub r#type: String,
}

#[derive(Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct EncodingElement {
    pub software: String,
    pub encoding_date: String,
    pub supports: Vec<SupportsElement>,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct CreatorElement {
    #[serde(rename(serialize = "@type"))]
    pub r#type: String,
    #[serde(rename = "$value")]
    pub value: String,
}
