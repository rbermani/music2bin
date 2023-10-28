use crate::notation::*;

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
pub enum DynamicsValue {
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

impl DynamicsValue {
    pub fn from_dynamics(dynamics: PhraseDynamics) -> Option<DynamicsValue> {
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

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct DynamicsElement {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamics: Option<DynamicsValue>,
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
    Dynamics(DynamicsElement),
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub alter: Option<String>,
    pub octave: i8,
}

#[derive(Clone, Copy, Default, Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum TupletType {
    #[default]
    Start,
    Stop,
    None,
}

pub type SlurType = TupletType;
pub type TiedType = TupletType;

#[derive(Clone, Debug, Default, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct TupletElement {
    #[serde(rename(serialize = "@type"))]
    pub r#type: TupletType,
    #[serde(rename(serialize = "@number"))]
    pub number: String,
}

#[derive(Clone, Debug, Default, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct TiedElement {
    #[serde(rename(serialize = "@type"))]
    pub r#type: TiedType,
}

#[derive(Clone, Debug, Default, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct SlurElement {
    #[serde(rename(serialize = "@type"))]
    pub r#type: SlurType,
    #[serde(rename(serialize = "@number"))]
    pub number: String,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum Notations {
    Tied(TiedElement),
    Slur(SlurElement),
    Tuplet(TupletElement),
    Articulations,
    Dynamics,
    Fermata,
    Arpeggiate,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct NotationsElement {
    pub notations: Vec<Notations>,
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
                if alter == Alter::None {
                    return PitchRest::Pitch(PitchElement {
                        step: step.to_string(),
                        octave: octave as i8,
                        alter: None,
                    });
                } else {
                    return PitchRest::Pitch(PitchElement {
                        step: step.to_string(),
                        octave: octave as i8,
                        alter: Some(alter.to_string()),
                    });
                }
            } else {
                panic!("Decode composite note failed");
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
#[serde(rename_all = "kebab-case")]
pub struct ChordElement {}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct DotElement {}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct GraceElement {
    #[serde(rename(serialize = "@slash"))]
    pub slash: String,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct NoteElement {
    #[serde(skip_serializing_if = "Option::is_none")]
    chord: Option<ChordElement>,
    #[serde(skip_serializing_if = "Option::is_none")]
    grace: Option<GraceElement>,
    pitch: PitchRest,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration: Option<String>,
    voice: String,
    r#type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    dot: Option<DotElement>,
    #[serde(skip_serializing_if = "Option::is_none")]
    time_modification: Option<TimeModificationElement>,
    staff: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    notations: Option<NotationsElement>,
}

impl NoteElement {
    pub fn new(
        note: NoteData,
        divisions: u32,
        beats: Beats,
        beat_type: BeatType,
        t_modification: Option<TimeModificationElement>,
        notations: Option<NotationsElement>,
        num_voices: usize,
    ) -> Self {
        Self {
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
            pitch: PitchRest::from(note.note_rest),
            duration: if note.special_note == SpecialNote::None {
                Some(note.get_duration_string(
                    divisions,
                    u32::from(beats),
                    u32::from(beat_type),
                    t_modification.map(|v| TimeModification::from(v)),
                ))
            } else {
                None
            },
            dot: if note.dotted {
                Some(DotElement {})
            } else {
                None
            },
            voice: (note.voice as u8 + 1).to_string(),
            r#type: note.note_type.get_type_string(),
            time_modification: t_modification,
            staff: Self::get_staff(note.voice, num_voices),
            notations: notations,
        }
    }

    pub fn insert_stop_tuple(&mut self, tuplet_number: String) {
        if self.notations.is_some() {
            let ne = self.notations.as_mut().unwrap();
            ne.notations.push(Notations::Tuplet(TupletElement {
                r#type: TupletType::Stop,
                number: tuplet_number,
            }));
        } else {
            self.notations = Some(NotationsElement {
                notations: vec![Notations::Tuplet(TupletElement {
                    r#type: TupletType::Stop,
                    number: tuplet_number,
                })],
            });
        }
    }

    pub fn clear_time_mods(&mut self) {
        self.time_modification = None;
    }

    pub fn get_staff(voice: Voice, num_voices: usize) -> String {
        if num_voices < 3 {
            if voice == Voice::One {
                return 1.to_string();
            } else {
                return 2.to_string();
            }
        } else {
            if voice == Voice::One || voice == Voice::Two {
                return 1.to_string();
            } else {
                return 2.to_string();
            }
        }
    }
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct BackupElement {
    pub duration: String,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum MeasureDirectionNote {
    None,
    Direction(DirectionElement),
    Barline(BarlineElement),
    Note(NoteElement),
    Backup(BackupElement),
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct EndingElement {
    #[serde(rename(serialize = "@number"), skip_serializing_if = "Option::is_none")]
    pub number: Option<String>,
    #[serde(rename(serialize = "@type"), skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(rename = "$value", skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct RepeatElement {
    #[serde(
        rename(serialize = "@direction"),
        skip_serializing_if = "Option::is_none"
    )]
    pub direction: Option<String>,
}

#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub struct BarlineElement {
    #[serde(
        rename(serialize = "@location"),
        skip_serializing_if = "Option::is_none"
    )]
    pub location: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ending: Option<EndingElement>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeat: Option<RepeatElement>,
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
