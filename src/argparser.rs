use std::path::PathBuf;
use clap::Parser;

#[derive(Parser)]
#[command(name = "pairedscan")]
#[command(version = "1.0")]
#[command(about = None, long_about = None)]
pub struct ArgParser
{
    /// Root folder to scan
    pub name: PathBuf,

    /// Scan provided folder and all of its descendents
    #[arg(short, long, action)]
    pub recursive: bool,

    /// Scan for gzipped fastqs instead of plaintext ones
    #[arg(short, long = "gz", action)]
    pub gzipped: bool,

    /// Enable interleaving each R1 with their matched R2
    #[arg(short, long, action)]
    pub interleave: bool,

    /// Set a custom R1 prefix
    #[arg(short = '1', long = "p1", action)]
    pub prefix_1: Option<String>,

    /// Set a custom R2 prefix
    #[arg(short = '2', long = "p2", action)]
    pub prefix_2: Option<String>,

    /// Set a custom paired prefix
    #[arg(short = 'p', long = "pp", action)]
    pub prefix_paired: Option<String>,

    /// Return as absolute?
    #[arg(short = 'a', long = "absolute", action)]
    pub absolute: bool,
}