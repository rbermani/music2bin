use super::muxml_parser::{
    does_note_contain_unpitched, parse_backup_tag, parse_direction_tag, parse_note_tag,
};
use crate::error::Result;
use crate::ir::measure_checker::MeasureChecker;
use crate::ir::notation::{
    BeatType, Beats, Ending, KeySignature, MeasureInitializer, MeasureMetaData, MeasureStartEnd,
    Tempo, TimeModification, TupletActual, TupletNormal,
};
use crate::ir::{MusicElement, MusicalPart, PartMap};

use log::info;
use roxmltree::*;
use std::collections::BTreeSet;
use std::str::FromStr;

use muxml::muxml_types::TimeModificationElement;

fn convert_time_modification(t_mod: &TimeModificationElement) -> TimeModification {
    let tup_ac = TupletActual::try_from(t_mod.actual_notes.as_ref())
        .expect("Cannot convert this TupletActual string.");
    let tup_norm = TupletNormal::try_from(t_mod.normal_notes.as_ref())
        .expect("Cannot convert this TupletNormal string.");

    TimeModification::new(tup_ac, tup_norm)
}

impl From<TimeModificationElement> for TimeModification {
    fn from(time_mod_elem: TimeModificationElement) -> Self {
        convert_time_modification(&time_mod_elem)
    }
}

impl From<&TimeModificationElement> for TimeModification {
    fn from(time_mod_elem: &TimeModificationElement) -> Self {
        convert_time_modification(time_mod_elem)
    }
}

