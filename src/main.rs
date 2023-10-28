#![allow(dead_code)]
mod bin_decoder;
mod bin_encoder;
mod bin_to_xml;
mod error;
mod music_xml_types;
mod notation;
mod utils;
mod xml_to_bin;

use crate::bin_to_xml::process_bin_to_xml;
use crate::error::Error;
use crate::xml_to_bin::process_xml_to_bin;
use env_logger::Env;
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
    #[structopt(
        short = "i",
        long = "input",
        default_value = "frelise.musicxml",
        parse(from_os_str)
    )]
    input: PathBuf,
    #[structopt(
        short = "o",
        long = "output",
        default_value = "music.bin",
        parse(from_os_str)
    )]
    output: PathBuf,
    #[structopt(short = "d", long = "dump")]
    dump_input: bool,
    #[structopt(subcommand)]
    mode: Option<Mode>,
}

fn main() -> Result<(), Error> {
    env_logger::Builder::from_env(Env::default().default_filter_or("trace")).init();

    let cli_opt = CliOpts::from_args();

    match cli_opt.mode {
        Some(Mode::Bin2Xml) => {
            process_bin_to_xml(cli_opt.input, cli_opt.output, cli_opt.dump_input)?;
        }
        _ => {
            // default case for either no subcommand or Xml2Bin is perform process_xml_to_bin
            process_xml_to_bin(cli_opt.input, cli_opt.output, cli_opt.dump_input)?;
        }
    }
    Ok(())
}
