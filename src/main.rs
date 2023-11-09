#![allow(dead_code)]
mod bin_format;
mod cli_handlers;
mod error;
mod ir;
mod repl_funcs;
mod utils;

use crate::error::Result;

use cli_handlers::{
    process_bin_to_xml, process_end_to_end, process_xml_multi, process_xml_to_bin, repl_shell,
};
use env_logger::Env;
use log::LevelFilter;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, Clone, PartialEq, StructOpt)]
#[structopt(name = "mode")]
enum Mode {
    #[structopt(name = "xml2bin")]
    Xml2Bin,
    #[structopt(name = "bin2xml")]
    Bin2Xml,
    #[structopt(name = "xmlmulti")]
    XmlMulti,
    #[structopt(name = "e2e")]
    End2End,
    #[structopt(name = "shell")]
    Shell,
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

fn main() -> Result<()> {
    let mut builder = env_logger::Builder::from_env(Env::default());

    builder
        .filter(Some("repl_funcs"), LevelFilter::Info)
        .filter(Some("cli_handlers"), LevelFilter::Info)
        .init();

    let cli_opt = CliOpts::from_args();

    match cli_opt.mode {
        Some(Mode::End2End) => {
            process_end_to_end(&cli_opt.input, &cli_opt.output, cli_opt.dump_input)?;
        }
        Some(Mode::Bin2Xml) => {
            process_bin_to_xml(&cli_opt.input, &cli_opt.output, cli_opt.dump_input)?;
        }
        Some(Mode::XmlMulti) => {
            process_xml_multi(&cli_opt.input, &cli_opt.output, cli_opt.dump_input)?;
        }
        Some(Mode::Xml2Bin) => {
            process_xml_to_bin(&cli_opt.input, &cli_opt.output, cli_opt.dump_input)?;
        }
        Some(Mode::Shell) => {
            repl_shell()?;
        }
        None => {
            println!("No command mode provided.")
        }
    }
    Ok(())
}
