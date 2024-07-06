#![allow(clippy::too_many_arguments)]
use crate::ir::MusicalPart;
use crate::ir::PartMap;
use muxml::muxml_types::EndingElement;
use muxml::muxml_types::KeyElement;
use muxml::muxml_types::RepeatElement;
use muxml::muxml_types::{
    ArticulationElement, ArticulationValue, AttributesElement, BackupElement, BarlineElement,
    ClefElement, DirectionElement, DirectionType, DirectionTypeElement, DynamicsElement, Measure,
    MeasureDirectionNote, Notations, NotationsElement, SlurElement, SlurType, SoundElement,
    TiedElement, TiedType, TimeElement, TimeModificationElement, TupletElement, TupletType,
    WordsElement,
};
use muxml::score::{CompleteParts, ScoreBuilder};
use muxml::ser::encode_muxml;

//use log::{debug, error};

use crate::ir::notation::{
    Arpeggiate, Articulation, BeatType, Beats, Chord, DescriptiveTempo, MeasureInitializer,
    MeasureMetaData, MeasureStartEnd, MusicElement, NoteConnection, NoteData, SlurConnection,
    TimeModification, TupletData, TupletStartStop, Voice,
};

use super::notation::get_staff;
use super::notation::NoteElementWrapper;

fn ser_measure_init(
    part: &MusicalPart,
    e: MeasureInitializer,
    m: &mut Measure,
    cur_measure_idx: i32,
    cur_beat: &mut Beats,
    cur_beat_type: &mut BeatType,
) {
    *cur_beat = e.beats;
    *cur_beat_type = e.beat_type;
    m.number = cur_measure_idx.to_string();
    m.attributes = Some(AttributesElement {
        divisions: part.get_initial_divisions().unwrap().to_string(),
        key: KeyElement {
            fifths: e.key_sig.to_string(),
        },
        time: TimeElement {
            beats: e.beats.to_string(),
            beat_type: e.beat_type.to_string(),
        },
        staves: "2".to_string(),
        clef: vec![
            ClefElement {
                number: "1".to_string(),
                sign: "G".to_string(),
            },
            ClefElement {
                number: "2".to_string(),
                sign: "F".to_string(),
            },
        ],
    });

    m.direction_note
        .push(MeasureDirectionNote::Direction(DirectionElement {
            direction_type: DirectionTypeElement {
                direction_type: DirectionType::Words(WordsElement {
                    value: DescriptiveTempo::from(e.tempo).to_string(),
                }),
            },
            staff: "1".to_string(),
            sound: Some(SoundElement {
                dynamics: None,
                tempo: Some(e.tempo.get_actual_f()),
            }),
        }));
}

fn ser_measure_meta(
    e: MeasureMetaData,
    m: &mut Measure,
    cur_measure_idx: &mut i32,
    measures: &mut Vec<Measure>,
    prev_voice: &mut Option<Voice>,
    measure_duration_tally: &mut u32,
) {
    match e.start_end {
        MeasureStartEnd::MeasureStart => {
            //println!("measure_idx: {}", cur_measure_idx);
            *prev_voice = None;
            *measure_duration_tally = 0;
            m.number = cur_measure_idx.to_string();
            if !e.ending.to_string().is_empty() {
                m.direction_note
                    .push(MeasureDirectionNote::Barline(BarlineElement {
                        location: Some("left".to_string()),
                        ending: (!e.ending.to_string().is_empty()).then(|| EndingElement {
                            number: Some(e.ending.to_string()),
                            r#type: Some("start".to_string()),
                            value: Some(e.ending.to_string()),
                        }),
                        repeat: None,
                    }));
            }
            //m.attributes = None;
            //m.direction_note = vec![];
        }
        MeasureStartEnd::MeasureEnd => {
            // Skip first case where there is no measure populated yet
            measures.push(m.clone());
            *m = Measure::default();
            //prev_measure_idx = cur_measure_idx;
            *cur_measure_idx += 1;
        }
        MeasureStartEnd::RepeatStart => {
            *prev_voice = None;
            *measure_duration_tally = 0;
            m.number = cur_measure_idx.to_string();
            m.direction_note
                .push(MeasureDirectionNote::Barline(BarlineElement {
                    location: Some("left".to_string()),
                    ending: (!e.ending.to_string().is_empty()).then(|| EndingElement {
                        number: Some(e.ending.to_string()),
                        r#type: Some("start".to_string()),
                        value: Some(e.ending.to_string()),
                    }),
                    repeat: Some(RepeatElement {
                        direction: Some("forward".to_string()),
                    }),
                }));
        }
        MeasureStartEnd::RepeatEnd => {
            m.direction_note
                .push(MeasureDirectionNote::Barline(BarlineElement {
                    location: Some("right".to_string()),
                    ending: (!e.ending.to_string().is_empty()).then(|| EndingElement {
                        number: Some(e.ending.to_string()),
                        r#type: Some("stop".to_string()),
                        value: None,
                    }),
                    repeat: Some(RepeatElement {
                        direction: Some("backward".to_string()),
                    }),
                }));
            measures.push(m.clone());
            *m = Measure::default();
            //prev_measure_idx = cur_measure_idx;
            *cur_measure_idx += 1;
        }
    };
}

