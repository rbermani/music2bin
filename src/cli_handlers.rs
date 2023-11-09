use crate::bin_format::{bin_to_ir, ir_to_bin};
use crate::error::{Error, Result};
use crate::ir::ir_to_xml::ir_to_xml;
use crate::ir::{xml_to_ir, PartMap};
use crate::repl_funcs::{add, append, hello, prepend, Context};
use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Write};
use std::path::PathBuf;

use repl_rs::Result as ReplResult;
use repl_rs::{crate_description, crate_name, crate_version};
use repl_rs::{initialize_repl, Repl};
use repl_rs::{Command, Parameter};

pub fn process_bin_to_xml(input: &PathBuf, output: &PathBuf, dump_input: bool) -> Result<()> {
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

pub fn process_xml_to_bin(input: &PathBuf, output: &PathBuf, dump_input: bool) -> Result<()> {
    let outfile = File::create(output).expect("IO Error Occurred");
    let docstring = fs::read_to_string(input).unwrap();
    let writer = BufWriter::new(outfile);

    // xml to bin only writes the first part, because MuBin only supports a single part
    let partmap = xml_to_ir(docstring, dump_input)?;
    let part = partmap.get_part(0).unwrap();
    ir_to_bin(writer, part, dump_input)?;
    Ok(())
}

pub fn process_xml_multi(input: &PathBuf, output: &PathBuf, dump_input: bool) -> Result<()> {
    let outfile = File::create(output).expect("IO Error Occurred");
    let mut writer = BufWriter::new(outfile);

    let docstring = fs::read_to_string(input).unwrap();
    let partmap = xml_to_ir(docstring, dump_input)?;

    let output_xml = ir_to_xml(partmap);
    writer
        .write_all(output_xml.as_bytes())
        .expect("IO Error occurred on write_all()");
    writer
        .flush()
        .map_err(|e| Error::IoKind(e.kind().to_string()))?;

    Ok(())
}

pub fn process_end_to_end(input: &PathBuf, output: &PathBuf, dump_input: bool) -> Result<()> {
    let tmp_path = PathBuf::from("tmp.bin");

    process_xml_to_bin(input, &tmp_path, dump_input)?;
    process_bin_to_xml(&tmp_path, output, dump_input)?;

    Ok(())
}

pub fn repl_shell() -> ReplResult<()> {
    let mut repl = initialize_repl!(Context::default())
        .use_completion(true)
        .add_command(
            Command::new("append", append)
                .with_parameter(Parameter::new("name").set_required(true)?)?
                .with_help("Append name to end of list"),
        )
        .add_command(
            Command::new("prepend", prepend)
                .with_parameter(Parameter::new("name").set_required(true)?)?
                .with_help("Prepend name to front of list"),
        )
        .add_command(
            Command::new("add", add)
                .with_parameter(Parameter::new("first").set_required(true)?)?
                .with_parameter(Parameter::new("second").set_required(true)?)?
                .with_help("Add two numbers together"),
        )
        .add_command(
            Command::new("hello", hello)
                .with_parameter(Parameter::new("who").set_required(true)?)?
                .with_help("Greetings!"),
        );
    repl.run()
}