pub fn xml_to_ir(docstring: String, _dump_input: bool) -> Result<PartMap> {
    let opt = ParsingOptions {
        allow_dtd: true,
        ..ParsingOptions::default()
    };
    let mut part_map = PartMap::new();

    let document =
        Document::parse_with_options(docstring.as_str(), opt).expect("Doc failed to parse.");
    let score_parts = document
        .root_element()
        .descendants()
        .filter(|n| n.has_tag_name("score-part"));

    for score_part in score_parts {
        part_map
            .add_part_id(score_part.attribute("id").unwrap())
            .expect("PartMap is not empty");
    }

    info!(
        "Preprocessing check found {} possible parts",
        part_map.num_part_ids()
    );

    let keys: Vec<String> = part_map.keys();
    let mut remove_cur_part = false;
    for part_str in keys {
        let part_tag = document
            .root_element()
            .descendants()
            .find(|n| n.has_tag_name("part") && n.attribute("id").unwrap().eq(part_str.as_str()));

        let mut musical_part: MusicalPart = MusicalPart::new();

        let measures = part_tag
            .unwrap()
            .children()
            .filter(|n| n.has_tag_name("measure"));

        let mut measure_init_present: bool = false;
        let mut quarter_division = 0;
        let mut prev_measure_init = MeasureInitializer::default();
        let mut voices = BTreeSet::new();
        for (measure_idx, measure) in measures.enumerate() {
            // if dump_input {
            //     debug!("Measure_idx {measure_idx} start");
            // }
            //let mut inserted_note_tally = 0;
            let mut measure_init = prev_measure_init;
            let mut measure_meta_start = MeasureMetaData::new(MeasureStartEnd::MeasureStart);
            let mut measure_meta_end = MeasureMetaData::new(MeasureStartEnd::MeasureEnd);

            // Each individual part duplicates the divisions entry at measure idx 0 (usually, but not always measure number 1)
            if measure_idx == 0 {
                if let Some(div) = measure.descendants().find(|n| n.has_tag_name("divisions")) {
                    quarter_division = div.text().unwrap().parse::<u32>().unwrap();
                } else {
                    panic!("No divisions tag found.");
                }
            }
            musical_part.set_divisions(quarter_division);

            // TODO: All of this XML parsing logic should be abstracted away another data type with methods
            // that can be re-used across xml2bin and xml multipart
            if let Some(t) = measure.descendants().find(|n| n.has_tag_name("time")) {
                let beats_node = t.children().find(|n| n.has_tag_name("beats")).unwrap();
                let beat_type_node = t.children().find(|n| n.has_tag_name("beat-type")).unwrap();

                measure_init.beats = Beats::from_str(beats_node.text().unwrap()).unwrap();
                measure_init.beat_type =
                    BeatType::from_str(beat_type_node.text().unwrap()).unwrap();
            };

            if let Some(n) = measure.descendants().find(|n| n.has_tag_name("repeat")) {
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
            };

            let barlines = measure.descendants().filter(|n| n.has_tag_name("barline"));
            for barline in barlines {
                if let Some(n) = barline.descendants().find(|n| n.has_tag_name("ending")) {
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

            if !measure_init_present {
                musical_part.push(MusicElement::MeasureInit(measure_init));
                measure_init_present = true;
            } else if measure_init != prev_measure_init {
                // If the current measure initializer state tracker deviates from the ongoing tracked value, insert a new measure initializer into the composition
                // to track the changed values.
                musical_part.push(MusicElement::MeasureInit(measure_init));
            }

            // Look ahead for forward tags first, to offset backup tags, because the intermediate representation
            // does not have a concept of forward and backward, and needs to insert rests as placeholders
            let mut forward_duration = 0;
            if let Some(forward_tag) = measure.children().find(|n| n.has_tag_name("forward")) {
                let duration_tag = forward_tag.first_element_child().unwrap().text().unwrap();
                forward_duration = duration_tag.parse::<usize>().unwrap();
            }
            let measure_tags = measure.children().filter(|n| {
                n.has_tag_name("note") || n.has_tag_name("direction") || n.has_tag_name("backup")
            });

            prev_measure_init = measure_init;
            let mut dynamic_value = None;
            let mut measure_checker = MeasureChecker::new(
                quarter_division,
                &measure_init,
                part_str.as_str(),
                measure_idx,
                forward_duration,
            );
            //println!("Forward duration is {}", forward_duration);
            musical_part.push(MusicElement::MeasureMeta(measure_meta_start));

            for measure_element in measure_tags {
                if measure_element.tag_name().name() == "note" {
                    // If a measure contains percussive (unpitched) content,
                    // throw this entire part away because we do not analyze drum content
                    if !does_note_contain_unpitched(&measure_element) {
                        parse_note_tag(
                            &measure_element,
                            &mut measure_checker,
                            &mut dynamic_value,
                            &mut voices,
                        );
                    } else {
                        remove_cur_part = true;
                        break;
                    }
                } else if measure_element.tag_name().name() == "direction" {
                    dynamic_value = parse_direction_tag(&measure_element);
                } else if measure_element.tag_name().name() == "backup" {
                    parse_backup_tag(&measure_element, &mut measure_checker);
                }
            }
            if !remove_cur_part {
                measure_checker.remove_incomplete_voices(&voices);
                musical_part.append(measure_checker.as_inner());
                musical_part.push(MusicElement::MeasureMeta(measure_meta_end));
            } else {
                break;
            }
        } // Process next measure in part
        let voice_cnt = if voices.len() > MeasureChecker::MAX_SUPPORTED_VOICES {
            info!(
                "Maximum supported voices is {} but piece contains {}. Threw away additional voices for part {}",
                MeasureChecker::MAX_SUPPORTED_VOICES,
                voices.len(),
                part_str,
            );
            MeasureChecker::MAX_SUPPORTED_VOICES
        } else {
            voices.len()
        };
        musical_part.set_num_voices(voice_cnt);
        if !remove_cur_part {
            part_map
                .push_part(part_str.as_str(), musical_part)
                .expect("Failed t push musical part to part map");
        } else {
            println!("Remove part {}", part_str);
            part_map.remove_part(part_str.as_str());
            remove_cur_part = false;
        }
        // info!(
        // "Musical part contains {} musical elements. {} voices.",
        // musical_part.len(),
        // voice_cnt,
        // );
    } // Process next part or loop completed

    // At this point, any vec_idx that is still None in the parts list can be discarded from the BTreeMap
    let parts_removed = part_map.get_removed_parts();
    println!("Processing step removed {} parts", parts_removed);
    Ok(part_map)
}