fn ser_note_rest(
    part: &MusicalPart,
    e: NoteData,
    m: &mut Measure,
    _cur_measure_idx: i32,
    prev_voice: &mut Option<Voice>,
    measure_duration_tally: &mut u32,
    cur_tuplet_info: &mut Option<TupletElement>,
    cur_t_modification: &Option<TimeModificationElement>,
    cur_beat: Beats,
    cur_beat_type: BeatType,
) {
    // Build the notations Vec here
    let mut notations = None;
    let mut notations_elems = vec![];
    if let Some(val) = cur_tuplet_info {
        let te = val.clone();
        match te.r#type {
            TupletType::Stop => {
                panic!("Incorrectly formatted data. Tuplet Start should be handled elsewhere.")
            }
            TupletType::Start => {
                notations_elems.push(Notations::Tuplet(te));
                cur_tuplet_info.as_mut().unwrap().r#type = TupletType::None;
            }
            TupletType::None => (),
        }
    }

    if let Some(cur_dynamic) = e.phrase_dynamics.into() {
        m.direction_note
            .push(MeasureDirectionNote::Direction(DirectionElement {
                direction_type: DirectionTypeElement {
                    direction_type: DirectionType::Dynamics(DynamicsElement {
                        dynamics: Some(cur_dynamic),
                    }),
                },
                staff: get_staff(e.voice, part.get_num_voices()),
                sound: None,
            }));
    }
    // When the voice changes, a backup element is necessary to go back to the beginning of the measure
    // MusicXML requires a backup element to begin populating notes
    // at the beginning of the following measure. This is also where new dynamic
    // information is inserted for the Bass clef staff

    if let (Some(pv), v) = (*prev_voice, e.voice) {
        if pv != v {
            //println!("BACKUP DUR{}", measure_duration_tally);
            m.direction_note
                .push(MeasureDirectionNote::Backup(BackupElement {
                    duration: measure_duration_tally.to_string(),
                }));
            *measure_duration_tally = 0;
        }
    }
    // if cur_measure_idx == 69 {
    //     println!("voice: {:?}", e.voice);
    // }
    // if cur_measure_idx == 40 {
    //     println!("tally: {}", *measure_duration_tally);
    // }
    if e.chord.eq(&Chord::NoChord) {
        let val = e.get_duration_numeric(
            part.get_initial_divisions().unwrap(),
            u32::from(cur_beat),
            u32::from(cur_beat_type),
            cur_t_modification.as_ref().map(TimeModification::from),
        );
        //println!("curdur: {val}");
        *measure_duration_tally += val;
        //println!("mdt: {}", *measure_duration_tally);
    }

    if e.arpeggiate.eq(&Arpeggiate::Arpeggiate) {
        notations_elems.push(Notations::Arpeggiate);
    }
    match e.ties {
        NoteConnection::EndTie => {
            notations_elems.push(Notations::Tied(TiedElement {
                r#type: TiedType::Stop,
            }));
        }
        NoteConnection::None => {}
        NoteConnection::StartTie => {
            notations_elems.push(Notations::Tied(TiedElement {
                r#type: TiedType::Start,
            }));
        }
    }

    if e.articulation.ne(&Articulation::None) {
        //println!("Articulation: {}", e.articulation.to_string());
        notations_elems.push(Notations::Articulations(ArticulationElement {
            articulations: e.articulation.into(),
        }))
    }

    match e.slur {
        SlurConnection::EndSlur => {
            notations_elems.push(Notations::Slur(SlurElement {
                r#type: SlurType::Stop,
                number: "1".to_string(),
            }));
        }
        SlurConnection::None => {}
        SlurConnection::StartSlur => {
            notations_elems.push(Notations::Slur(SlurElement {
                r#type: SlurType::Start,
                number: "1".to_string(),
            }));
        }
    }

    if !notations_elems.is_empty() {
        notations = Some(NotationsElement {
            notations: notations_elems,
        });
    }
    let note_element_wrap = NoteElementWrapper::create_wrap(
        e,
        part.get_initial_divisions().unwrap(),
        cur_beat,
        cur_beat_type,
        cur_t_modification.as_ref().cloned(),
        notations,
        part.get_num_voices(),
    );
    m.direction_note.push(MeasureDirectionNote::Note(
        note_element_wrap.inner().clone(),
    ));
    *prev_voice = Some(e.voice);
}

