use crate::{
    music_xml_types::*,
    notation::{DescriptiveTempo, MusicElement, TrebleBassClef, TupletNumber, TupletStartStop},
};
use quick_xml::se::to_string;

pub fn serialize_xml(music: Vec<MusicElement>) {
    let mut measures: Vec<Measure> = vec![];
    let mut cur_measure = Measure::default(); // Measure element currently being serialized
    let mut cur_measure_idx = 0;
    let mut prev_measure_idx = 0;
    let mut notations: Option<Vec<Notations>> = None;
    let mut time_modification: Option<TimeModificationElement> = None;
    for elem in music {
        match elem {
            MusicElement::MeasureInit(e) => {
                if cur_measure_idx == (prev_measure_idx + 1) {
                    // Skip first case where there is no measure populated yet
                    measures.push(cur_measure.clone());
                    cur_measure = Measure::default();
                }
                prev_measure_idx = cur_measure_idx;
                cur_measure_idx += 1;

                let m = &mut cur_measure;
                let bass_clef_direction_elem = DirectionElement {
                    direction_type: DirectionTypeElement {
                        direction_type: DirectionType::Dynamics(MXmlDynamics::from_dynamics(
                            e.bass_dynamics,
                        )),
                    },
                    staff: "2".to_string(),
                    sound: None,
                };

                m.number = cur_measure_idx.to_string();
                m.attributes = Some(AttributesElement {
                    divisions: "1".to_string(),
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

                m.direction_note
                    .push(MeasureDirectionNote::Direction(DirectionElement {
                        direction_type: DirectionTypeElement {
                            direction_type: DirectionType::Dynamics(MXmlDynamics::from_dynamics(
                                e.treble_dynamics,
                            )),
                        },
                        staff: "1".to_string(),
                        sound: None,
                    }));
            }
            MusicElement::MeasureMeta(e) => {
                if cur_measure_idx == (prev_measure_idx + 1) {
                    // Skip first case where there is no measure populated yet
                    measures.push(cur_measure.clone());
                    cur_measure = Measure::default();
                }
                prev_measure_idx = cur_measure_idx;
                cur_measure_idx += 1;
                let m = &mut cur_measure;

                m.number = cur_measure_idx.to_string();
                m.attributes = None;

                m.direction_note = vec![];
            }
            MusicElement::NoteRest(e) => {
                //let meas_ref = measures.
                let m = &mut cur_measure;
                m.direction_note
                    .push(MeasureDirectionNote::Note(NoteElement {
                        chord: Some(ChordElement()),
                        pitch: PitchRest::from(e.note_rest),
                        duration: "4".to_string(),
                        r#type: e.rhythm_value.get_type_string(),
                        staff: match e.treble_bass {
                            TrebleBassClef::TrebleClef => "1".to_string(),
                            TrebleBassClef::BassClef => "2".to_string(),
                        },
                        notations: notations.clone(),
                        time_modification: time_modification,
                    }));
            }
            MusicElement::Tuplet(t) => {
                let tmp_t_modification = TimeModificationElement {
                    actual_notes: t.actual_notes,
                    normal_notes: t.normal_notes,
                };
                time_modification = Some(tmp_t_modification);
                if let Some(o_tup_type) = match t.start_stop {
                    TupletStartStop::TupletStart => Some(TupletType::Start),
                    TupletStartStop::TupletNone => None,
                    TupletStartStop::TupletStop => Some(TupletType::Stop),
                } {
                    let tmp_not = Notations::Tuplet(TupletElement {
                        r#type: o_tup_type,
                        number: match t.tuplet_number {
                            TupletNumber::TupletOne => MXmlTupletNumber::TupletOne,
                            TupletNumber::TupletTwo => MXmlTupletNumber::TupletTwo,
                        },
                    });
                    if let Some(n) = &mut notations {
                        n.push(tmp_not);
                    } else {
                        notations = Some(vec![tmp_not]);
                    }
                };
            }
        }
    }
    // Push last measure
    measures.push(cur_measure.clone());
    println!("measures is {}", measures.len());
    let item = ScorePartWise {
        version: "4.0".to_string(),
        work: WorkElement {
            work_title: "Untitled".to_string(),
        },
        identification: IdentificationElement {
            creator: CreatorElement {
                r#type: "composer".to_string(),
                value: "Composer / Arranger".to_string(),
            },
            encoding: EncodingElement {
                software: "xml2bin".to_string(),
                encoding_date: "2023-11-22".to_string(),
                supports: vec![
                    SupportsElement {
                        element: "accidental".to_string(),
                        r#type: "yes".to_string(),
                    },
                    SupportsElement {
                        element: "beam".to_string(),
                        r#type: "yes".to_string(),
                    },
                ],
            },
        },
        part_list: vec![PartListElement {
            score_part: ScorePart {
                id: "P1".to_string(),
                part_name: "Piano".to_string(),
            },
        }],
        part: vec![Part {
            id: "P1".to_string(),
            measure: measures,
            //  vec![
            //     Measure {
            //         number: "1".to_string(),
            //         attributes: Some(AttributesElement {
            //             divisions: "1".to_string(),
            //             key: KeyElement {
            //                 fifths: "1".to_string(),
            //             },
            //             time: TimeElement {
            //                 beats: "1".to_string(),
            //                 beat_type: "2".to_string(),
            //             },
            //             staves: "2".to_string(),
            //             clef: vec![
            //                 ClefElement {
            //                     number: "1".to_string(),
            //                     sign: "G".to_string(),
            //                 },
            //                 ClefElement {
            //                     number: "2".to_string(),
            //                     sign: "F".to_string(),
            //                 },
            //             ],
            //         }),
            //         direction: vec![DirectionElement {
            //             direction_type: DirectionTypeElement {
            //                 direction_type: DirectionType::Coda,
            //             },
            //             staff: "1".to_string(),
            //             sound: None,
            //         }],
            //         note: vec![NoteElement {
            //             pitch: PitchRest::Rest,
            //             duration: 2,
            //             r#type: "quarter".to_string(),
            //             staff: 1,
            //             notations: None,
            //         }],
            //     },
            //     Measure {
            //         number: "2".to_string(),
            //         attributes: None,
            //         direction: vec![DirectionElement {
            //             direction_type: DirectionTypeElement {
            //                 direction_type: DirectionType::Coda,
            //             },
            //             staff: "1".to_string(),
            //             sound: None,
            //         }],
            //         note: vec![NoteElement {
            //             pitch: PitchRest::Rest,
            //             duration: 2,
            //             r#type: "quarter".to_string(),
            //             staff: 1,
            //             notations: None,
            //         }],
            //     },
            // ],
        }],
    };
    let xml_header = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>".to_string();
    let serialized_item = to_string(&item).unwrap();
    println!("{}", xml_header);
    println!("{}", serialized_item);
}
