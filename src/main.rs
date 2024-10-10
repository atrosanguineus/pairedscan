use std::{path::PathBuf, process::exit};
use clap::Parser;
use support::{get_raw_filelist, parse_filelist};

pub mod argparser;
pub mod support;
use argparser::ArgParser;

fn exit_with(code: i32, reason: &str)
{
    eprintln!("{}", reason);
    exit(code);
}

fn main() 
{
    let argparser: ArgParser = ArgParser::parse();
    if !argparser.name.is_dir() {
        exit_with(-1, "Path provided is not a directory");
    }

    let search_depth: usize = match argparser.recursive {
        true => 100,
        false => 1
    };

    let suffixes: Vec<&str> = match argparser.gzipped {
        true => vec![".fq.gz", ".fastq.gz"],
        false => vec![".fq", ".fastq"]
    };

    let raw_filelist: Vec<PathBuf> = get_raw_filelist(&argparser.name, search_depth, &suffixes);
    let parsed_filelist: String = parse_filelist(&raw_filelist, &argparser);
    println!("{}", parsed_filelist);
}
