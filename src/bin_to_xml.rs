use crate::bin_decoder::MusicDecoder;
use crate::error::{Error, Result};
use crate::utils::NL;
use crate::{music_xml_types::*, notation::*};
use log::{debug, error};
use quick_xml::se::Serializer;
use serde::Serialize;

use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;
const MAX_SUPPORTED_VOICES: usize = 4;

fn handle_measure_init(
    e: MeasureInitializer,
    m: &mut Measure,
    divisions: u32,
    cur_measure_idx: i32,
    cur_beat: &mut Beats,
    cur_beat_type: &mut BeatType,
) {
    *cur_beat = e.beats;
    *cur_beat_type = e.beat_type;
    m.number = cur_measure_idx.to_string();
    m.attributes = Some(AttributesElement {
        divisions: divisions.to_string(),
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

fn handle_measure_meta(
    e: MeasureMetaData,
    m: &mut Measure,
    cur_measure_idx: &mut i32,
    measures: &mut Vec<Measure>,
    prev_voice: &mut Option<Voice>,
    measure_duration_tally: &mut u32,
) {
    match e.start_end {
        MeasureStartEnd::MeasureStart => {
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
            if !e.ending.to_string().is_empty() {
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

fn handle_note_rest(
    e: NoteData,
    m: &mut Measure,
    divisions: u32,
    prev_voice: &mut Option<Voice>,
    measure_duration_tally: &mut u32,
    cur_tuplet_info: &mut Option<TupletElement>,
    cur_t_modification: &Option<TimeModificationElement>,
    cur_beat: Beats,
    cur_beat_type: BeatType,
    num_voices: usize,
) {
    // Build the notations Vec here
    let mut notations = None;
    let mut notations_elems = vec![];
    if cur_tuplet_info.is_some() {
        let te = cur_tuplet_info.clone().unwrap();
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

    if let Some(cur_dynamic) = DynamicsValue::from_dynamics(e.phrase_dynamics) {
        m.direction_note
            .push(MeasureDirectionNote::Direction(DirectionElement {
                direction_type: DirectionTypeElement {
                    direction_type: DirectionType::Dynamics(DynamicsElement {
                        dynamics: Some(cur_dynamic),
                    }),
                },
                staff: NoteElement::get_staff(e.voice, num_voices),
                sound: None,
            }));
    }

    // When the voice changes, a backup element is necessary to go back to the beginning of the measure
    // MusicXML requires a backup element to begin populating notes
    // at the beginning of the following measure. This is also where new dynamic
    // information is inserted for the Bass clef staff
    if let Some(t) = *prev_voice {
        if t != e.voice {
            m.direction_note
                .push(MeasureDirectionNote::Backup(BackupElement {
                    duration: measure_duration_tally.to_string(),
                }));
        }
    }
    if e.special_note == SpecialNote::None {
        *measure_duration_tally += e.get_duration_numeric(
            divisions,
            u32::from(cur_beat),
            u32::from(cur_beat_type),
            cur_t_modification.map(|v| TimeModification::from(v)),
        );
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
        NoteConnection::NoTie => {}
        NoteConnection::StartTie => {
            notations_elems.push(Notations::Tied(TiedElement {
                r#type: TiedType::Start,
            }));
        }
    }
    match e.slur {
        SlurConnection::EndSlur => {
            notations_elems.push(Notations::Slur(SlurElement {
                r#type: SlurType::Stop,
                number: "1".to_string(),
            }));
        }
        SlurConnection::NoSlur => {}
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
    m.direction_note
        .push(MeasureDirectionNote::Note(NoteElement::new(
            e,
            divisions,
            cur_beat,
            cur_beat_type,
            *cur_t_modification,
            notations,
            num_voices,
        )));
    *prev_voice = Some(e.voice);
    //prev_clef = Some(e.treble_bass);
}

fn handle_tuplet_data(
    t: TupletData,
    m: &mut Measure,
    cur_tuplet_info: &mut Option<TupletElement>,
    cur_t_modification: &mut Option<TimeModificationElement>,
) {
    match t.start_stop {
        TupletStartStop::TupletStart => {
            *cur_t_modification = Some(TimeModificationElement {
                actual_notes: t.actual_notes,
                normal_notes: t.normal_notes,
            });
            *cur_tuplet_info = Some(TupletElement {
                r#type: TupletType::Start,
                number: match t.tuplet_number {
                    TupletNumber::TupletOne => "1".to_string(),
                    TupletNumber::TupletTwo => "2".to_string(),
                },
            });
        }
        TupletStartStop::TupletNone => {
            *cur_tuplet_info = None;
        }
        TupletStartStop::TupletStop => {
            // Since Tuplet stop elements must come after the NoteData elements they encapsulate, but
            // MusicXML tracks the Stop Tuplet event as part of the Note tag,
            // we must search backwards through the measure to find the most
            // recent NoteData element and insert the TupletStop information there.
            for elem in m.direction_note.iter_mut().rev() {
                match elem {
                    MeasureDirectionNote::Note(ne) => {
                        // First extract the current tuplet tracking number, which must be populated if we are getting a TupletStop
                        let tuplet_number = cur_tuplet_info.clone().unwrap().number;
                        ne.insert_stop_tuple(tuplet_number);
                        break;
                    }
                    _ => (),
                }
            }
            *cur_tuplet_info = None;
            *cur_t_modification = None;
        }
    }
}

fn serialize_xml(music: Vec<MusicElement>, divisions: u32, num_voices: usize) -> String {
    // If the number of voices is 2, voice 1 goes to Treble Cleff, 2 to Bass Clef
    // If the number of voices is 4, voice 1-2 goes to Treble Cleff, 2-3 to Bass Clef
    // However, there will need to be additional heuristics for properly notating based on actual note octaves
    // in the future.
    let mut measures: Vec<Measure> = vec![];
    let mut cur_measure = Measure::default(); // Measure element currently being serialized
    let mut cur_measure_idx = 1;
    let mut cur_tuplet_info: Option<TupletElement> = None;
    let mut cur_t_modification: Option<TimeModificationElement> = None;
    let mut prev_voice = None;
    let mut measure_duration_tally = 0;
    let mut cur_beat = Beats::default();
    let mut cur_beat_type = BeatType::default();
    for elem in music {
        match elem {
            MusicElement::MeasureInit(e) => handle_measure_init(
                e,
                &mut cur_measure,
                divisions,
                cur_measure_idx,
                &mut cur_beat,
                &mut cur_beat_type,
            ),
            MusicElement::MeasureMeta(e) => handle_measure_meta(
                e,
                &mut cur_measure,
                &mut cur_measure_idx,
                &mut measures,
                &mut prev_voice,
                &mut measure_duration_tally,
            ),
            MusicElement::NoteRest(e) => handle_note_rest(
                e,
                &mut cur_measure,
                divisions,
                &mut prev_voice,
                &mut measure_duration_tally,
                &mut cur_tuplet_info,
                &mut cur_t_modification,
                cur_beat,
                cur_beat_type,
                num_voices,
            ),
            MusicElement::Tuplet(t) => handle_tuplet_data(
                t,
                &mut cur_measure,
                &mut cur_tuplet_info,
                &mut cur_t_modification,
            ),
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
        }],
    };
    let mut xml_string = format!("<?xml version=\"1.0\" encoding=\"UTF-8\"?>{0}<!DOCTYPE score-partwise PUBLIC \"-//Recordare//DTD MusicXML 4.0 Partwise//EN\" \"http://www.musicxml.org/dtds/partwise.dtd\">{0}", NL);
    let mut ser = Serializer::new(&mut xml_string);
    ser.indent(' ', 2);
    item.serialize(ser).unwrap();
    xml_string
}

pub fn process_bin_to_xml(input: PathBuf, output: PathBuf, dump_input: bool) -> Result<()> {
    let mut outfile = File::create(output).expect("IO Error occurred on file create()");
    let infile = File::open(input).expect("IO Error occurred on file open()");
    let reader = BufReader::new(infile);
    let mut music_decoder = MusicDecoder::new(Some(reader));
    music_decoder.reader_read()?;

    let parsed = music_decoder.parse_data()?;

    // For tuplets, the associated note type is embedded in the NoteData type. The Tuplet data information element
    // precedes the note data element, so to determine the shortest value represented in the piece, both the tuplet information
    // is needed and all of the notes within the tuplet section. For the minimum, we're looking for the shortest note type
    // that is within a tuplet, and the most actual notes within the number of normal notes indicated in the Tuplet data
    // and finding a LCM (least common multiple) for them
    let (divisions, voice_len) = calc_divisions_voices(parsed, dump_input);

    if voice_len > MAX_SUPPORTED_VOICES {
        error!(
            "Maximum supported voices is {MAX_SUPPORTED_VOICES} but piece contains {}.",
            voice_len
        );
        return Err(Error::OutofBounds);
    }

    debug!("Divisions is {divisions}. Voices is {}", voice_len);
    let output = serialize_xml(music_decoder.parse_data()?, divisions, voice_len);
    outfile
        .write_all(output.as_bytes())
        .expect("IO Error occurred on write_all()");
    Ok(())
}
