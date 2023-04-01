use chrono::{Duration, Local};

use mysql::prelude::*;
use mysql::*;

use std::result::Result;
use std::result::Result::Ok;
use std::string::String;

use super::redis::{get_cached_clip, cache_clip};

extern crate rand;
extern crate serde;
extern crate serde_json;

static DB_URL: &str = "mysql://root:@localhost:3306/iclip";

pub fn get_db_clip(code: String) -> Result<Option<String>, mysql::Error> {
    let pool = Pool::new(DB_URL)?;
    let conn = pool.get_conn();

    let mut conn = match conn {
        Ok(conn) => conn,
        Err(e) => {
            error!("{}", e);
            return Err(e);
        }
    };

    let query = format!("SELECT url FROM userurl WHERE usr = '{}'", code);
    let result = conn.query_first(query);

    match get_cached_clip(&code) {
        Ok(url) => {
            if let Some(url) = url {
                return Ok(Some(url));
            }
        }
        Err(e) => {
            error!("Redis Error: {}", e);
        }
    }

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
    let pool = Pool::new(DB_URL)?;
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
    let pool = Pool::new(DB_URL)?;
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
        Ok(_) => {}
        Err(e) => {
            error!("Redis Error: {}", e);
        }
    }

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}