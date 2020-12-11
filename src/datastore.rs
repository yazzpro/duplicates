use rusqlite::{params, Connection, Result};
use rusqlite::NO_PARAMS;
use std::convert::TryInto;

#[derive(Debug)]
pub struct FileInfo {
    pub full_path: String,
    pub size: u64,
    pub hash: String,
    pub last_modified: u64
}

pub struct DataStore {}

impl DataStore {
    pub fn new() -> DataStore {
        DataStore{}
    }
}
#[cfg_attr(test,mockall::automock)]
pub trait DataManager {
    fn create_tables(&self) -> Result<()>;
    fn get_entries_by_hash(&self,hash: &str) -> Result<Vec<FileInfo>>;
    fn get_entry_for_path(&self,path: &str) -> Result<Option<FileInfo>>;
    fn delete_entry_for_path(&self,path: &str) -> Result<()>;
    fn add_entry(&self,entry: &FileInfo) -> Result<()>;    
}

static DBFILENAME : &'static str = "filehashes.db";
impl DataManager for DataStore {
     fn create_tables(&self) -> Result<()> {
    let connection = Connection::open(DBFILENAME)?;
    //https://rust-lang-nursery.github.io/rust-cookbook/database/sqlite.html
    connection.execute(
        "CREATE TABLE IF NOT EXISTS file_hashes (
             id INTEGER PRIMARY KEY,
             path TEXT NOT NULL UNIQUE,
             hash TEXT NOT NULL,
             file_size INTEGER,
             last_modified INTEGER
         )",
        NO_PARAMS,
    )?;

    Ok(())
}

    fn get_entries_by_hash(&self,hash: &str) -> Result<Vec<FileInfo>> {
        let connection = Connection::open(DBFILENAME)?;

        let sql = r#"SELECT path, hash, file_size, last_modified
                    FROM file_hashes
                    WHERE hash=?"#;
        let mut stmt = connection.prepare(sql)?;
        let mut entries = stmt.query_map(&[hash], 
            |row| 
                
            Ok(FileInfo { 
                full_path : row.get(0)?, 
                hash: row.get(1)?,
                size : row.get::<usize,i64>(2)?.try_into().unwrap() ,
                last_modified : row.get::<usize,i64>(3)?.try_into().unwrap() ,
            })).unwrap();
                
        let mut list: Vec<FileInfo> = Vec::new();
            while let Some(result) = entries.next() {
                if let Some(entry) = result.ok() {
                    list.push(entry);
                }
            }            
            Ok(list)                                      
    }

    fn get_entry_for_path(&self,path: &str) -> Result<Option<FileInfo>> {
        let connection = Connection::open(DBFILENAME)?;

        let sql = r#"SELECT path, hash, file_size, last_modified
                    FROM file_hashes
                    WHERE path=?"#;
        let mut stmt = connection.prepare(sql)?;
        let mut entries = stmt.query_map(&[path], 
            |row| {           
            Ok(FileInfo { 
                full_path : row.get(0)?, 
                hash: row.get(1)?,
                size : row.get::<usize,i64>(2)?.try_into().unwrap() ,
                last_modified : row.get::<usize,i64>(3)?.try_into().unwrap() 
            })}).unwrap();
    
        if let Some(result) = entries.next() {
            if let Some(entry) = result.ok() {
                return Ok(Some(entry));
            }
        }      
        Ok(None)                                                
    }

    fn delete_entry_for_path(&self,path: &str) -> Result<()> {
        let connection = Connection::open(DBFILENAME)?;

        let sql = r#"DELETE
                    FROM file_hashes
                    WHERE path=?"#;
        connection.execute(sql, &[path])?;
        Ok(())                                                
    }

    fn add_entry(&self,entry: &FileInfo) -> Result<()> {
        let connection = Connection::open(DBFILENAME)?;
        let size_sql :i64 = entry.size.try_into().unwrap();
        let modified: i64 = entry.last_modified.try_into().unwrap();
        connection.execute(
            "INSERT INTO file_hashes (path, hash, file_size, last_modified) values (?1,?2,?3,?4)",
            params![&entry.full_path, &entry.hash, &size_sql, &modified]
        )?;

        Ok(())
    }
}
