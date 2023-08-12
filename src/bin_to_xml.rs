use crate::bin_decoder::MusicDecoder;
use crate::function;
use crate::notation::*;
use crate::xml_ser::serialize_xml;
use failure::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

pub fn process_bin_to_xml(input: PathBuf, _output: PathBuf) -> Result<(), Error> {
    let infile = File::open(input)?;
    let reader = BufReader::new(infile);
    let mut music_decoder = MusicDecoder::new(Some(reader));
    music_decoder.reader_read()?;
    let parsed = music_decoder.parse_data()?;

    println!("veclen: {} {:?}", parsed.len(), parsed);
    serialize_xml(parsed);

    Ok(())
}
