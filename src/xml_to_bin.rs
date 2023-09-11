use crate::bin_encoder::MusicEncoder;
use crate::error::Result;
use crate::notation::*;

use log::{error, info, trace};
use num_traits::FromPrimitive;
use roxmltree::*;
use std::collections::BTreeSet;
use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::str::FromStr;

const MAX_SUPPORTED_VOICES: usize = 4;

fn handle_backup_tag(measure_element: &Node<'_, '_>, measure_checker: &mut MeasureChecker) {
    let duration_tag = measure_element
        .first_element_child()
        .unwrap()
        .text()
        .unwrap();
    let duration_val = duration_tag.parse::<u32>().unwrap();
    // If the backup tag did not move fully back to measure start time before
    // the new voice notes were inserted, we must insert a placeholder rest
    // as a substitute for the time, because musicbin format does not have a concept of backup or support incomplete
    // measures or voices beginning in the middle of the measure
    measure_checker.conform_backup_placeholder_rests(duration_val as usize);
}

fn handle_direction_tag(measure_element: &Node<'_, '_>) -> Option<PhraseDynamics> {
    //Dynamics::from_str(t.tag_name().name()).expect("Unsupported dynamic type found.")
    let dynamics_tag = measure_element
        .children()
        .find(|n| n.has_tag_name("dynamics"));

    if dynamics_tag.is_some() {
        match PhraseDynamics::from_str(
            dynamics_tag
                .unwrap()
                .first_element_child()
                .unwrap()
                .tag_name()
                .name(),
        ) {
            Ok(t) => Some(t),
            Err(_) => None,
        }
    } else {
        None
    }
}

