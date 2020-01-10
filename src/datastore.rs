use rusqlite::{params, Connection, Result};
use rusqlite::NO_PARAMS;

static DBFILENAME : &'static str = "filehashes.db";

fn create_tables() -> Result<()> {
    let connection = Connection::open(DBFILENAME)?;
    //https://rust-lang-nursery.github.io/rust-cookbook/database/sqlite.html
    connection.execute(
        "CREATE TABLE IF NOT EXISTS file_hashes (
             id INTEGER PRIMARY KEY,
             path TEXT NOT NULL UNIQUE,
             hash TEXT NOT NULL,
             filesize INTEGER
         )",
        NO_PARAMS,
    )?;

    Ok(())
}

