use std::{fs::File, io::BufWriter};

use crate::bin_format;
use crate::bin_format::MusicEncoder;
use crate::error::Result;
use crate::ir::{MusicElement, MusicalPart};
use log::debug;

pub fn ir_to_bin(
    writer: BufWriter<File>,
    complete_part: &MusicalPart,
    dump_input: bool,
) -> Result<()> {
    let mut music_encoder = MusicEncoder::new(writer);
    // Encode the musical composition into binary format
    music_encoder.create_header(complete_part.len() * bin_format::MUSIC_ELEMENT_LENGTH)?;
    for element in complete_part.inner() {
        if dump_input {
            debug!("{:?}", element);
        }
        match *element {
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
