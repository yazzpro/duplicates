use rusqlite::{params, Connection, Result};
use rusqlite::NO_PARAMS;
use std::convert::TryInto;
#[derive(Debug)]
pub struct FileInfo {
    pub full_path: String,
    pub size: u64,
    pub hash: String
}

static DBFILENAME : &'static str = "filehashes.db";

pub fn create_tables() -> Result<()> {
    let connection = Connection::open(DBFILENAME)?;
    //https://rust-lang-nursery.github.io/rust-cookbook/database/sqlite.html
    connection.execute(
        "CREATE TABLE IF NOT EXISTS file_hashes (
             id INTEGER PRIMARY KEY,
             path TEXT NOT NULL UNIQUE,
             hash TEXT NOT NULL,
             file_size INTEGER
         )",
        NO_PARAMS,
    )?;

    Ok(())
}

pub fn get_entries_by_hash(hash: &str) -> Result<Vec<FileInfo>> {
    let connection = Connection::open(DBFILENAME)?;

    let sql = r#"SELECT path, hash, file_size
                 FROM file_hashes
                 WHERE hash=?"#;
    let mut stmt = connection.prepare(sql)?;
    let mut entries = stmt.query_map(&[hash], 
        |row| 
            
        Ok(FileInfo { 
            full_path : row.get(0)?, 
            hash: row.get(1)?,
            size : row.get::<usize,i64>(2)?.try_into().unwrap() 
        })).unwrap();
            
    let mut list: Vec<FileInfo> = Vec::new();
        while let Some(result) = entries.next() {
            if let Some(entry) = result.ok() {
                list.push(entry);
            }
        }            
        Ok(list)                                      
}

pub fn get_entry_for_path(path: &str) -> Result<Option<FileInfo>> {
    let connection = Connection::open(DBFILENAME)?;

    let sql = r#"SELECT path, hash, file_size
                 FROM file_hashes
                 WHERE path=?"#;
    let mut stmt = connection.prepare(sql)?;
    let mut entries = stmt.query_map(&[path], 
        |row| 
            
        Ok(FileInfo { 
            full_path : row.get(0)?, 
            hash: row.get(1)?,
            size : row.get::<usize,i64>(2)?.try_into().unwrap() 
        })).unwrap();
   
    if let Some(result) = entries.next() {
        if let Some(entry) = result.ok() {
            return Ok(Some(entry));
        }
    }      
    Ok(None)                                                
}

pub fn add_entry(entry: &FileInfo) -> Result<()> {
    let connection = Connection::open(DBFILENAME)?;
    let size_sql :i64 = entry.size.try_into().unwrap();
    connection.execute(
        "INSERT INTO file_hashes (path, hash, file_size) values (?1,?2,?3)",
        params![&entry.full_path, &entry.hash, &size_sql]
    )?;

    Ok(())
}

