use crate::encoder::MusicEncoder;
use crate::notation::*;
use failure::Error;
use roxmltree::*;
use std::fs;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;
use std::str::FromStr;

pub fn process_xml_to_bin(input: PathBuf, output: PathBuf) -> Result<(), Error> {
    let outfile = File::create(output)?;

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

    let mut measure_init_present: bool = false;

    for measure in measures {
        let mut measure_init = MeasureInitializer::default();
        let mut measure_meta_start = MeasureMetaData::default();
        let mut measure_meta_end = MeasureMetaData::default();
        measure_meta_end.start_end = MeasureStartEnd::MeasureEnd;
        let notes = measure.descendants().filter(|n| n.has_tag_name("note"));

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
                    measure_meta_end.repeat = Repeats::Repeat;
                } else if measure_direction_str.eq("forward") {
                    measure_meta_start.repeat = Repeats::Repeat;
                } else {
                    // Unsupported direction attribute
                    println!(
                        "Encountered unsupported repeat direction attribute: {}",
                        measure_direction_str
                    );
                }
            }
            _ => (),
        };
        let keysig: Option<KeySignature> =
            match measure.descendants().find(|n| n.has_tag_name("fifths")) {
                Some(n) => KeySignature::from_str(n.text().unwrap()).ok(),
                None => None,
            };
        println!("{:?}", keysig);

        let tempo: Option<Tempo> = match measure.descendants().find(|n| n.has_tag_name("sound")) {
            Some(n) => Tempo::from_str(n.attribute("tempo").unwrap()).ok(),
            None => None,
        };
        println!("Tempo is {:?}", tempo);
        let dynamics_tags: Vec<Node> = measure
            .descendants()
            .filter(|n| n.has_tag_name("dynamics"))
            .collect();
        // Need to check if the dynamics elements were found, if two were found, ensure they are children of a direction-type element.
        // One of the siblings of the direction-type must be a staff element, which will identify which staff the dynamics apply to.
        // If no staff element is present, assume a single stave piece with G clef as default for now.
        let mut single_stave_piece = false;
        let mut dynamics_val_first = None;
        let mut dynamics_val_second = None;
        if dynamics_tags.len() == 1 {
            // Either there is a single stave piece with one dynamics tag or a two stave piece with only one dynamic tag
            // Get the direction tag two levels above
            let first_staff_value = match dynamics_tags[0]
                .parent()
                .unwrap()
                .parent()
                .expect("Direction Tag not found")
                .children()
                .find(|n| n.has_tag_name("staff"))
            {
                Some(n) => n.text(),
                None => {
                    single_stave_piece = true;
                    None
                }
            };
            println!("staff val {:?}", first_staff_value);
            dynamics_val_first = Some(dynamics_tags[0].first_element_child().unwrap());
            dynamics_val_second = None;
        } else if dynamics_tags.len() == 2 {
            let _first_staff_value = match dynamics_tags[0]
                .parent()
                .unwrap()
                .parent()
                .expect("Direction Tag not found")
                .children()
                .find(|n| n.has_tag_name("staff"))
            {
                Some(n) => n.text(),
                None => {
                    panic!("Two dynamics annotations found without two staff elements.");
                }
            };
            let _second_staff_value = match dynamics_tags[1]
                .parent()
                .unwrap()
                .parent()
                .expect("Direction Tag not found")
                .children()
                .find(|n| n.has_tag_name("staff"))
            {
                Some(n) => n.text(),
                None => {
                    panic!("Two dynamics annotations found without two staff elements.");
                }
            };
            dynamics_val_first = Some(dynamics_tags[0].first_element_child().unwrap());
            dynamics_val_second = Some(dynamics_tags[1].first_element_child().unwrap());
        } else if dynamics_tags.len() > 2 {
            panic!("Two measure dynamics tags per measure is unsupported.");
        }

        println!("{:?} {:?}", dynamics_val_first, dynamics_val_second);

        measure_init.treble_dynamics = match dynamics_val_first {
            Some(t) => {
                Dynamics::from_str(t.tag_name().name()).expect("Unsupported dynamic type found.")
            }
            None => Dynamics::default(),
        };
        measure_init.bass_dynamics = match dynamics_val_second {
            Some(t) => {
                Dynamics::from_str(t.tag_name().name()).expect("Unsupported dynamic type found.")
            }
            None => Dynamics::default(),
        };

        if measure_init_present == false {
            complete_music.push(MusicElement::MeasureInit(measure_init));
            measure_init_present = true;
        } else if measure_init != MeasureInitializer::default() {
            // If the current measure initializer state tracker deviates from the default, insert a new measure initializer into the composition
            // to track the changed values.
            complete_music.push(MusicElement::MeasureInit(measure_init));
        }
        complete_music.push(MusicElement::MeasureMeta(measure_meta_start));
        let mut slur_engaged = false;
        for note in notes {
            let mut note_data = NoteData::default();
            let note_type = note.children().find(|n| n.has_tag_name("type")).unwrap();
            let notations_tag = note.children().find(|n| n.has_tag_name("notations"));
            let rest_tag = note.children().find(|n| n.has_tag_name("rest"));
            let staff_text = note
                .children()
                .find(|n| n.has_tag_name("staff"))
                .unwrap()
                .text()
                .unwrap();
            match staff_text {
                "1" => {
                    note_data.rh_lh = RightHandLeftHand::RightHand;
                }
                "2" => {
                    note_data.rh_lh = RightHandLeftHand::LeftHand;
                }
                _ => {
                    panic!("Unhandled staff tag value case");
                }
            }

            match notations_tag {
                Some(n) => {
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
                        Some(t) => Arpeggiate::Arpeggiate,
                        None => Arpeggiate::NoArpeggiation,
                    };
                    let artic = match artic_tag {
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
                    match slur_tag {
                        Some(t) => match t.attribute("type").unwrap() {
                            "start" => {
                                slur_engaged = true;
                            }
                            "stop" => {
                                slur_engaged = false;
                            }
                            _ => {
                                panic!("Unhandled slur tag attribute case");
                            }
                        },
                        None => (),
                    }

                    if slur_engaged {
                        // Legato overrides other articulations, except accents, which are handled independently
                        note_data.articulation = Articulation::Legato;
                    } else {
                        note_data.articulation = artic;
                    }
                }
                None => (),
            }
            note_data.rhythm_value = Duration::from_str(note_type.text().unwrap()).unwrap();

            match rest_tag {
                Some(_) => {
                    println!("rest {:?}", note_data.rhythm_value);
                    note_data.note_rest = 0;
                }
                None => {
                    let pitch_tag = note.children().find(|n| n.has_tag_name("pitch")).unwrap();
                    let step_tag = pitch_tag.children().find(|n| n.has_tag_name("step"));
                    let octave_tag = pitch_tag.children().find(|n| n.has_tag_name("octave"));
                    let alter_tag = pitch_tag.children().find(|n| n.has_tag_name("alter"));

                    // alter tags are optional, others are mandatory
                    let alter_note = match alter_tag {
                        Some(t) => Alter::from_str(t.text().unwrap()).unwrap(),
                        None => Alter::None,
                    };
                    note_data.note_rest = NoteData::encode_numeric_note(
                        Note::from_str(step_tag.unwrap().text().unwrap()).unwrap(),
                        alter_note,
                        Octave::from_str(octave_tag.unwrap().text().unwrap()).unwrap(),
                    )
                    .expect("Parsed note is not supported by ImgMusic format.");
                    println!(
                        "note {:?} number: {}",
                        note_data.rhythm_value, note_data.note_rest
                    );
                }
            }

            complete_music.push(MusicElement::NoteRest((note_data)));
        }
        complete_music.push(MusicElement::MeasureMeta(measure_meta_end));
    }
    println!(
        "Complete musical piece contains {} musical elements",
        complete_music.len()
    );
    let mut music_encoder = MusicEncoder::new(BufWriter::new(outfile));
    // Encode the musical composition into binary format
    for element in complete_music {
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
        }
    }
    music_encoder.flush()?;

    Ok(())
}
