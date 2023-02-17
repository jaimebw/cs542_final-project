use dotenvy::{dotenv, Error};
use log::{error, info};
use std::env;
use std::env::VarError;
use std::ffi::OsStr;
use std::process::exit;

/// Explicitly load .env file so we can log any issues
pub fn setup_dotenv() {
    match dotenv() {
        Ok(path) => info!(
            "Successfully loaded .env file to environment: {}",
            path.display()
        ),
        Err(Error::Io(err)) => error!("Failed to read .env due to IO error: {}", err),
        Err(Error::LineParse(line, index)) => {
            let padding = " ".repeat(index);
            error!("Failed to parse line in .env:\n{}\n{}^", line, padding)
        }
        Err(err) => panic!("Error parsing .env: {}", err),
    }
}

/// Require an environment variable be present. If not present then log the issue and exit.
pub fn var<K: AsRef<OsStr>>(key: K) -> String {
    let err = match env::var(&key) {
        Ok(v) => return v,
        Err(e) => e,
    };

    let key_lossy = key.as_ref().to_string_lossy();
    match err {
        VarError::NotPresent => error!("Unable to find environment variable {}", key_lossy),
        VarError::NotUnicode(_) => {
            error!("Environment variable {} is not valid unicode", key_lossy)
        }
    }

    exit(1)
}
