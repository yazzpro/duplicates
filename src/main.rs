extern crate config;
extern crate serde;

use crypto::sha2::Sha512;
use crypto::digest::Digest;
use walkdir::WalkDir;

use std::time::{SystemTime, UNIX_EPOCH};
use std::fs;
use std::fs::File;
use std::path::{PathBuf, Path};
use std::io::{BufReader, Read};
use std::env;

mod datastore;
mod settings;

use datastore::*;
use settings::Settings;

#[macro_use]
extern crate serde_derive;

fn get_file_info(path: &str) -> Option<FileInfo> {
    let srcdir = PathBuf::from(&path);
    let full_path = fs::canonicalize(&srcdir).expect("File could not be processed");
    let file = File::open(&full_path).expect("can't upen file");
    let meta = file.metadata().unwrap();
    if meta.is_dir() {
        return None;
    }
    let last_update_time = meta.modified().unwrap().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let mut hash = String::from("");
    let existing_entry = get_entry_for_path(full_path.to_str().unwrap()).unwrap();
    let should_recalculate = match existing_entry {
        None => true,
        Some(v) => {
            hash = v.hash;
            v.last_modified < last_update_time
            }
    };
    if should_recalculate { 
        print!("(re)calculating hash for file {}", path);
        hash = calculate_hash_for_file(&file) ;
    } 
    let file_length = meta.len();
   
    Some(FileInfo {
        full_path : full_path.to_str().expect("Path could not be translated").to_string(),
        size : file_length,
        hash : hash,
        last_modified : last_update_time       
    })
}
fn calculate_hash_for_file(file: &File) -> String {    
    let mut reader = BufReader::new(file);
    let mut hasher = Sha512::new();    
    let mut buffer = [0; 4096];

    loop {
        let count = reader.read(&mut buffer).expect("what kind of error can happen on reading buffer?");
        if count == 0 {
            break;
        }
        hasher.input(&buffer[0..count]);
    }    
    
    hasher.result_str()
}
fn get_duplicates_for_hash(hash:&str) -> Vec<FileInfo> {
    let entries = get_entries_by_hash(&hash).expect("get_entries failed");
    let mut result: Vec<FileInfo> = Vec::new(); 
    for entry_to_test in entries.into_iter() {
        if Path::new(&entry_to_test.full_path).exists() {
            result.push(entry_to_test);
        } else {
            // file doesn't exist anymore: let's delete its data
            delete_entry_for_path(&entry_to_test.full_path).unwrap_or_default();
        }
    }
    result
}
fn process_path( settings: Settings) {
    
    'filewalker: for entry in WalkDir::new(settings.working_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path().to_str().unwrap();
        let s_path = String::from(path);
        for s in  &settings.ignore_paths {
            if s_path.contains(s) {
                continue 'filewalker;
            }
        }
        if !entry.file_type().is_dir() {
            
            match get_file_info(path) {
                Some(info) => {
                    let mut file_already_added = false;
                    let data_for_path = get_entry_for_path(&info.full_path).expect("I assume None but not error!");
                    match data_for_path {
                        Some(d) => {
                            file_already_added = true;
                            if d.hash != info.hash {
                                println!("HASH changed for file : {} ! ", info.full_path);
                            }
                        }
                        None => ()
                    }
                    let possible_duplicates = get_duplicates_for_hash(&info.hash);
                    for dup_info in possible_duplicates.iter() {
                        if info.full_path != dup_info.full_path {
                            if info.hash == dup_info.hash && info.size == dup_info.size {
                                println!("Hashes are the same for files : {} and {} ! ", info.full_path, dup_info.full_path);
                            }     
                        }
                    }                   
                    if !file_already_added {
                        add_entry(&info).expect("Unable to add entry to db");
                    }

                    println!("file: {}", info.full_path );
                }
                None => println!("File at path {} was not processed", path)
            }            
            
        }
        
    }
}
fn main() -> Result<(), std::io::Error> {
    let settings = Settings::new();
    create_tables().expect("I couldn't create tables!");        
    if settings.is_ok() {
        process_path(settings.unwrap());
    } else
    if let Some(arg) = env::args().nth(1) {
        process_path(Settings{ 
                  ignore_paths : vec![], 
                  working_dir : String::from(arg)
                });
        
    } else {
        println!("USAGE: duplicates PATH_TO_CHECK")
    }    
    
    Ok(())
}
