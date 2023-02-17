use rusqlite::Connection;
use std::fs;
use std::io::ErrorKind;

const TEST_DATABASE: &str = "local.sqlite";

fn main() -> rusqlite::Result<()> {
    // Rerun this build script if the database or schema change. Or in other words, for each run.
    println!("cargo:rerun-if-changed={}", TEST_DATABASE);
    println!("cargo:rerun-if-changed=schema.sql");

    // Remove old database
    match fs::remove_file(TEST_DATABASE) {
        Ok(_) => {}
        Err(e) if e.kind() == ErrorKind::NotFound => {}
        Err(e) => panic!("Unable to remove test database: {}", e),
    }

    // Build new database based on the sql schema
    let connection = Connection::open(TEST_DATABASE)?;
    connection.execute_batch(include_str!("schema.sql"))
}
