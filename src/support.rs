use std::{collections::HashSet, path::{Path, PathBuf}, str::FromStr, vec};
use anyhow::{anyhow, Result, Context};
use regex::Regex;
use walkdir::WalkDir;

use crate::argparser::ArgParser;


// Is this folder or file hidden in the filesystem?
fn _is_hidden(entry: &walkdir::DirEntry) -> bool {

    entry.file_name()
         .to_str()
         .map(|s| s.starts_with("."))
         .unwrap_or(false)
}

// Does the current file have a suffix compatible with the suffixes list provided?
fn has_suffix(entry: &PathBuf, suffixes: &[&str]) -> bool {
    
    let entry_str: &str = entry.file_name().expect("Error in filename-to-string conversion, bailing out").to_str().expect("Error in entry-to-string conversion, bailing out");
    return suffixes.iter().any(|suff|
        entry_str.ends_with(suff)
    )    
}


fn infer_prefixes(argref: &ArgParser) -> Result<(Regex, Regex)> {

    match &argref.prefix_paired 
    {
        // In case we've specified a general prefix..
        Some(x) => {

            // Since we've specified a general prefix, don't accept specific ones
            if argref.prefix_1.is_some() || argref.prefix_2.is_some() {
                return Err(anyhow!("You can't set a common prefix while setting separate prefixes too!"));
            }

            let common_prefix_fullstr: String = format!("{}{}{}", r"(.*?)", x, r"(.*?)");
            // Otherwise, build the two identifiers from it
            let pref_from_general: Regex = Regex::from_str(&common_prefix_fullstr)
                .context("Failed to infer regex from common prefix")?;
            
            return Ok((pref_from_general.clone(), pref_from_general));

        },

        // If we've not specified a general prefix..
        None => {

            // If both specific ones are specified, use them
            if argref.prefix_1.is_some() && argref.prefix_2.is_some() {

                let prefix_1_fullstr: String = format!("{}{}{}", r"(.*?)", argref.prefix_1.as_ref().unwrap(), r"(.*?)");
                let prefix_2_fullstr: String = format!("{}{}{}", r"(.*?)", argref.prefix_2.as_ref().unwrap(), r"(.*?)");
                let pref_1: Regex = Regex::from_str(&prefix_1_fullstr)
                    .context("Failed to infer a regex from prexfix 1")?;
                let pref_2: Regex = Regex::from_str(&prefix_2_fullstr)
                    .context("Failed to infer regex from prefix 2")?;
                return Ok((pref_1, pref_2));
            }

            // If we've specified only one, refuse it
            else if (argref.prefix_1.is_none() && argref.prefix_2.is_some()) || (argref.prefix_1.is_some() && argref.prefix_2.is_none()) {
                return Err(anyhow!("Please set both prefixes if you set one!"));
            }
            
            // If nothing has been specified, proceed with the standard R1 and R2
            let pref_1: Regex = Regex::from_str(r"(.*?)R1(.*)")?;
            let pref_2: Regex = Regex::from_str(r"(.*?)R2(.*)")?;
            return Ok((pref_1, pref_2));
        }
    };
}


// Using the WalkDir crate to, recursively or not, scan the current directory
// and return its content lexicographically sorted.
// Outputs a vector of PathBufs, to be used in parsing later
pub fn get_raw_filelist(root: &Path, depth: usize, suffixes: &[&str]) -> Result<Vec<PathBuf>> {

    let mut out_vec: Vec<PathBuf> = Vec::new();
    let dir_walker = WalkDir::new(root).max_depth(depth).contents_first(true)
                                                                .follow_links(true).sort_by_file_name().into_iter();
                                                                //.filter_entry(|e| !is_hidden(e));
    for entry in dir_walker {
        let curr_entry: PathBuf = entry.context("Error in entry-to-path conversion while getting the raw filelist")?.path().to_path_buf();
        if curr_entry.is_dir() || !has_suffix(&curr_entry, suffixes){
            continue;
        }
        out_vec.push(curr_entry);
    }

    println!("{:?}", out_vec);
    //std::process::exit(-1);

    return Ok(out_vec);
}


