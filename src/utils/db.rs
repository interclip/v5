use chrono::{Duration, Local};

use mysql::prelude::*;
use mysql::*;

use std::result::Result;
use std::result::Result::Ok;
use std::string::String;

use super::redis::{cache_clip, get_cached_clip};

extern crate rand;
extern crate serde;
extern crate serde_json;
extern crate dotenv;
use dotenv::dotenv;
use std::env;
use mysql::Pool;

// A custom structure to hold our DB_URL
pub struct DatabaseUrl {
    pub server: String,
    pub db_name: String,
    pub username: String,
    pub password: String,
}

/// Load environment variables and create the DB_URL
fn load_db_url() -> DatabaseUrl {
    dotenv().ok();

    let server = env::var("DB_SERVER").expect("DB_SERVER must be set");
    let db_name = env::var("DB_NAME").expect("DB_NAME must be set");
    let username = env::var("USERNAME").expect("USERNAME must be set");
    let password = env::var("PASSWORD").expect("PASSWORD must be set");

    DatabaseUrl { server, db_name, username, password }
}

pub fn get_db_clip(code: String) -> Result<Option<String>, mysql::Error> {
  let db_url = load_db_url();

  let opts = OptsBuilder::new()
      .ip_or_hostname(Some(db_url.server))
      .db_name(Some(db_url.db_name))
      .user(Some(db_url.username))
      .pass(Some(db_url.password));

  let pool = Pool::new(opts)?;
  let conn = pool.get_conn();

    let mut conn = match conn {
        Ok(conn) => conn,
        Err(e) => {
            error!("{}", e);
            return Err(e);
        }
    };

    match get_cached_clip(&code) {
        Ok(url) => {
            if let Some(url) = url {
                info!("Using cached clip for {}", code);
                return Ok(Some(url));
            }
        }
        Err(e) => {
            error!("Redis Error: {}", e);
        }
    }

    let query = format!("SELECT url FROM userurl WHERE usr = '{}'", code);
    let result = conn.query_first(query);

    let result = match result {
        Ok(result) => result,
        Err(e) => {
            error!("{}", e);
            return Err(e);
        }
    };

    Ok(result)
}

pub fn get_db_clip_by_url(url: String) -> Result<Option<String>, mysql::Error> {
  let db_url = load_db_url();

  let opts = OptsBuilder::new()
      .ip_or_hostname(Some(db_url.server))
      .db_name(Some(db_url.db_name))
      .user(Some(db_url.username))
      .pass(Some(db_url.password));

  let pool = Pool::new(opts)?;
  let conn = pool.get_conn();

    let mut conn = match conn {
        Ok(conn) => conn,
        Err(e) => {
            error!("{}", e);
            return Err(e);
        }
    };

    let query = format!("SELECT usr FROM userurl WHERE url = '{}'", url);
    let result = conn.query_first(query);

    let result = match result {
        Ok(result) => result,
        Err(e) => {
            error!("{}", e);
            return Err(e);
        }
    };

    Ok(result)
}

pub fn insert_db_clip(code: String, url: String) -> Result<(), mysql::Error> {
  let db_url = load_db_url();

  let opts = OptsBuilder::new()
      .ip_or_hostname(Some(db_url.server))
      .db_name(Some(db_url.db_name))
      .user(Some(db_url.username))
      .pass(Some(db_url.password));

  let pool = Pool::new(opts)?;
  let conn = pool.get_conn();

    let mut conn = match conn {
        Ok(conn) => conn,
        Err(e) => {
            error!("{}", e);
            return Err(e);
        }
    };

    let start_date = Local::now().naive_local();
    let expires = start_date + Duration::days(30);
    let expiry_date = expires.format("%Y-%m-%d").to_string();

    let query = format!(
        "INSERT INTO userurl (usr, url, date, expires) VALUES ('{}', '{}', NOW(), '{}')",
        code, url, expiry_date
    );
    let result = conn.query_drop(query);

    match cache_clip(&code, &url) {
        Ok(_) => {
            info!("Cached clip for {}", code);
        }
        Err(e) => {
            error!("Redis Error: {}", e);
        }
    }

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}
