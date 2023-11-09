use crate::bin_format::bin_to_ir;
use crate::error::Result;
use crate::ir::ir_to_xml::ir_to_xml;
use crate::ir::PartMap;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::PathBuf;

pub fn process_bin_to_xml(input: PathBuf, output: PathBuf, dump_input: bool) -> Result<()> {
    let mut outfile = File::create(output).expect("IO Error occurred on file create()");
    let infile = File::open(input).expect("IO Error occurred on file open()");
    let reader = BufReader::new(infile);

    let mut partmap = PartMap::new();
    // The MusicBin format only supports a single piano part
    let part = bin_to_ir(reader, dump_input)?;
    partmap
        .push_part("P1", part)
        .expect("Failed to push part to part map");
    let output = ir_to_xml(partmap);
    outfile
        .write_all(output.as_bytes())
        .expect("IO Error occurred on write_all()");
    Ok(())
}