fn strip_prefix_with_regex<'a>(filename: &'a str, regex: &Regex) -> Option<String> {
    if let Some(captures) = regex.captures(filename) {
        // Capture groups 1 and 2 represent the parts before and after the prefix
        Some(format!("{}{}", &captures[1], &captures[2]))
    } else {
        None // Return None if the filename doesn't match the pattern
    }
}


pub fn parse_filelist(filelist: &Vec<PathBuf>, argref: &ArgParser) -> Result<Vec<String>> {

    let mut vec_r1: Vec<PathBuf> = Vec::new();
    let mut vec_r2: Vec<PathBuf> = Vec::new();

    let prefixes: (Regex, Regex) = infer_prefixes(argref)?;
    let r1_pref: Regex = prefixes.0;
    let r2_pref: Regex = prefixes.1;

    // First things first: we get all the files and sort them in R1-containing ones and R2-containing ones
    for file in filelist {

        let sane_file: PathBuf = if argref.absolute {
            file.canonicalize().expect("Error in canonicalising path: what the hell?")
        } else {
            file.to_path_buf()
        };

        let temp_filename_str: &str = sane_file.file_name().unwrap().to_str().context("Error in filelist entry-to-str conversion, bailing out")?;
        let contains_r1: bool = r1_pref.is_match(temp_filename_str);
        let contains_r2: bool = r2_pref.is_match(temp_filename_str);

        match (contains_r1, contains_r2) {
            (true, true) => {
                vec_r1.push(sane_file.clone());
                vec_r2.push(sane_file);
            },
            (true, false) => {
                vec_r1.push(sane_file);
            },
            (false, true) => {
                vec_r2.push(sane_file);
            },
            (false, false) => {
                return Err(anyhow!(format!("A file does NOT contain either an \"{}\" or \"{}\" identifier. This file is: \"{}\"", r1_pref, r2_pref, temp_filename_str)))
            }
        }
    }

    // Then we check: are there the same number of R1s to R2s?
    // We don't check filenames yet, just the raw R1s number to R2s number
    if vec_r1.len() != vec_r2.len() {
        if vec_r1.len() > vec_r2.len(){
            return Err(anyhow!(format!("Unpaired number of \"{}\" and \"{}\" identifiers! There are more \"{}\" than \"{}\"", r1_pref, r2_pref, r1_pref, r2_pref)));
        }
        return Err(anyhow!(format!("Unpaired number of \"{}\" and \"{}\" identifiers! There are more \"{}\" than \"{}\"", r1_pref, r2_pref, r2_pref, r1_pref)));
    }

    // Strip prefixes and collect base filenames into sets
    let r1_bases: HashSet<_> = vec_r1.iter()
        .filter_map(|f| strip_prefix_with_regex(f.file_name()?.to_str()?, &r1_pref))
        .collect();
    let r2_bases: HashSet<_> = vec_r2.iter()
        .filter_map(|f| strip_prefix_with_regex(f.file_name()?.to_str()?, &r2_pref))
        .collect();


    // Find missing files in each set
    let missing_in_r2: HashSet<_> = r1_bases.difference(&r2_bases).collect();
    let missing_in_r1: HashSet<_> = r2_bases.difference(&r1_bases).collect();

    if !missing_in_r2.is_empty() {
        eprintln!("Missing in R2: {:?}", missing_in_r2);
    }
    if !missing_in_r1.is_empty() {
        eprintln!("Missing in R1: {:?}", missing_in_r1);
    }
    if !missing_in_r1.is_empty() || !missing_in_r2.is_empty() {
        return Err(anyhow!(""))
    }

    
    // Finally, we format the output based on interleaving argument
    let mut final_vec: Vec<PathBuf> = vec![];
    match argref.interleave
    {
        true => {
            for i in 0..vec_r1.len() {
                final_vec.push(vec_r1[i].to_path_buf());
                final_vec.push(vec_r2[i].to_path_buf());
            }
        },
        false => {
            for r1 in vec_r1 {
                final_vec.push(r1);
            }
            for r2 in vec_r2 {
                final_vec.push(r2);
            }
        }
    }

    // We convert each PathBuf to a String, then we separate by newline
    let final_string_vec: Vec<String> = final_vec.iter().map(|path| path.to_str().unwrap().to_string()).collect();
    return Ok(final_string_vec);
}