fn ser_tuplet_data(
    t: TupletData,
    m: &mut Measure,
    cur_tuplet_info: &mut Option<TupletElement>,
    cur_t_modification: &mut Option<TimeModificationElement>,
) {
    *cur_t_modification = t.into();
    if t.start_stop == TupletStartStop::TupletStop {
        // Since Tuplet stop elements must come after the NoteData elements they encapsulate, but
        // MusicXML tracks the Stop Tuplet event as part of the Note tag,
        // we must search backwards through the measure to find the most
        // recent NoteData element and insert the TupletStop information there.
        for elem in m.direction_note.iter_mut().rev() {
            if let MeasureDirectionNote::Note(ne) = elem {
                // First extract the current tuplet tracking number, which must be populated if we are getting a TupletStop
                let tuplet_number = cur_tuplet_info.clone().unwrap().number;
                ne.insert_stop_tuple(tuplet_number);
                break;
            }
        }
    }

    // This must come last due to non-commutive property of state change
    *cur_tuplet_info = t.into();
}

impl From<Articulation> for ArticulationValue {
    fn from(t: Articulation) -> Self {
        match t {
            Articulation::None => ArticulationValue::None,
            Articulation::Accent => ArticulationValue::Accent,
            Articulation::StrongAccent => ArticulationValue::StrongAccent,
            Articulation::Staccato => ArticulationValue::Staccato,
            Articulation::Staccatissimo => ArticulationValue::Staccatissimo,
            Articulation::Tenuto => ArticulationValue::Tenuto,
            Articulation::DetachedLegato => ArticulationValue::DetachedLegato,
            Articulation::Stress => ArticulationValue::Stress,
        }
    }
}

fn from_musical_part(t: &MusicalPart) -> Vec<Measure> {
    // If the number of voices is 2, voice 1 goes to Treble Cleff, 2 to Bass Clef
    // If the number of voices is 4, voice 1-2 goes to Treble Cleff, 2-3 to Bass Clef
    // However, there will need to be additional heuristics for properly notating based on actual note octaves
    // in the future.

    if t.get_initial_divisions().is_none() || t.get_num_voices() == 0 {
        return vec![];
    }
    let mut measures: Vec<Measure> = vec![];
    let mut cur_measure = Measure::default(); // Measure element currently being serialized
    let mut cur_measure_idx = 1;
    let mut cur_tuplet_info: Option<TupletElement> = None;
    let mut cur_t_modification: Option<TimeModificationElement> = None;
    let mut prev_voice = None;
    let mut measure_duration_tally = 0;
    let mut cur_beat = Beats::default();
    let mut cur_beat_type = BeatType::default();

    for elem in t.inner() {
        match *elem {
            MusicElement::MeasureInit(e) => ser_measure_init(
                t,
                e,
                &mut cur_measure,
                cur_measure_idx,
                &mut cur_beat,
                &mut cur_beat_type,
            ),
            MusicElement::MeasureMeta(e) => ser_measure_meta(
                e,
                &mut cur_measure,
                &mut cur_measure_idx,
                &mut measures,
                &mut prev_voice,
                &mut measure_duration_tally,
            ),
            MusicElement::NoteRest(e) => ser_note_rest(
                t,
                e,
                &mut cur_measure,
                cur_measure_idx,
                &mut prev_voice,
                &mut measure_duration_tally,
                &mut cur_tuplet_info,
                &cur_t_modification,
                cur_beat,
                cur_beat_type,
            ),
            MusicElement::Tuplet(t) => ser_tuplet_data(
                t,
                &mut cur_measure,
                &mut cur_tuplet_info,
                &mut cur_t_modification,
            ),
        }
    }
    measures
}

impl From<&MusicalPart> for Vec<Measure> {
    fn from(t: &MusicalPart) -> Self {
        from_musical_part(t)
    }
}

impl From<MusicalPart> for Vec<Measure> {
    fn from(t: MusicalPart) -> Self {
        from_musical_part(&t)
    }
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

pub fn ir_to_xml(parts: PartMap) -> String {
    let complete_parts: CompleteParts = parts
        .try_into()
        .expect("Failed to convert PartMap into CompleteParts");

    let score = ScoreBuilder::new()
        .work_title("Untitled".to_string())
        .composer("Untitled".to_string())
        .software("muxml rust crate".to_string())
        .encoding_date("2023-11-22".to_string())
        .complete_parts(complete_parts)
        .build();

    encode_muxml(score)
}
