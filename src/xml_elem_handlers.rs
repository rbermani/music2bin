
use crate::measure_checker::MeasureChecker;
use crate::notation::*;

use log::warn;
use num_traits::FromPrimitive;
use roxmltree::*;
use std::collections::BTreeSet;
use std::str::FromStr;
use strum::EnumCount;

const MAX_NUMBER_OF_SUPPORTED_TUPLET_ELEMENTS: usize = TupletNumber::COUNT;

pub fn handle_backup_tag(measure_element: &Node<'_, '_>, measure_checker: &mut MeasureChecker) {
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

pub fn handle_direction_tag(measure_element: &Node<'_, '_>) -> Option<PhraseDynamics> {
    //Dynamics::from_str(t.tag_name().name()).expect("Unsupported dynamic type found.")
    let dynamics_tag = measure_element
        .children()
        .find(|n| n.has_tag_name("dynamics"));

    if let Some(val) = dynamics_tag {
        match PhraseDynamics::from_str(val.first_element_child().unwrap().tag_name().name()) {
            Ok(t) => Some(t),
            Err(_) => None,
        }
    } else {
        None
    }
}

pub fn handle_note_tag(
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

    if dotted.is_some() {
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
    let voice_idx = voices.iter().position(|&x| x == voice_num).unwrap();

    if voices.len() > MeasureChecker::MAX_SUPPORTED_VOICES {
        warn!("Too many voices case, skipping notes");
        // Don't let the number of voices in the voices set exceed the maximum
        voices.remove(&voice_num);
        return;
    } else {
        note_data.voice = FromPrimitive::from_u8((voice_idx) as u8)
            .expect("Unable to FromPrimitive on u8 to voice type.");
    }

    let time_mod_value = if let Some(n) = time_mod_tag {
        let actual_notes_tag = n.children().find(|n| n.has_tag_name("actual-notes"));
        let normal_notes_tag = n.children().find(|n| n.has_tag_name("normal-notes"));
        if let (Some(an_tag), Some(nn_tag)) = (actual_notes_tag, normal_notes_tag) {
            let actual_notes = an_tag.text().unwrap().parse().unwrap();
            let normal_notes = nn_tag.text().unwrap().parse().unwrap();
            Some(TimeModification::new(actual_notes, normal_notes))
        } else {
            None
        }
    } else {
        None
    };

    if let Some(phrase_dyn) = dynamic_value {
        note_data.phrase_dynamics = *phrase_dyn;
        *dynamic_value = None;
    }
    if let Some(n) = notations_tag {
        let tuplet_tags = n.children().filter(|n| n.has_tag_name("tuplet"));
        let tied_tag = n.children().find(|n| n.has_tag_name("tied"));
        let slur_tag = n.children().find(|n| n.has_tag_name("slur"));
        let arp_tag = n.children().find(|n| n.has_tag_name("arpeggiate"));
        let artic_tag = n.children().find(|n| n.has_tag_name("articulations"));

        let num_tuplets = tuplet_tags.clone().count();
        if num_tuplets > MAX_NUMBER_OF_SUPPORTED_TUPLET_ELEMENTS {
            panic!(
                "measure_idx: {} Maximum number of supported tuplet tags {} was exceeded by {}",
                measure_checker.measure_idx(),
                MAX_NUMBER_OF_SUPPORTED_TUPLET_ELEMENTS,
                num_tuplets,
            )
        }

        note_data.ties = match tied_tag {
            Some(t) => NoteConnection::from_str(t.attribute("type").unwrap())
                .expect("Unsupported Tied Type"),
            None => NoteConnection::None,
        };

        note_data.arpeggiate = match arp_tag {
            Some(_t) => Arpeggiate::Arpeggiate,
            None => Arpeggiate::NoArpeggiation,
        };

        note_data.articulation = if let Some(t) = artic_tag {
            Articulation::from_str(t.first_element_child().unwrap().tag_name().name()).expect("Articulation::from_str method never returns Err")
        } else {
            Articulation::None
        };

        note_data.slur = match slur_tag {
            Some(t) => SlurConnection::from_str(t.attribute("type").unwrap())
                .expect("Unhandled slur tag attribute case"),
            None => SlurConnection::None,
        };

        if num_tuplets > 0 {
            if let Some(time_mod_value) = time_mod_value {
                for t in tuplet_tags {
                    match t.attribute("type").unwrap() {
                        "start" => {
                            measure_checker.push_elem(MusicElement::Tuplet(TupletData {
                                start_stop: TupletStartStop::TupletStart,
                                tuplet_number: TupletNumber::One,
                                actual_notes: time_mod_value.get_actual(),
                                normal_notes: time_mod_value.get_normal(),
                                dotted: false,
                            }));
                        }
                        "stop" => {
                            stop_tuplet_elem = Some(MusicElement::Tuplet(TupletData {
                                start_stop: TupletStartStop::TupletStop,
                                tuplet_number: TupletNumber::One,
                                actual_notes: time_mod_value.get_actual(),
                                normal_notes: time_mod_value.get_normal(),
                                dotted: false,
                            }));
                        }
                        _ => {
                            panic!("Unhandled tuplet tag attribute case");
                        }
                    }
                }
            } else {
                panic!("time mod value should always be populated if tuplets > 0 ");
            }
        }
    }

    note_data.note_type = if let Some(n) = note_type {
        NoteType::from_str(n.text().unwrap()).unwrap()
    } else {
        // Whole rests sometimes provide no "type" tag, but whole rests are different durations
        // depending on the time signature, so we must manually calculate the rhythm value based on duration
        if let Some(n) = xml_note_duration {
            if let Some((rest_duration, is_dotted, time_mod)) = NoteData::from_numeric_duration(
                n.text().unwrap().parse::<u32>().unwrap(),
                measure_checker.quarter_division(),
            ) {
                if time_mod.is_some() {
                    warn!("time modification for rest is present, but not being used.")
                }
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
    if let Some(st_elem) = stop_tuplet_elem {
        measure_checker.push_elem(st_elem);
    }
}