use chrono::{Duration, Local};
use rusqlite::{params, Connection, Error, Result};

use std::{env, fmt};

use super::redis::{cache_clip, get_cached_clip};

pub enum DatabaseError {
    SqliteError(rusqlite::Error),
    CustomError(String),
}

impl From<rusqlite::Error> for DatabaseError {
    fn from(err: rusqlite::Error) -> DatabaseError {
        DatabaseError::SqliteError(err)
    }
}

impl fmt::Display for DatabaseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DatabaseError::SqliteError(err) => write!(f, "SQLite error: {}", err),
            DatabaseError::CustomError(err) => write!(f, "Database error: {}", err),
        }
    }
}

/// Load the path to the database file
fn load_url() -> String {
    env::var("DB_PATH").unwrap_or("db.sqlite".to_string())
}

/// Tries to connect to the database and if it doesn't exist, it creates it from the current schema
pub fn initialize() -> Result<()> {
    let conn = Connection::open(load_url())?;

    let schema = "
        CREATE TABLE IF NOT EXISTS clips (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            code TEXT NOT NULL,
            url TEXT NOT NULL,
            date DATETIME NOT NULL,
            expires DATE NOT NULL
        );
    ";

    conn.execute(schema, [])?;

    Ok(())
}

pub fn get_clip(code: String) -> Result<Option<String>, DatabaseError> {
    let db_url = load_url();

    let conn = Connection::open(db_url)?;

    match get_cached_clip(&code) {
        Ok(url) => {
            if let Some(url) = url {
                log::info!("Using cached clip for {}", code);
                return Ok(Some(url));
            }
        }
        Err(e) => {
            log::error!("Redis Error: {}", e);
        }
    }

    let query = "SELECT url FROM clips WHERE code = ?1";

    match conn.query_row(query, params![code], |row| row.get(0)) {
        Ok(username) => Ok(Some(username)),
        Err(Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(DatabaseError::CustomError(format!("Database error: {}", e))),
    }
}

pub fn get_clip_by_url(url: String) -> Result<Option<String>, DatabaseError> {
    let db_url = load_url();

    let conn = Connection::open(db_url)?;
    let query = "SELECT code FROM clips WHERE url = ?1";

    match conn.query_row(query, params![url], |row| row.get(0)) {
        Ok(username) => Ok(Some(username)),
        Err(Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(DatabaseError::CustomError(format!("Database error: {}", e))),
    }
}

pub fn insert_clip(code: String, url: String) -> Result<(), rusqlite::Error> {
    let db_url = load_url();

    let conn = Connection::open(db_url)?;

    let start_date = Local::now().naive_local();
    let expires = start_date + Duration::days(30);
    let expiry_date = expires.format("%Y-%m-%d").to_string();

    let query = "INSERT INTO clips (code, url, date, expires) VALUES (?1, ?2, datetime('now'), ?3)";
    conn.execute(query, params![code, url, expiry_date])?;

    match cache_clip(&code, &url) {
        Ok(_) => {
            log::info!("Cached clip for {}", code);
        }
        Err(e) => {
            log::error!("Redis Error: {}", e);
        }
    }

    Ok(())
}