fn handle_note_tag(
    measure_element: &Node<'_, '_>,
    measure_checker: &mut MeasureChecker,
    dynamic_value: &mut Option<PhraseDynamics>,
    voices: &mut BTreeSet<u8>,
) {
    let mut note_data = NoteData::default();
    let mut stop_tuplet_elem: Option<MusicElement> = None;
    let note_type = measure_element.children().find(|n| n.has_tag_name("type"));
    let xml_note_duration = measure_element
        .children()
        .find(|n| n.has_tag_name("duration"));
    let dotted = measure_element.children().find(|n| n.has_tag_name("dot"));
    let grace_note = measure_element.children().find(|n| n.has_tag_name("grace"));
    note_data.special_note = match grace_note {
        Some(n) => match n.attribute("slash") {
            None => SpecialNote::None,
            Some(t) => SpecialNote::from_str(t).expect("Unsupported Tied Type"),
        },
        None => SpecialNote::None,
    };

    if let Some(_) = dotted {
        note_data.dotted = true;
    }

    let time_mod_tag = measure_element
        .children()
        .find(|n| n.has_tag_name("time-modification"));
    let notations_tag = measure_element
        .children()
        .find(|n| n.has_tag_name("notations"));
    let rest_tag = measure_element.children().find(|n| n.has_tag_name("rest"));
    let voice_text = measure_element
        .children()
        .find(|n| n.has_tag_name("voice"))
        .unwrap()
        .text()
        .unwrap();
    let voice_num = voice_text
        .parse::<u8>()
        .expect("Unable to parse voices string");
    voices.insert(voice_num);

    let mut skip_note: bool = false;
    // Convert the arbitrary 4 voice max numbering to a normalized 1-4 voice numbering in the binary format
    for (idx, voice) in voices.clone().iter().enumerate() {
        if voice_num == *voice {
            if (idx as usize) > (MAX_SUPPORTED_VOICES - 1) {
                skip_note = true;
                break;
            } else {
                note_data.voice = FromPrimitive::from_u8((idx) as u8)
                    .expect("Unable to FromPrimitive on u8 to voice type.");
            }
        }
    }

    if skip_note {
        println!("skip_note_case");
        return;
    }

    let mut time_mod_value: Option<TimeModification> = None;
    match time_mod_tag {
        Some(n) => {
            let actual_notes_tag = n.children().find(|n| n.has_tag_name("actual-notes"));
            let normal_notes_tag = n.children().find(|n| n.has_tag_name("normal-notes"));
            if actual_notes_tag.is_some() && normal_notes_tag.is_some() {
                let actual_notes = actual_notes_tag
                    .unwrap()
                    .text()
                    .unwrap()
                    .parse::<u8>()
                    .unwrap();
                let normal_notes = normal_notes_tag
                    .unwrap()
                    .text()
                    .unwrap()
                    .parse::<u8>()
                    .unwrap();
                time_mod_value = Some(TimeModification::new(actual_notes, normal_notes));
            }
        }
        None => (),
    }

    if let Some(phrase_dyn) = dynamic_value {
        note_data.phrase_dynamics = *phrase_dyn;
        *dynamic_value = None;
    }
    match notations_tag {
        Some(n) => {
            let tuplet_tag = n.children().find(|n| n.has_tag_name("tuplet"));
            let tied_tag = n.children().find(|n| n.has_tag_name("tied"));
            let slur_tag = n.children().find(|n| n.has_tag_name("slur"));
            let arp_tag = n.children().find(|n| n.has_tag_name("arpeggiate"));
            let artic_tag = n.children().find(|n| n.has_tag_name("articulations"));

            note_data.ties = match tied_tag {
                Some(t) => NoteConnection::from_str(t.attribute("type").unwrap())
                    .expect("Unsupported Tied Type"),
                None => NoteConnection::NoTie,
            };

            note_data.arpeggiate = match arp_tag {
                Some(_t) => Arpeggiate::Arpeggiate,
                None => Arpeggiate::NoArpeggiation,
            };

            let _artic = match artic_tag {
                Some(t) => {
                    match t.first_child().unwrap().tag_name().name() {
                        "staccato" => Articulation::Stacatto,
                        "strong-accent" => Articulation::Marcato,
                        "accent" => {
                            // Stressed beat case, handled outside of Articulation case
                            note_data.stress = Stress::Accented;
                            Articulation::None
                        }
                        _ => {
                            // Unsupported articulation tag
                            Articulation::None
                        }
                    }
                }
                None => Articulation::None,
            };

            note_data.slur = match slur_tag {
                Some(t) => SlurConnection::from_str(t.attribute("type").unwrap())
                    .expect("Unhandled slur tag attribute case"),
                None => SlurConnection::NoSlur,
            };

            if note_data.slur.eq(&SlurConnection::StartSlur) {
                // Legato overrides other articulations, except accents, which are handled independently
                note_data.articulation = Articulation::Legato;
            }

            // if tuplet_tag.is_none() {
            //     println!("No Tuple Case: measure {measure_idx}");
            // }

            match tuplet_tag {
                Some(t) => match t.attribute("type").unwrap() {
                    "start" => {
                        measure_checker.push_elem(MusicElement::Tuplet(TupletData {
                            start_stop: TupletStartStop::TupletStart,
                            tuplet_number: TupletNumber::TupletOne,
                            actual_notes: time_mod_value.unwrap().get_actual(),
                            normal_notes: time_mod_value.unwrap().get_normal(),
                            dotted: false,
                        }));
                    }
                    "stop" => {
                        stop_tuplet_elem = Some(MusicElement::Tuplet(TupletData {
                            start_stop: TupletStartStop::TupletStop,
                            tuplet_number: TupletNumber::TupletOne,
                            actual_notes: time_mod_value.unwrap().get_actual(),
                            normal_notes: time_mod_value.unwrap().get_normal(),
                            dotted: false,
                        }));
                    }
                    _ => {
                        panic!("Unhandled tuplet tag attribute case");
                    }
                },
                None => {
                    if measure_checker.measure_idx() == 27 {}
                    ()
                }
            }
        }
        None => (),
    }

    note_data.note_type = if let Some(n) = note_type {
        NoteType::from_str(n.text().unwrap()).unwrap()
    } else {
        // Whole rests sometimes provide no "type" tag, but whole rests are different durations
        // depending on the time signature, so we must manually calculate the rhythm value based on duration
        if let Some(n) = xml_note_duration {
            if let Some((rest_duration, is_dotted)) = NoteData::from_numeric_duration(
                n.text().unwrap().parse::<u32>().unwrap(),
                measure_checker.quarter_division(),
            ) {
                note_data.dotted = is_dotted;
                rest_duration
            } else {
                panic!("Could not convert numeric duration value to internal note duration representation");
            }
        } else {
            panic!("No note duration provided.");
        }
    };

    match rest_tag {
        Some(_) => {
            //debug!("rest {:?}", note_data.rhythm_value);
            note_data.note_rest = NoteRestValue::Rest;
        }
        None => {
            let chord_tag = measure_element.children().find(|n| n.has_tag_name("chord"));
            let pitch_tag = measure_element
                .children()
                .find(|n| n.has_tag_name("pitch"))
                .unwrap();
            let step_tag = pitch_tag.children().find(|n| n.has_tag_name("step"));
            let octave_tag = pitch_tag.children().find(|n| n.has_tag_name("octave"));
            let alter_tag = pitch_tag.children().find(|n| n.has_tag_name("alter"));

            // alter tags are optional, others are mandatory
            let alter_note = match alter_tag {
                Some(t) => Alter::from_str(t.text().unwrap()).unwrap(),
                None => Alter::None,
            };
            note_data.chord = match chord_tag {
                Some(_t) => Chord::Chord,
                None => Chord::NoChord,
            };
            note_data.note_rest = NoteRestValue::derive_numeric_note(
                Note::from_str(step_tag.unwrap().text().unwrap()).unwrap(),
                alter_note,
                Octave::from_str(octave_tag.unwrap().text().unwrap()).unwrap(),
            )
            .expect("Parsed note is not supported by Music2Bin format.");
            //debug!(
            //    "note {:?} number: {:?}",
            //    note_data.rhythm_value, note_data.note_rest
            //);
        }
    }

    // The MeasureChecker checks for correct total duration. Incomplete voices are thrown away.
    measure_checker.push_elem(MusicElement::NoteRest(note_data));
    if stop_tuplet_elem.is_some() {
        measure_checker.push_elem(stop_tuplet_elem.unwrap());
    }
}

