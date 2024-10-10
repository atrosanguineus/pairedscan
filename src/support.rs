use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::{argparser::ArgParser, exit_with};

// Is this folder or file hidden in the filesystem?
// Internal use, not pub
fn is_hidden(entry: &walkdir::DirEntry) -> bool
{
    entry.file_name()
         .to_str()
         .map(|s| s.starts_with("."))
         .unwrap_or(false)
}

// Does the current file have a suffix compatible with the suffixes list provided?
// Internal use, not pub
fn has_suffix(entry: &walkdir::DirEntry, suffixes: &[&str]) -> bool
{
    let entry_str: &str = entry.file_name().to_str().expect("Error in entry-to-string conversion, bailing out");
    return suffixes.iter().any(|suff|
        entry_str.ends_with(suff)
    )    
}


fn infer_prefixes(argref: &ArgParser) -> (String, String)
{
    match &argref.prefix_paired 
    {
        // In case we've specified a general prefix..
        Some(x) => 
        { 
            // Since we've specified a general prefix, don't accept specific ones
            if argref.prefix_1.is_some() || argref.prefix_2.is_some()
            {
                exit_with(-1, "You can't set a common prefix while setting separate prefixes too!")
            }

            // Otherwise, build the two identifiers from it
            let pref1_from_general: String = x.replace('X', "1");
            let pref2_from_general: String = x.replace('X', "2");
            
            return(pref1_from_general, pref2_from_general);

        },
        // If we've not specified a general prefix..
        None => 
        {
            // If both specific ones are specified, use them
            if argref.prefix_1.is_some() && argref.prefix_2.is_some()
            {
                return(argref.prefix_1.clone().unwrap(), argref.prefix_2.clone().unwrap().to_owned());
            }
            // If we've specified only one, refuse it
            else if argref.prefix_1.is_some() || argref.prefix_2.is_some()
            {
                exit_with(-1, "Please set both prefixes if you set one!");
            }
            
            // If nothing has been specified, proceed with the standard R1 and R2
            let pref1: String = String::from("R1");
            let pref2: String = String::from("R2");
            return(pref1, pref2);
        }
    };
}


// Using the WalkDir crate to, recursively or not, scan the current directory
// and return its content lexicographically sorted.
// Outputs a vector of PathBufs, to be used in parsing later
pub fn get_raw_filelist(root: &Path, depth: usize, suffixes: &[&str]) -> Vec<PathBuf>
{
    let mut out_vec: Vec<PathBuf> = vec![];
    let dir_walker = WalkDir::new(root).max_depth(depth).contents_first(true)
                                                                .follow_links(true).sort_by_file_name().into_iter()
                                                                .filter_entry(|e| !is_hidden(e) && (e.path().is_dir() || has_suffix(e, suffixes)));
    for entry in dir_walker
    {
        let curr_entry = entry.expect("Error in entry-to-path conversion, bailing out").path().to_path_buf();
        if curr_entry.is_dir() {
            continue;
        }
        out_vec.push(curr_entry);
    }

    return out_vec;
}


pub fn parse_filelist(filelist: &Vec<PathBuf>, argref: &ArgParser) -> String
{
    let mut vec_r1: Vec<PathBuf> = vec![];
    let mut vec_r2: Vec<PathBuf> = vec![];

    let prefixes: (String, String) = infer_prefixes(argref);
    let r1_pref: String = prefixes.0;
    let r2_pref: String = prefixes.1;

    // First things first: we get all the files and sort them in R1-containing ones and R2-containing ones
    for file in filelist
    {
        let str_temp: String = file.to_str().expect("Error in filelist entry-to-str conversion, bailing out").to_string();
        let contains_r1: bool = str_temp.contains(&r1_pref);
        let contains_r2: bool = str_temp.contains(&r2_pref);

        let final_file = if argref.absolute {
            file.canonicalize().expect("Error in canonicalising path: what the hell?")
        } else {
            file.to_path_buf()
        };
        

        match contains_r1 {
            true => vec_r1.push(final_file),
            false => match contains_r2 {
                true => vec_r2.push(final_file),
                false => exit_with(-1, &format!("A fastq file does NOT contain either an \"{}\" or \"{}\" identifier. This file is: \"{}\"", r1_pref, r2_pref, str_temp))
                }
            }
    }

    // Then we check: are there the same number of R1s to R2s?
    // We don't check filenames yet, just the raw R1s number to R2s number
    if vec_r1.len() != vec_r2.len() {
        if vec_r1.len() > vec_r2.len(){
            exit_with(-1, &format!("Unpaired number of \"{}\" and \"{}\" identifiers! There are more \"{}\" than \"{}\"", r1_pref, r2_pref, r1_pref, r2_pref));
        }
        exit_with(-1, &format!("Unpaired number of \"{}\" and \"{}\" identifiers! There are more \"{}\" than \"{}\"", r1_pref, r2_pref, r2_pref, r1_pref));
    }

    // Then we analyse each filename, replacing R1 and R2 identifiers with an X.
    // Fastest way to check for names in common, I presume
    for i in 0..vec_r1.len()
    {
        let curr_r1_elem_string = vec_r1[i].to_str().expect("Error in path unwrapping to string");
        let curr_r2_elem_string = vec_r2[i].to_str().expect("Error in path unwrapping to string");
        let curr_r1_elem: String = curr_r1_elem_string.replace(&r1_pref, "X");
        let curr_r2_elem: String = curr_r2_elem_string.replace(&r2_pref, "X");
        
        if curr_r1_elem != curr_r2_elem {
            exit_with(-1, &format!("Found non-matching pair: \"{}\" with \"{}\"", curr_r1_elem_string, curr_r2_elem_string))
        }
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
    let final_string: String = final_string_vec.join("\n");
    return final_string;
}