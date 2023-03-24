use mysql::prelude::*;
use mysql::*;

use std::result::Result;
use std::result::Result::Ok;
use std::string::String;

#[macro_use]
extern crate rocket;

#[get("/status")]
fn index() -> &'static str {
    "OK"
}

/*
"examples": {
    "0": {
    "value": "{\"status\":\"success\",\"result\":\"https:\\/\\/taskord.com\\/\"}"
    }
}
*/

static DB_URL: &str = "mysql://root:password@localhost:3307/db_name";

fn get_db_clip(code: String) -> Result<Option<String>, mysql::Error> {
    let pool = Pool::new(DB_URL)?;
    let conn = pool.get_conn();

    let mut conn = match conn {
        Ok(conn) => conn,
        Err(e) => {
            println!("Error: {}", e);
            return Err(e);
        }
    };

    let query = format!("SELECT * FROM userurl WHERE usr = '{}'", code);
    let result: Result<Vec<String>, mysql::Error> = conn.query_map(query, |url| (url));

    let result = match result {
        Ok(result) => result.get(0).cloned(),
        Err(e) => {
            return Err(e);
        }
    };

    Ok(result)
}

#[get("/get?<code>")]
fn get_clip(code: String) -> String {
    let result = get_db_clip(code);
    match result {
        Ok(result) => {
            let result = match result {
                Some(result) => result,
                None => {
                    return format!("get: {}", "this code does not exist");
                }
            };
            return format!("get: {}", result);
        }
        Err(e) => {
            println!("Error: {}", e);
            return format!("get: {}", "db prob");
        }
    }
}

#[get("/get")]
fn get_clip_empty() -> String {
    format!("get: {}", "no code")
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/api", routes![index, get_clip, get_clip_empty])
}
