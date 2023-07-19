use crate::{music_xml_types::*, notation::MusicElement};
use quick_xml::se::to_string;

pub fn serialize_xml(music: Vec<MusicElement>) {
    let measures: Vec<Measure> = vec![];
    for elem in music {
        match elem {
            MusicElement::MeasureInit(e) => {
                let trebel_direction_elem = vec![
                    DirectionElement {
                        direction_type: DirectionTypeElement {
                            direction_type: DirectionType::Words(WordsElement {
                                value: e.tempo.to_string(),
                            }),
                        },
                        staff: "1".to_string(),
                        sound: Some(SoundElement {
                            dynamics: None,
                            tempo: Some(e.tempo.as_float()),
                        }),
                    },
                    DirectionElement {
                        direction_type: DirectionTypeElement {
                            direction_type: DirectionType::Dynamics(MXmlDynamics::from_dynamics(
                                e.treble_dynamics,
                            )),
                        },
                        staff: "1".to_string(),
                        sound: None,
                    },
                ];
                let bass_direction_elem = vec![
                    
                    DirectionElement {
                        direction_type: DirectionTypeElement {
                            direction_type: DirectionType::Dynamics(MXmlDynamics::from_dynamics(
                                e.bass_dynamics,
                            )),
                        },
                        staff: "2".to_string(),
                        sound: None,
                    },
                ];
                let attributes_elem = Some(AttributesElement {
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
            }
            MusicElement::MeasureMeta(e) => {}
            MusicElement::NoteRest(e) => {}
        }
    }
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
                encoding_date: "11/22/2023".to_string(),
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
            measure: vec![
                Measure {
                    number: "1".to_string(),
                    attributes: Some(AttributesElement {
                        divisions: "1".to_string(),
                        key: KeyElement {
                            fifths: "1".to_string(),
                        },
                        time: TimeElement {
                            beats: "1".to_string(),
                            beat_type: "2".to_string(),
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
                    }),
                    direction: vec![DirectionElement {
                        direction_type: DirectionTypeElement {
                            direction_type: DirectionType::Coda,
                        },
                        staff: "1".to_string(),
                        sound: None,
                    }],
                    note: vec![NoteElement {
                        pitch: PitchRest::Rest,
                        duration: 2,
                        r#type: "quarter".to_string(),
                        staff: 1,
                        notations: None,
                    }],
                },
                Measure {
                    number: "2".to_string(),
                    attributes: None,
                    direction: vec![DirectionElement {
                        direction_type: DirectionTypeElement {
                            direction_type: DirectionType::Coda,
                        },
                        staff: "1".to_string(),
                        sound: None,
                    }],
                    note: vec![NoteElement {
                        pitch: PitchRest::Rest,
                        duration: 2,
                        r#type: "quarter".to_string(),
                        staff: 1,
                        notations: None,
                    }],
                },
            ],
        }],
    };

    let serialized_item = to_string(&item).unwrap();
    println!("{}", serialized_item)
}
