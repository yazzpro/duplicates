extern crate crypto;

use std::fs;
use std::path::PathBuf;


#[derive(Debug)]
struct FileInfo {
    full_path: String,
    size: usize,
    hash: String
}

fn main() {
    
    let args : Vec<String> = std::env::args().collect();
    let a1 = &args[1];

    let srcdir = PathBuf::from(&a1);
    let full_path = fs::canonicalize(&srcdir).expect("File could not be processed");


    let f = FileInfo {
        full_path : full_path.to_str().expect("Path could not be translated").to_string(),
        size : 1,
        hash : "hash".to_string()
    };
    println!("{:?}", f);

}