pub fn process_xml_to_bin(input: PathBuf, output: PathBuf, dump_output: bool) -> Result<()> {
    let outfile = File::create(output).expect("IO Error Occurred");

    let mut complete_music: Vec<MusicElement> = Vec::new();
    let docstring = fs::read_to_string(input).unwrap();
    let opt = ParsingOptions {
        allow_dtd: true,
        ..ParsingOptions::default()
    };
    let document = Document::parse_with_options(&docstring, opt).expect("Doc failed to parse.");

    let measures = document
        .root_element()
        .descendants()
        .filter(|n| n.has_tag_name("measure"));
    let mut quarter_division = 0;
    let mut measure_init_present: bool = false;
    let mut prev_measure_init = MeasureInitializer::default();
    let mut voices = BTreeSet::new();

    for (measure_idx, measure) in measures.enumerate() {
        println!("Measure_idx {measure_idx} start");
        //let mut inserted_note_tally = 0;
        let mut measure_init = prev_measure_init;
        let mut measure_meta_start = MeasureMetaData::new(MeasureStartEnd::MeasureStart);
        let mut measure_meta_end = MeasureMetaData::new(MeasureStartEnd::MeasureEnd);

        if measure_idx == 0 {
            if let Some(div) = measure.descendants().find(|n| n.has_tag_name("divisions")) {
                quarter_division = div.text().unwrap().parse::<u32>().unwrap();
            } else {
                panic!("No divisions tag found.");
            }
        }

        match measure.descendants().find(|n| n.has_tag_name("time")) {
            Some(t) => {
                let beats_node = t.children().find(|n| n.has_tag_name("beats")).unwrap();
                let beat_type_node = t.children().find(|n| n.has_tag_name("beat-type")).unwrap();

                measure_init.beats = Beats::from_str(beats_node.text().unwrap()).unwrap();
                measure_init.beat_type =
                    BeatType::from_str(beat_type_node.text().unwrap()).unwrap();
            }
            _ => (),
        };

        match measure.descendants().find(|n| n.has_tag_name("repeat")) {
            Some(n) => {
                let measure_direction_str = n.attribute("direction").unwrap();
                if measure_direction_str.eq("backward") {
                    measure_meta_end.start_end = MeasureStartEnd::RepeatEnd;
                } else if measure_direction_str.eq("forward") {
                    measure_meta_start.start_end = MeasureStartEnd::RepeatStart;
                } else {
                    // Unsupported direction attribute
                    panic!(
                        "Encountered unsupported repeat direction attribute: {}",
                        measure_direction_str
                    );
                }
            }
            _ => (),
        };

        let barlines = measure.descendants().filter(|n| n.has_tag_name("barline"));
        for barline in barlines {
            match barline.descendants().find(|n| n.has_tag_name("ending")) {
                Some(n) => {
                    let ending_type_str = n.attribute("type").unwrap();
                    let ending_number_str = n.attribute("number").unwrap();
                    match ending_type_str {
                        "start" => {
                            measure_meta_start.ending = Ending::from_str(ending_number_str)
                                .expect("Invalid Ending string.");
                        }
                        "stop" => {
                            // Used for first endings with a "downward jog"
                            measure_meta_end.ending = Ending::from_str(ending_number_str)
                                .expect("Invalid Ending string.");
                        }
                        "discontinue" => {
                            // Used for second endings with no "downward jog"
                            measure_meta_end.ending = Ending::from_str(ending_number_str)
                                .expect("Invalid Ending string.");
                        }
                        t => {
                            panic!("Encountered unsupported measure ending type {}", t);
                        }
                    }
                }
                _ => (),
            };
        }

        if let Some(keysig) = match measure.descendants().find(|n| n.has_tag_name("fifths")) {
            Some(n) => KeySignature::from_str(n.text().unwrap()).ok(),
            None => None,
        } {
            measure_init.key_sig = keysig;
        }

        if let Some(tempo) = match measure
            .descendants()
            .find(|n| n.has_tag_name("sound") && n.attribute("tempo").is_some())
        {
            Some(n) => Tempo::from_str(n.attribute("tempo").unwrap()).ok(),
            None => None,
        } {
            measure_init.tempo = tempo;
        }

        if measure_init_present == false {
            complete_music.push(MusicElement::MeasureInit(measure_init));
            measure_init_present = true;
        } else if measure_init != prev_measure_init {
            // If the current measure initializer state tracker deviates from the ongoing tracked value, insert a new measure initializer into the composition
            // to track the changed values.
            complete_music.push(MusicElement::MeasureInit(measure_init));
        }

        let measure_tags = measure.children().filter(|n| {
            n.has_tag_name("note") || n.has_tag_name("direction") || n.has_tag_name("backup")
        });

        prev_measure_init = measure_init;
        let mut dynamic_value = None;
        let mut measure_checker = MeasureChecker::new(quarter_division, &measure_init, measure_idx);
        complete_music.push(MusicElement::MeasureMeta(measure_meta_start));
        for measure_element in measure_tags {
            if measure_element.tag_name().name() == "note" {
                handle_note_tag(
                    &measure_element,
                    &mut measure_checker,
                    &mut dynamic_value,
                    &mut voices,
                );
            } else if measure_element.tag_name().name() == "direction" {
                dynamic_value = handle_direction_tag(&measure_element);
            } else if measure_element.tag_name().name() == "backup" {
                handle_backup_tag(&measure_element, &mut measure_checker);
                println!("");
            }
        }
        println!("");
        //measure_checker.remove_incomplete_voices(&voices);
        complete_music.append(measure_checker.as_inner());
        complete_music.push(MusicElement::MeasureMeta(measure_meta_end));
    }

    let voice_cnt;
    if voices.len() > MAX_SUPPORTED_VOICES {
        info!(
            "Maximum supported voices is {MAX_SUPPORTED_VOICES} but piece contains {}. Threw away additional voices",
            voices.len()
        );
        voice_cnt = MAX_SUPPORTED_VOICES;
    } else {
        voice_cnt = voices.len();
    }

    info!(
        "Complete musical piece contains {} musical elements. {} voices.",
        complete_music.len(),
        voice_cnt,
    );

    let mut music_encoder = MusicEncoder::new(BufWriter::new(outfile));
    // Encode the musical composition into binary format
    for element in complete_music {
        if dump_output {
            trace!("{:?}", element);
        }
        match element {
            MusicElement::MeasureInit(m) => {
                music_encoder.insert_measure_initializer(m)?;
            }
            MusicElement::MeasureMeta(m) => {
                music_encoder.insert_measure_metadata(m)?;
            }
            MusicElement::NoteRest(n) => {
                music_encoder.insert_note_data(n)?;
            }
            MusicElement::Tuplet(t) => {
                music_encoder.insert_tuplet_data(t)?;
            }
        }
    }
    music_encoder.flush()?;
    Ok(())
}
