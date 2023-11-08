use crate::bin_encoder;
use crate::bin_encoder::MusicEncoder;
use crate::measure_checker::MeasureChecker;
use crate::error::Result;
use crate::notation::*;
use crate::xml_elem_handlers::{handle_backup_tag,handle_direction_tag,handle_note_tag};

use log::{debug, info};
use roxmltree::*;
use std::collections::BTreeSet;
use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::str::FromStr;

pub fn process_xml_to_bin(input: PathBuf, output: PathBuf, dump_input: bool) -> Result<()> {
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
        if dump_input {
            //debug!("Measure_idx {measure_idx} start");
        }
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

        if let Some(t) = measure.descendants().find(|n| n.has_tag_name("time")) {
            let beats_node = t.children().find(|n| n.has_tag_name("beats")).unwrap();
            let beat_type_node = t.children().find(|n| n.has_tag_name("beat-type")).unwrap();

            measure_init.beats = Beats::from_str(beats_node.text().unwrap()).unwrap();
            measure_init.beat_type = BeatType::from_str(beat_type_node.text().unwrap()).unwrap();
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
                        measure_meta_start.ending =
                            Ending::from_str(ending_number_str).expect("Invalid Ending string.");
                    }
                    "stop" => {
                        // Used for first endings with a "downward jog"
                        measure_meta_end.ending =
                            Ending::from_str(ending_number_str).expect("Invalid Ending string.");
                    }
                    "discontinue" => {
                        // Used for second endings with no "downward jog"
                        measure_meta_end.ending =
                            Ending::from_str(ending_number_str).expect("Invalid Ending string.");
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
            }
        }
        measure_checker.remove_incomplete_voices(&voices);
        complete_music.append(measure_checker.as_inner());
        complete_music.push(MusicElement::MeasureMeta(measure_meta_end));
    }

    let voice_cnt = if voices.len() > MeasureChecker::MAX_SUPPORTED_VOICES {
        info!(
            "Maximum supported voices is {} but piece contains {}. Threw away additional voices",
            MeasureChecker::MAX_SUPPORTED_VOICES,
            voices.len(),
        );
        MeasureChecker::MAX_SUPPORTED_VOICES
    } else {
        voices.len()
    };

    info!(
        "Complete musical piece contains {} musical elements. {} voices.",
        complete_music.len(),
        voice_cnt,
    );

    let mut music_encoder = MusicEncoder::new(BufWriter::new(outfile));
    // Encode the musical composition into binary format
    music_encoder.create_header(complete_music.len() * bin_encoder::MUSIC_ELEMENT_LENGTH)?;
    for element in complete_music {
        if dump_input {
            debug!("{:?}", element);
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
