use super::muxml_parser::{
    does_note_contain_unpitched, parse_backup_tag, parse_direction_tag, parse_note_tag,
};
use crate::error::{Result,Error};
use crate::ir::notation::{
    BeatType, Beats, Ending, KeySignature, MeasureInitializer, MeasureMetaData, MeasureStartEnd,
    Tempo
};
use crate::ir::{MusicalPart, PartMap};

use log::info;
use roxmltree::*;
use std::str::FromStr;

const MAX_SUPPORTED_PARTS: usize = 4;

pub fn multipartxml_to_ir(docstring: String, _dump_input: bool, input_filename: &str) -> Result<PartMap> {
    let opt = ParsingOptions {
        allow_dtd: true,
        ..ParsingOptions::default()
    };
    let mut ir_part_map = PartMap::new();

    let xml_document =
        Document::parse_with_options(docstring.as_str(), opt).expect("Doc failed to parse.");
    let xml_score_parts = xml_document
        .root_element()
        .descendants()
        .filter(|n| n.has_tag_name("score-part"));

    for xml_score_part in xml_score_parts.clone() {
        let part_id = xml_score_part.attribute("id").unwrap();
        ir_part_map
            .add_part_id(part_id)
            .expect("PartMap is not empty");
    }

    info!(
        "Preprocessing check found {} possible parts",
        ir_part_map.num_part_ids()
    );
    if ir_part_map.num_part_ids() > MAX_SUPPORTED_PARTS {
        println!("The number of parts {} exceeds the supported amount {}", ir_part_map.num_part_ids(), MAX_SUPPORTED_PARTS);
        return Err(Error::Unit);
    }
    if ir_part_map.num_part_ids() == MAX_SUPPORTED_PARTS {
        println!("File name {}", input_filename);
        for score_part in xml_score_parts {
            match score_part.descendants().find(|n| n.has_tag_name("part-name")).unwrap().text() {
                Some(t) => println!("Name {}", t),
                None => (),
            }
        }
    }
    let ir_parts: Vec<String> = ir_part_map.keys();
    let mut remove_cur_part = false;
    let mut total_voices: usize = 0;
    for ir_part_str in ir_parts {
        let xml_part_tag = xml_document
            .root_element()
            .descendants()
            .find(|n| n.has_tag_name("part") && n.attribute("id").unwrap().eq(ir_part_str.as_str()));

        let mut ir_musical_part: MusicalPart = MusicalPart::new(ir_part_str.as_str());

        let xml_measures = xml_part_tag
            .unwrap()
            .children()
            .filter(|n| n.has_tag_name("measure"));

        for (xml_measure_idx, xml_measure) in xml_measures.enumerate() {
            // if dump_input {
            //     debug!("Measure_idx {measure_idx} start");
            // }
            //let mut inserted_note_tally = 0;
            let mut ir_measure_init = ir_musical_part.get_cur_init_measure();
            let mut ir_measure_meta_start = MeasureMetaData::new(MeasureStartEnd::MeasureStart);
            let mut ir_measure_meta_end = MeasureMetaData::new(MeasureStartEnd::MeasureEnd);

            // Each individual part duplicates the divisions entry at measure idx 0 (usually, but not always measure number 1)
            let mut quarter_division = 0;
            if xml_measure_idx == 0 {
                if let Some(div) = xml_measure.descendants().find(|n| n.has_tag_name("divisions")) {
                    quarter_division = div.text().unwrap().parse::<u32>().unwrap();
                } else {
                    panic!("No divisions tag found.");
                }
            }
            ir_musical_part.set_initial_divisions(quarter_division);

            // TODO: All of this XML parsing logic should be abstracted away another data type with methods
            // that can be re-used across xml2bin and xml multipart
            if let Some(xml_time_tag) = xml_measure.descendants().find(|n| n.has_tag_name("time")) {
                let xml_beats_tag = xml_time_tag.children().find(|n| n.has_tag_name("beats")).unwrap();
                let xml_beat_type_tag = xml_time_tag.children().find(|n| n.has_tag_name("beat-type")).unwrap();

                ir_measure_init.beats = Beats::from_str(xml_beats_tag.text().unwrap()).unwrap();
                ir_measure_init.beat_type =
                    BeatType::from_str(xml_beat_type_tag.text().unwrap()).unwrap();
            };

            if let Some(xml_repeat_tag) = xml_measure.descendants().find(|n| n.has_tag_name("repeat")) {
                let measure_direction_str = xml_repeat_tag.attribute("direction").unwrap();
                if measure_direction_str.eq("backward") {
                    ir_measure_meta_end.start_end = MeasureStartEnd::RepeatEnd;
                } else if measure_direction_str.eq("forward") {
                    ir_measure_meta_start.start_end = MeasureStartEnd::RepeatStart;
                } else {
                    // Unsupported direction attribute
                    panic!(
                        "Encountered unsupported repeat direction attribute: {}",
                        measure_direction_str
                    );
                }
            };

            let xml_barlines = xml_measure.descendants().filter(|n| n.has_tag_name("barline"));
            for xml_barline in xml_barlines {
                if let Some(xml_ending_tag) = xml_barline.descendants().find(|n| n.has_tag_name("ending")) {
                    let ending_type_str = xml_ending_tag.attribute("type").unwrap();
                    let ending_number_str = xml_ending_tag.attribute("number").unwrap();
                    match ending_type_str {
                        "start" => {
                            ir_measure_meta_start.ending = Ending::from_str(ending_number_str)
                                .expect("Invalid Ending string.");
                        }
                        "stop" => {
                            // Used for first endings with a "downward jog"
                            ir_measure_meta_end.ending = Ending::from_str(ending_number_str)
                                .expect("Invalid Ending string.");
                        }
                        "discontinue" => {
                            // Used for second endings with no "downward jog"
                            ir_measure_meta_end.ending = Ending::from_str(ending_number_str)
                                .expect("Invalid Ending string.");
                        }
                        t => {
                            panic!("Encountered unsupported measure ending type {}", t);
                        }
                    }
                };
            }

            if let Some(ir_key_sig) = match xml_measure.descendants().find(|n| n.has_tag_name("fifths")) {
                Some(xml_fifths_tag) => KeySignature::from_str(xml_fifths_tag.text().unwrap()).ok(),
                None => None,
            } {
                ir_measure_init.key_sig = ir_key_sig;
            }

            if let Some(xml_tempo) = match xml_measure
                .descendants()
                .find(|n| n.has_tag_name("sound") && n.attribute("tempo").is_some())
            {
                Some(n) => Tempo::from_str(n.attribute("tempo").unwrap()).ok(),
                None => None,
            } {
                ir_measure_init.tempo = xml_tempo;
            }

            if ir_musical_part.get_cur_init_measure_idx().is_none() {
                ir_musical_part.push_init_measure(ir_measure_init);
            } else if ir_measure_init != ir_musical_part.get_cur_init_measure() {
                // If the current measure initializer state tracker deviates from the ongoing tracked value, insert a new measure initializer into the composition
                // to track the changed values.
                ir_musical_part.push_init_measure(ir_measure_init);
            }

            // Look ahead for forward tags first, to offset backup tags, because the intermediate representation
            // does not have a concept of forward and backward, and needs to insert rests as placeholders
            let mut forward_duration = 0;
            if let Some(forward_tag) = xml_measure.children().find(|n| n.has_tag_name("forward")) {
                let duration_tag = forward_tag.first_element_child().unwrap().text().unwrap();
                forward_duration = duration_tag.parse::<usize>().unwrap();
            }

            ir_musical_part.push_meta_start(ir_measure_meta_start, forward_duration, xml_measure_idx);

            let xml_measure_elements = xml_measure.children().filter(|n| {
                n.has_tag_name("note") || n.has_tag_name("direction") || n.has_tag_name("backup")
            });
            for xml_measure_element in xml_measure_elements {
                if xml_measure_element.tag_name().name() == "note" {
                    // If a measure contains percussive (unpitched) content,
                    // throw this entire part away because we do not analyze drum content
                    if !does_note_contain_unpitched(&xml_measure_element) {
                        parse_note_tag(
                            &xml_measure_element,
                            &mut ir_musical_part,
                        );
                    } else {
                        remove_cur_part = true;
                        break;
                    }
                } else if xml_measure_element.tag_name().name() == "direction" {
                    parse_direction_tag(&xml_measure_element, &mut ir_musical_part);
                } else if xml_measure_element.tag_name().name() == "backup" {
                    parse_backup_tag(&xml_measure_element, &mut ir_musical_part);
                }
            }
            if !remove_cur_part {
                ir_musical_part.push_meta_end(ir_measure_meta_end);
            } else {
                break;
            }
        } // Process next measure in part
        // let voice_cnt = if voices.len() > MeasureChecker::MAX_SUPPORTED_VOICES {
        //     info!(
        //         "Maximum supported voices is {} but piece contains {}. Threw away additional voices for part {}",
        //         MeasureChecker::MAX_SUPPORTED_VOICES,
        //         voices.len(),
        //         ir_part_str,
        //     );
        //     MeasureChecker::MAX_SUPPORTED_VOICES
        // } else {
        //     voices.len()
        // };
        // ir_musical_part.set_num_voices(voice_cnt);
        total_voices += ir_musical_part.get_num_voices();
        if !remove_cur_part {
            ir_part_map
                .push_part(ir_part_str.as_str(), ir_musical_part)
                .expect("Failed t push musical part to part map");
        } else {
            println!("Remove part {}", ir_part_str);
            ir_part_map.remove_part(ir_part_str.as_str());
            remove_cur_part = false;
        }
        // info!(
        // "Musical part contains {} musical elements. {} voices.",
        // musical_part.len(),
        // voice_cnt,
        // );
    } // Process next part or loop completed
    if ir_part_map.num_part_ids() == MAX_SUPPORTED_PARTS {
        println!("Total voices is {}", total_voices);
    }
    // At this point, any vec_idx that is still None in the parts list can be discarded from the BTreeMap
    let parts_removed = ir_part_map.get_removed_parts();
    println!("Processing step removed {} parts", parts_removed);

    // Combine parts into one part
    // if total_voice == 4 && ir_part_map.num_parts() == 4 {

    // }
    Ok(ir_part_map)
}
