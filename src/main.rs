use std::path::PathBuf;
use anyhow::{anyhow, bail, Context, Result};
use clap::Parser;
use support::{get_raw_filelist, make_samplesheet, parse_filelist};

pub mod argparser;
pub mod support;
use argparser::ArgParser;

fn main() -> Result<()>
{
    let argparser: ArgParser = ArgParser::parse();
    if !argparser.name.is_dir() {
        return Err(anyhow!("Path provided is not a directory"));
    }

    let search_depth: usize = match argparser.recursive {
        true => usize::max_value(),
        false => 1
    };

    let suffixes: Vec<&str> = match argparser.gzipped {
        true => vec![".fq.gz", ".fastq.gz"],
        false => vec![".fq", ".fastq"]
    };

    let raw_filelist: Vec<PathBuf> = get_raw_filelist(&argparser.name, search_depth, &suffixes)?;
    let parsed_filelist: Vec<String> = parse_filelist(&raw_filelist, &argparser)?;

    match &argparser.out_samplesheet {
        Some(samplesheet_path) => {
            if !&argparser.interleave {
                bail!("Can't write a samplesheet if you don't interleave R1s and R2s")
            }
            make_samplesheet(&samplesheet_path, &parsed_filelist)
                .context("Failed to create an output samplesheet")?
        },
        None => {
            for file in parsed_filelist {
                println!("{}", file);
            }
        },
    }


    Ok(())
}
