use mysql::prelude::*;
use mysql::*;

use std::result::Result;
use std::result::Result::Ok;
use std::string::String;

use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};

extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate rocket;

#[derive(Serialize, Deserialize, Clone)]
struct APIResponse {
    status: APIStatus,
    result: String,
}

#[derive(Serialize, Deserialize, Clone)]
enum APIStatus {
    #[serde(rename = "success")]
    Success,
    #[serde(rename = "error")]
    Error,
}

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

static DB_URL: &str = "mysql://root:@localhost:3306/iclip";

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
fn get_clip(code: String) -> Json<APIResponse> {
    let result = get_db_clip(code);
    match result {
        Ok(result) => {
            match result {
                Some(result) => {
                    let response = APIResponse {
                        status: APIStatus::Success,
                        result
                    };

                    return Json(response);
                }
                None => {
                    let response = APIResponse {
                        status: APIStatus::Error,
                        result: "clip not found".to_string()
                    };

                    return Json(response);
                }
            };
        }
        Err(e) => {
            println!("Error: {}", e);
            let response = APIResponse {
                status: APIStatus::Error,
                result: "A problem with the database has occurred".to_string(),
            };
            return Json(response);
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
