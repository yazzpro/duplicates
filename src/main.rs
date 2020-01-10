use crypto::sha2::Sha512;
use crypto::digest::Digest;
use walkdir::WalkDir;

use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::io::{BufReader, Read};

mod datastore;

use datastore::*;

#[derive(Debug)]
struct FileInfo {
    full_path: String,
    size: u64,
    hash: String
}
fn get_file_info(path: &str) -> Option<FileInfo> {
    let srcdir = PathBuf::from(&path);
    let full_path = fs::canonicalize(&srcdir).expect("File could not be processed");
    let file = File::open(&full_path).expect("can't upen file");
    let meta = file.metadata().unwrap();
    if meta.is_dir() {
        return None;
    }
    let file_length = meta.len();
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
    
    let digest = hasher.result_str();    

    Some(FileInfo {
        full_path : full_path.to_str().expect("Path could not be translated").to_string(),
        size : file_length,
        hash : digest
    })
}
fn process_path(path: &str) {
    
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if !entry.file_type().is_dir() {
            println!("{:?}",  get_file_info(entry.path().to_str().unwrap()));
        }
        
    }
}
fn main() {
    
    process_path(".");

}
