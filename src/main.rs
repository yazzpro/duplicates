use crypto::sha2::Sha512;
use crypto::digest::Digest;

use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::io::{BufReader, Read};

#[derive(Debug)]
struct FileInfo {
    full_path: String,
    size: u64,
    hash: String
}

fn main() {
    
    let args : Vec<String> = std::env::args().collect();
    let a1 = &args[1];

    let srcdir = PathBuf::from(&a1);
    let full_path = fs::canonicalize(&srcdir).expect("File could not be processed");
    let file = File::open(&full_path).expect("can't upen file");
    let file_length = file.metadata().unwrap().len();
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

    println!("digest is {:?}",&digest);

    let f = FileInfo {
        full_path : full_path.to_str().expect("Path could not be translated").to_string(),
        size : file_length,
        hash : digest
    };
    println!("{:?}", f);

}
