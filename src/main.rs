mod utils;

use rand::Rng;
use rocket::http::Status;
use rocket::response::status::Custom;

use std::result::Result;
use std::result::Result::Ok;
use std::string::String;

use rocket::serde::json::Json;
use rocket::serde::{Deserialize, Serialize};

use utils::db::{get_db_clip, get_db_clip_by_url, insert_db_clip};

extern crate rand;
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

/* Generated an alphanumeric ID (only lowercase letters), n letters long */
fn gen_id(length: usize) -> String {
    let mut code = String::new();
    let mut rng = rand::thread_rng();
    let chars: Vec<char> = "abcdefghijklmnopqrstuvwxyz0123456789".chars().collect();

    for _ in 0..length {
        let random_char = rng.gen_range(0..chars.len());
        code.push(chars[random_char]);
    }

    code
}

#[post("/set?<url>")]
fn set_clip(url: String) -> Result<Json<APIResponse>, Custom<Json<APIResponse>>> {
    // Check if the URL is valid
    let url = match url.parse::<url::Url>() {
        Ok(url) => url,
        Err(e) => {
            println!("Error: {}", e);
            let response = APIResponse {
                status: APIStatus::Error,
                result: "Invalid URL".to_string(),
            };
            return Err(Custom(Status::BadRequest, Json(response)));
        }
    };

    // Check for existence of the URL in the database
    let existing_clip = get_db_clip_by_url(url.to_string());

    match existing_clip {
        Ok(existing_clip) => {
            match existing_clip {
                Some(existing_clip) => {
                    let response = APIResponse {
                        status: APIStatus::Success,
                        result: existing_clip,
                    };

                    return Ok(Json(response));
                }
                None => {}
            };
        }
        Err(e) => {
            println!("Error: {}", e);
            let response = APIResponse {
                status: APIStatus::Error,
                result: "A problem with the database has occurred".to_string(),
            };
            return Err(Custom(Status::InternalServerError, Json(response)));
        }
    };

    let code = gen_id(5);
    let result = insert_db_clip(code.clone(), url.to_string());
    match result {
        Ok(_) => {
            let response = APIResponse {
                status: APIStatus::Success,
                result: code,
            };
            return Ok(Json(response));
        }
        Err(e) => {
            println!("Error: {}", e);
            let response = APIResponse {
                status: APIStatus::Error,
                result: "A problem with the database has occurred".to_string(),
            };
            return Err(Custom(Status::InternalServerError, Json(response)));
        }
    }
}

#[get("/get?<code>")]
fn get_clip(code: String) -> Result<Custom<Json<APIResponse>>, Custom<Json<APIResponse>>> {
    let result = get_db_clip(code);
    match result {
        Ok(result) => {
            match result {
                Some(result) => {
                    let response = APIResponse {
                        status: APIStatus::Success,
                        result,
                    };

                    return Ok(Custom(Status::Created, Json(response)));
                }
                None => {
                    let response = APIResponse {
                        status: APIStatus::Error,
                        result: "clip not found".to_string(),
                    };

                    return Err(Custom(Status::NotFound, Json(response)));
                }
            };
        }
        Err(e) => {
            println!("Error: {}", e);
            let response = APIResponse {
                status: APIStatus::Error,
                result: "A problem with the database has occurred".to_string(),
            };
            return Err(Custom(Status::InternalServerError, Json(response)));
        }
    }
}

#[get("/get")]
fn get_clip_empty() -> String {
    format!("get: {}", "no code")
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/api", routes![index, get_clip, get_clip_empty, set_clip])
}
