use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::ErrorKind;
use uuid::Uuid;

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
    connection.execute_batch(include_str!("schema.sql"))?;

    // Add test users for convenience
    add_test_user(&connection, "test@test.me", "12345678")?;
    add_test_user(&connection, "a@b.c", "123")?;

    Ok(())
}

fn password_hash(password: &str) -> [u8; 32] {
    const SALT: [u8; 8] = [242, 94, 145, 122, 201, 1, 131, 203];

    let mut hasher = Sha256::new();
    hasher.update(SALT);
    hasher.update(password);

    hasher.finalize().into()
}

fn add_test_user(conn: &Connection, email: &str, password: &str) -> rusqlite::Result<()> {
    conn.execute(
        "INSERT INTO Site_users (sid, email, password_hash) VALUES (?, ?, ?)",
        params![Uuid::new_v4().as_bytes(), email, password_hash(password)],
    )?;

    Ok(())
}
