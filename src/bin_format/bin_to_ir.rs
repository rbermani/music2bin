use super::bin_decoder::MusicDecoder;
use crate::error::{Error, Result};
use crate::ir::measure_checker::calc_divisions_voices;
use crate::ir::MusicalPart;
use log::{debug, error};
use std::fs::File;
use std::io::BufReader;

const MAX_SUPPORTED_VOICES: usize = 4;

pub fn bin_to_ir(reader: BufReader<File>, dump_input: bool) -> Result<MusicalPart> {
    let mut music_decoder = MusicDecoder::new(Some(reader));
    music_decoder.reader_read()?;

    let parsed_elems = music_decoder.parse_data()?;

    // For tuplets, the associated note type is embedded in the NoteData type. The Tuplet data information element
    // precedes the note data element, so to determine the shortest value represented in the piece, both the tuplet information
    // is needed and all of the notes within the tuplet section. For the minimum, we're looking for the shortest note type
    // that is within a tuplet, and the most actual notes within the number of normal notes indicated in the Tuplet data
    // and finding a LCM (least common multiple) for them
    let (divisions, voice_len) = calc_divisions_voices(parsed_elems.clone(), dump_input);

    if voice_len > MAX_SUPPORTED_VOICES {
        error!(
            "Maximum supported voices is {MAX_SUPPORTED_VOICES} but piece contains {}.",
            voice_len
        );
        return Err(Error::OutofBounds);
    }

    debug!("Divisions is {divisions}. Voices is {}", voice_len);
    //let music_elems = music_decoder.parse_data()?;
    let part = MusicalPart::new_from_elems(divisions, voice_len, parsed_elems);
    Ok(part)
}
