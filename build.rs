use rusqlite::{params, Connection};
use sha2::{Digest, Sha256};
use std::fs;
use std::io::ErrorKind;
use uuid::Uuid;
use std::path::Path;

const TEST_DATABASE: &str = "local.sqlite";

fn main() -> rusqlite::Result<()> {
    // Rerun this build script if the database or schema change. Or in other words, for each run.
    println!("cargo:rerun-if-changed={}", TEST_DATABASE);
    println!("cargo:rerun-if-changed=schema.sql");

    // Remove old database
    if <str as AsRef<Path>>::as_ref(TEST_DATABASE).exists() {
        return Ok(())
    }


    match fs::remove_file(TEST_DATABASE) {
        Ok(_) => {}
        Err(e) if e.kind() == ErrorKind::NotFound => {}
        Err(e) => panic!("Unable to remove test database: {}", e),
    }
   
    let connection = Connection::open(TEST_DATABASE)?;
    connection.execute_batch(include_str!("schema.sql"))?;

    // Add test users for convenience
    add_test_data(&connection, "test@test.me", "12345678",
                  "Cooldep1",
                  "Acme",
                  "www.lol.com","Super prod",
                  "First var","Cool type",
                  "AAAAAAAAAA","GOOD","Yesterday")?;

    // Build new database based on the sql schema
    //add_test_data(&connection, "a@b.c", "123","BBBBBBBBBB","BAD","TODAY")?;

    Ok(())
}

fn password_hash(password: &str) -> [u8; 32] {
    const SALT: [u8; 8] = [242, 94, 145, 122, 201, 1, 131, 203];

    let mut hasher = Sha256::new();
    hasher.update(SALT);
    hasher.update(password);

    hasher.finalize().into()
}

fn add_test_data(conn: &Connection, email: &str,password: &str,
                 dep_name: &str,
                 manu_name: &str,
                 url: &str, product_name:&str,
                 variation: &str, typep: &str,
                 asin:&str, conditions: &str, last_notification: &str) -> rusqlite::Result<()> {
    // Basic generation of test data for the
    // INSERT INTO Deal_Alert_on VALUES (' ', ' ', ' ');
    // INSERT INTO Subscribes_To VALUES (' ', ' ', ' ');
    let user_id = Uuid::new_v4();
    let dep_id = Uuid::new_v4();
    let manu_id = Uuid::new_v4();
    let pid_id = Uuid::new_v4();


    conn.execute(
        "INSERT INTO Site_users (sid, email, password_hash) VALUES (?, ?, ?)",
        params![user_id.as_bytes(), email, password_hash(password)],
    )?;
    conn.execute(
        "INSERT INTO Department (DepID, name) VALUES (?, ?)",
        params![dep_id.as_bytes(), dep_name],
    )?;
    conn.execute(
        "INSERT INTO Manufacturer (ManuID, name) VALUES (?, ?)",
        params![manu_id.as_bytes(), manu_name],
    )?;

    conn.execute(
        "INSERT INTO Sold_Product_Manufactured (PID,URL,name,DepID,ManuID) VALUES (?, ?, ?, ?,?)",
        params![pid_id.as_bytes(), url, product_name, dep_id.as_bytes(),manu_id.as_bytes()],
    )?;

    conn.execute(
        "INSERT INTO Product_variant_Sold (ASIN,variation,type,PID) VALUES (?, ?, ?,?)",
        params![asin, variation, typep, pid_id.as_bytes()], 
    )?;

    conn.execute(
        "INSERT INTO Deal_Alert_on (conditions, ASIN, last_notification) VALUES (?, ?, ?)",
        params![conditions,asin,last_notification]
        )?;
    conn.execute(
        "INSERT INTO Subscribes_To (conditions, ASIN, sid) VALUES (?, ?, ?)",
        params![conditions,asin,user_id.as_bytes()]
        )?;
    Ok(())
}
