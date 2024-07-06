use super::bin_decoder::MusicDecoder;
use crate::error::Result;
use crate::ir::MusicalPart;
use log::debug;
use std::fs::File;
use std::io::BufReader;

pub fn bin_to_ir(reader: BufReader<File>, dump_input: bool) -> Result<MusicalPart> {
    let mut music_decoder = MusicDecoder::new(Some(reader));
    music_decoder.reader_read()?;

    let parsed_elems = music_decoder.parse_data()?;

    let part = MusicalPart::new_from_elems("P1", parsed_elems)?;
    debug!("Divisions is {}. Voices is {}", part.get_initial_divisions().unwrap(), part.get_num_voices());
    Ok(part)
}
