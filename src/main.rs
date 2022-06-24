extern crate confy;
extern crate serde;
extern crate notify;

use crypto::sha2::Sha512;
use crypto::digest::Digest;

use notify::{Watcher, RecursiveMode,RecommendedWatcher, DebouncedEvent};

use lettre::transport::smtp::authentication::Credentials;
use lettre::{Message, SmtpTransport, Transport};

use std::sync::mpsc::channel;
use std::time::{UNIX_EPOCH, Duration};
use std::fs::File;
use std::path::{PathBuf, Path};
use std::io::{BufReader, Read};
use std::env;

mod datastore;
mod settings;
mod file_manager;
mod logger;

use file_manager::*;
use datastore::*;
use settings::Settings;
use logger::*;

#[macro_use]
extern crate serde_derive;

#[cfg(test)]
mod tests;

fn get_file_info(path: &str, file_manager: &impl HandleFiles, data_manager: &impl DataManager) -> Option<FileInfo> {
    let srcdir = PathBuf::from(&path);
    let full_path = file_manager.get_full_path(&srcdir).expect("File could not be processed");
    
    let file = file_manager.get_file(&full_path).expect("can't upen file");
    let meta = file.metadata().unwrap();
    if meta.is_dir() {
        return None;
    }
    let last_update_time = meta.modified().unwrap().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let mut hash = String::from("");
    let existing_entry = data_manager.get_entry_for_path(full_path.to_str().unwrap()).unwrap();
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
fn get_duplicates_for_hash(hash:&str, data_manager: &impl DataManager) -> Vec<FileInfo> {
    let entries = data_manager.get_entries_by_hash(&hash).expect("get_entries failed");
    let mut result: Vec<FileInfo> = Vec::new(); 
    for entry_to_test in entries.into_iter() {
        if Path::new(&entry_to_test.full_path).exists() {
            result.push(entry_to_test);
        } else {
            // file doesn't exist anymore: let's delete its data
            data_manager.delete_entry_for_path(&entry_to_test.full_path).unwrap_or_default();
        }
    }
    result
}
/// Main logic
fn process_file(path: &str,settings: &Settings,file_manager: &impl HandleFiles, data_manager: &impl DataManager, log: &mut Logger) {
    match get_file_info(path, file_manager,data_manager) {
        Some(info) => {
            let mut file_already_added = false;
            let data_for_path = data_manager.get_entry_for_path(&info.full_path).expect("I assume None but not error!");
            match data_for_path {
                Some(d) => {
                    
                    if d.hash != info.hash {
                        
                        println!("HASH changed for file : {} ! ", info.full_path);
                        log.log(format!("HASH changed for file : {} ! ", info.full_path).to_string());
                        data_manager.delete_entry_for_path(path).unwrap();               // current fileinfo will be added as new
                    } else {
                        file_already_added = true;
                    }
                }
                None => ()
            }
                     
            if !file_already_added {
                data_manager.add_entry(&info).expect("Unable to add entry to db");
            }

            let possible_duplicates = get_duplicates_for_hash(&info.hash, data_manager);
            //println!("possible duplicates: {:?}", &possible_duplicates);
            if possible_duplicates.len() >1
            {
                process_duplicates(&info, possible_duplicates, settings, file_manager,data_manager, log);    // new method for handling duplicates
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
fn mark_for_deletion(filenames: Vec<String>, log: &mut Logger) {
    if filenames.len() <= 1 {
        return;
    }
    let mut i = 0;
    println!("Duplicates found:");
    log.log("Duplicate found:".to_string());
    while i < filenames.len() -1 {// -1 is crucial as we don't want to delete every occurence
        println!("DELETE: {}", &filenames[i]);
        log.log(format!("DELETE: {}", &filenames[i]).to_string());
        i+= 1;
    }
    log.log(format!("LEAVE: {}" , &filenames.last().unwrap() ).to_string());
    println!("LEAVE: {}" , &filenames.last().unwrap() );
}
fn delete(filenames: Vec<String>, file_manager: &impl HandleFiles, data_manager: &impl DataManager, log: &mut Logger) {
    if filenames.len() <= 1 {
        return;
    }
    let mut i = 0;
    let mut items : Vec<String> = vec![];
    // making copy of filenames in case the same item was passed more than once. In that case we don't want to delete it
    for i in &filenames {
        if !items.iter().any(|x| x == i) {
            items.push(i.clone());
        }
    }
    log.log("Duplicate found:".to_string());
    println!("Duplicates found:");
    while i < items.len() -1 {// -1 is crucial as we don't want to delete every occurence        
        log.log(format!("DELETE: {}", &items[i]).to_string());
        println!("DELETE: {}", &items[i]);
        file_manager.remove_file(&items[i]).unwrap();
        data_manager.delete_entry_for_path(&items[i]).unwrap();        
        i+= 1;
    }
    log.log(format!("LEAVE: {}" , &items.last().unwrap() ).to_string());
    println!("LEAVE: {}" , &items.last().unwrap() );
}
fn process_duplicates(info: &FileInfo, dups: Vec<FileInfo>, settings: &Settings, file_manager: &impl HandleFiles, data_manager: &impl DataManager, log: &mut Logger) {
    let d = get_duplicates_sorted_by_score(&dups, settings);
    match settings.action.as_str() {
        "D" => delete(d, file_manager,data_manager, log), 
        "T" => mark_for_deletion(d, log),
        "S" => { mark_for_deletion(d, log); std::process::exit(1); }
        _ => {  // default action - write about hashes
            for dup_info in dups.iter() {
                if info.full_path != dup_info.full_path {
                    if info.hash == dup_info.hash && info.size == dup_info.size {
                        println!("Hashes are the same for files : {} and {} ! ", info.full_path, dup_info.full_path);
                        log.log(format!("Hashes are the same for files : {} and {} ! ", info.full_path, dup_info.full_path).to_string());
                    }     
                }
            }      
        }
    }
    
    
}
fn notify_changes( settings: &Settings,file_manager: &impl HandleFiles, data_manager: &impl DataManager, log: &mut Logger) {    
    let (tx, rx) = channel();
    let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(2)).unwrap();
    watcher.watch(&settings.working_dir, RecursiveMode::Recursive).unwrap_or_default();
    loop {
        match rx.recv() {
            Ok(event) => {
                match event {
                    DebouncedEvent::Write(p) => process_file_check_ignore(&p, settings, file_manager,data_manager, log),
                    DebouncedEvent::Create(p) => process_file_check_ignore(&p, settings, file_manager,data_manager, log), 
                    _ =>  (),//println!("{:?}", event)
                } 
                
            },
            Err(e) => println!("watch error: {:?}", e),
        }
    }
}
fn process_file_check_ignore(path_buf: &PathBuf, settings: &Settings, file_manager: &impl HandleFiles, data_manager: &impl DataManager, log: &mut Logger) {
    if !should_ignore_path(path_buf, settings,file_manager) {
        let f_path = file_manager.get_full_path(&path_buf);
        if f_path.is_ok() {
        let full_path = f_path.unwrap();
        let s_path = full_path.to_str().unwrap();
        process_file(s_path,settings, file_manager,data_manager, log);
        }
    }
}
fn should_ignore_path(path_buf: &PathBuf, settings: &Settings, file_manager: &impl HandleFiles) -> bool{
    match file_manager.get_full_path(&path_buf) {
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
fn process_path( settings: &Settings, file_manager: &impl HandleFiles, data_manager: &impl DataManager, log: &mut Logger) {
    
    'filewalker: for entry in file_manager.walkdir(&settings.working_dir).filter_map(|e| e.ok()) {
        let path = entry.path().to_str().unwrap();
        let srcdir = PathBuf::from(&path);
        let full_path_o = file_manager.get_full_path(&srcdir);
        if full_path_o.is_ok() {
            let s_path = String::from(full_path_o.unwrap().to_str().unwrap());
        
            if should_ignore_path(&srcdir, settings, file_manager) {
                continue 'filewalker;
            }
    
            if !entry.file_type().is_dir() {            
                process_file(&s_path, settings,file_manager,data_manager, log);               
            }
    }
        
    }
}
fn main() -> std::result::Result<(), std::io::Error> {
    let settings = Settings::new();
    let file_manager = FileManager::new();
    let data_manager = DataStore::new();
    let mut log = Logger::new();
    data_manager.create_tables().expect("I couldn't create tables!");            
    if settings.is_ok() {  
        let u_settings = settings.unwrap();      
        
        process_path(&u_settings, &file_manager,&data_manager, &mut log);
        if u_settings.watchdog {
            notify_changes(&u_settings, &file_manager,&data_manager, &mut log);
        } 
        if let Some(email_address) = u_settings.email_result_to {
            let email = Message::builder()
            .from("Report <jaroslaw@majatech.pl>".parse().unwrap())
            .to(email_address.parse().unwrap())
            .subject("Duplicates report")
            .body(log.dump())
            .unwrap();

let creds = Credentials::new(u_settings.email_username.unwrap(), u_settings.email_password.unwrap());

// Open a remote connection to gmail
let mailer = SmtpTransport::starttls_relay(&u_settings.email_hostname.unwrap())
    .unwrap()
    .credentials(creds)
    .build();

// Send the email
match mailer.send(&email) {
    Ok(_) => println!("Email sent successfully!"),
    Err(e) => panic!("Could not send email: {:?}", e),
}

        }
    } else
    if let Some(arg) = env::args().nth(1) {

        process_path(&Settings{ 
                  ignore_paths : vec![], 
                  working_dir : String::from(arg),
                  action: "T".to_string(), 
                  delete_score: vec![], 
                  watchdog: false,
                  email_result_to: None,
                  email_hostname: None,
                  email_password: None,
                  email_username: None
                }, &file_manager,&data_manager, &mut log);
                
        
    } else {
        println!("USAGE: duplicates PATH_TO_CHECK");        
    }    
    
    Ok(())
}

