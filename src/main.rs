extern crate config;
extern crate serde;
extern crate notify;

use crypto::sha2::Sha512;
use crypto::digest::Digest;
use walkdir::WalkDir;

use notify::{Watcher, RecursiveMode,RecommendedWatcher, DebouncedEvent};

use std::sync::mpsc::channel;
use std::time::{UNIX_EPOCH, Duration};
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
       // print!("(re)calculating hash for file {}", path);
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
/// Main logic
fn process_file(path: &str,settings: &Settings) {
    match get_file_info(path) {
        Some(info) => {
            let mut file_already_added = false;
            let data_for_path = get_entry_for_path(&info.full_path).expect("I assume None but not error!");
            match data_for_path {
                Some(d) => {
                    
                    if d.hash != info.hash {
                        println!("HASH changed for file : {} ! ", info.full_path);
                        delete_entry_for_path(path).unwrap();               // current fileinfo will be added as new
                    } else {
                        file_already_added = true;
                    }
                }
                None => ()
            }
                     
            if !file_already_added {
                add_entry(&info).expect("Unable to add entry to db");
            }

            let possible_duplicates = get_duplicates_for_hash(&info.hash);
            //println!("possible duplicates: {:?}", &possible_duplicates);
            if possible_duplicates.len() >1
            {
                process_duplicates(&info, possible_duplicates, settings);    // new method for handling duplicates
            }
           
        }
        None => println!("File at path {} was not processed", path) 
    }     
}
fn get_duplicates_sorted_by_score(dups: &Vec<FileInfo>, settings: &Settings) -> Vec<String>{
    let mut scoring_items : Vec<String> = settings.delete_score.to_vec();
    scoring_items.reverse();
    let mut scores: Vec<(String, i32)> = dups.into_iter().map(
        |e| {
            let mut v = 1;
            let mut s = 0; //score
            for i in &scoring_items {
                if e.full_path.contains(i) {
                    s += v;
                }
                v += 1;
            }
            (e.full_path.clone(), s)
        } ).collect();
    
    scores.sort_by_key(|i| i.1);
    scores.reverse();

    //println!("duplicates with score: {:?}", &scores);

    let just_filenames : Vec<String>= scores.into_iter().map( |s| s.0).collect();

    just_filenames
}
fn mark_for_deletion(filenames: Vec<String>) {
    if filenames.len() <= 1 {
        return;
    }
    let mut i = 0;
    println!("Duplicates found:");
    while i < filenames.len() -1 {// -1 is crucial as we don't want to delete every occurence
        println!("DELETE: {}", &filenames[i]);
        i+= 1;
    }
    println!("LEAVE: {}" , &filenames.last().unwrap() );
}
fn delete(filenames: Vec<String>) {
    if filenames.len() <= 1 {
        return;
    }
    let mut i = 0;
    println!("Duplicates found:");
    while i < filenames.len() -1 {// -1 is crucial as we don't want to delete every occurence
        println!("DELETE: {}", &filenames[i]);
        fs::remove_file(&filenames[i]).unwrap();
        delete_entry_for_path(&filenames[i]).unwrap();
        i+= 1;
    }
    println!("LEAVE: {}" , &filenames.last().unwrap() );
}
fn process_duplicates(info: &FileInfo, dups: Vec<FileInfo>, settings: &Settings) {
    let d = get_duplicates_sorted_by_score(&dups, settings);
    match settings.action.as_str() {
        "D" => delete(d), 
        "T" => mark_for_deletion(d),
        "S" => { mark_for_deletion(d); std::process::exit(1); }
        _ => {  // default action - write about hashes
            for dup_info in dups.iter() {
                if info.full_path != dup_info.full_path {
                    if info.hash == dup_info.hash && info.size == dup_info.size {
                        println!("Hashes are the same for files : {} and {} ! ", info.full_path, dup_info.full_path);
                    }     
                }
            }      
        }
    }
    
    
}
fn notify_changes( settings: &Settings) {    
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(2)).unwrap();
    watcher.watch(&settings.working_dir, RecursiveMode::Recursive).unwrap_or_default();
    loop {
        match rx.recv() {
            Ok(event) => {
                match event {
                    DebouncedEvent::Write(p) => process_file_check_ignore(&p, settings),
                    DebouncedEvent::Create(p) => process_file_check_ignore(&p, settings), 
                    _ =>  (),//println!("{:?}", event)
                } 
                
            },
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}
fn process_file_check_ignore(path_buf: &PathBuf, settings: &Settings) {
    if !should_ignore_path(path_buf, settings) {
        let f_path = fs::canonicalize(&path_buf);
        if f_path.is_ok() {
        let full_path = f_path.unwrap();
        let s_path = full_path.to_str().unwrap();
        process_file(s_path,settings);
        }
    }
}
fn should_ignore_path(path_buf: &PathBuf, settings: &Settings) -> bool{
    match fs::canonicalize(&path_buf) {
        Ok(full_path) => {
            let s_path = String::from(full_path.to_str().unwrap());
            for s in &settings.ignore_paths {
                if s_path.contains(s) {
                    return true;
                }
            }
        },
        Err(e) => {
            println!("should ignore path err {:?}", e);
            return true; 
        }
    }
   
    false
}
fn process_path( settings: &Settings) {
    
    'filewalker: for entry in WalkDir::new(&settings.working_dir).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path().to_str().unwrap();
        let srcdir = PathBuf::from(&path);
        let full_path = fs::canonicalize(&srcdir).expect("File could not be processed");
        let s_path = String::from(full_path.to_str().unwrap());
        
        if should_ignore_path(&srcdir, settings) {
            continue 'filewalker;
        }
    
        if !entry.file_type().is_dir() {            
                process_file(&s_path, settings);               
        }
        
    }
}
fn main() -> std::result::Result<(), std::io::Error> {
    let settings = Settings::new();
    create_tables().expect("I couldn't create tables!");            
    if settings.is_ok() {  
        let u_settings = settings.unwrap();      
        process_path(&u_settings);
        notify_changes(&u_settings);
    } else
    if let Some(arg) = env::args().nth(1) {

        process_path(&Settings{ 
                  ignore_paths : vec![], 
                  working_dir : String::from(arg),
                  action: "T".to_string(), delete_score: vec![],
                });
                
        
    } else {
        println!("USAGE: duplicates PATH_TO_CHECK");        
    }    
    
    Ok(())
}
