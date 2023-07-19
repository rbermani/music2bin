#[macro_use]
extern crate failure;
mod bin_to_xml;
mod decoder;
mod encoder;
mod layout;
mod music_xml_types;
mod notation;
mod utils;
mod xml_ser;
mod xml_to_bin;

use crate::bin_to_xml::process_bin_to_xml;
use crate::xml_to_bin::process_xml_to_bin;
use failure::Error;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, Clone, PartialEq, StructOpt)]
#[structopt(name = "mode")]
enum Mode {
    #[structopt(name = "xml2bin")]
    Xml2Bin,
    #[structopt(name = "bin2xml")]
    Bin2Xml,
}

#[derive(Debug, Clone, StructOpt)]
#[structopt(
    name = "music2bin",
    about = "An application for transforming MusicXML files into a condensed binary format targeted at ML training."
)]

struct CliOpts {
    #[structopt(default_value = "single_measure.musicxml", parse(from_os_str))]
    input: PathBuf,
    #[structopt(default_value = "music.bin", parse(from_os_str))]
    output: PathBuf,
    #[structopt(subcommand)]
    mode: Option<Mode>,
}

fn main() -> Result<(), Error> {
    let cli_opt = CliOpts::from_args();

    match cli_opt.mode {
        Some(Mode::Bin2Xml) => {
            process_bin_to_xml(cli_opt.input, cli_opt.output)?;
        }
        _ => {
            // default case for either no subcommand or Xml2Bin is perform process_xml_to_bin
            process_xml_to_bin(cli_opt.input, cli_opt.output)?;
        }
    }
    Ok(())
}
