mod models;
mod schema;
mod utils;

use rocket::http::Status;
use rocket::response::status::Custom;
use utils::id::gen_id;
use utils::log::setup_logger;

use std::result::Result;
use std::result::Result::Ok;
use std::string::String;

use rocket::form::Form;
use rocket::serde::json::Json;

use utils::db;

use crate::utils::rate_limit::RateLimiter;
use crate::utils::structs::{APIResponse, APIStatus};

use git2::Repository;

extern crate rand;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate rocket;
extern crate fern;
extern crate log;

#[get("/status")]
fn status(_rate_limiter: RateLimiter) -> Result<Json<APIResponse>, Custom<Json<APIResponse>>> {
    let mut db_connection = db::initialize();

    let code = "test".to_string();
    let url = "https://github.com".to_string();

    if let Err(e) = db::insert_clip(&mut db_connection, url, code.clone()) {
        error!("{}", e);
        let response = APIResponse {
            status: APIStatus::Error,
            result: "A problem with the database has occurred".to_string(),
        };
        return Err(Custom(Status::InternalServerError, Json(response)));
    }

    let result = db::get_clip(&mut db_connection, code);

    match result {
        Ok(_) => {
            let response = APIResponse {
                status: APIStatus::Success,
                result: "OK".to_string(),
            };

            Ok(Json(response))
        }
        Err(e) => {
            error!("{}", e);
            let response = APIResponse {
                status: APIStatus::Error,
                result: "A problem with the database has occurred".to_string(),
            };
            Err(Custom(Status::InternalServerError, Json(response)))
        }
    }
}

#[get("/set")]
fn set_clip_get() -> Result<Json<APIResponse>, Custom<Json<APIResponse>>> {
    Err(Custom(
        Status::MethodNotAllowed,
        Json(APIResponse {
            status: APIStatus::Error,
            result: "For creating clips, only POST is allowed".to_string(),
        }),
    ))
}

#[derive(FromForm)]
struct FormData {
    url: String,
}

#[catch(429)]
fn too_many_requests() -> Json<APIResponse> {
    Json(APIResponse {
        status: APIStatus::Error,
        result: "Too many requests".to_string(),
    })
}

#[catch(404)]
fn not_found() -> Json<APIResponse> {
    Json(APIResponse {
        status: APIStatus::Error,
        result: "Endpoint not found".to_string(),
    })
}

#[post("/set", data = "<form_data>")]
fn set_clip(
    form_data: Form<FormData>,
    _rate_limiter: RateLimiter,
) -> Result<Json<APIResponse>, Custom<Json<APIResponse>>> {
    let url = &form_data.url;
    if url.is_empty() {
        let response = APIResponse {
            status: APIStatus::Error,
            result: "No URL provided".to_string(),
        };
        return Err(Custom(Status::BadRequest, Json(response)));
    }

    let url = match url.parse::<url::Url>() {
        Ok(url) => url,
        Err(e) => {
            error!("{}", e);
            let response = APIResponse {
                status: APIStatus::Error,
                result: "Invalid URL".to_string(),
            };
            return Err(Custom(Status::BadRequest, Json(response)));
        }
    };

    let mut db_connection = db::initialize();

    // Check for existence of the URL in the database
    let existing_clip = db::get_clip_by_url(&mut db_connection, url.to_string());

    if let Ok(Some(existing_clip)) = existing_clip {
        let response = APIResponse {
            status: APIStatus::Success,
            result: existing_clip.code,
        };
        return Ok(Json(response));
    }

    if let Err(e) = existing_clip {
        error!("{}", e);
        let response = APIResponse {
            status: APIStatus::Error,
            result: "A problem with the database has occurred".to_string(),
        };
        return Err(Custom(Status::InternalServerError, Json(response)));
    }

    let code = gen_id(5);
    let result = db::insert_clip(&mut db_connection, url.to_string(), code.clone());
    match result {
        Ok(_) => {
            let response = APIResponse {
                status: APIStatus::Success,
                result: code,
            };
            Ok(Json(response))
        }
        Err(e) => {
            error!("{}", e);
            let response = APIResponse {
                status: APIStatus::Error,
                result: "A problem with the database has occurred".to_string(),
            };
            Err(Custom(Status::InternalServerError, Json(response)))
        }
    }
}

#[get("/get?<code>")]
fn get_clip(
    code: String,
    _rate_limiter: RateLimiter,
) -> Result<Custom<Json<APIResponse>>, Custom<Json<APIResponse>>> {
    if code.is_empty() {
        let response = APIResponse {
            status: APIStatus::Error,
            result: "No code provided".to_string(),
        };
        return Err(Custom(Status::BadRequest, Json(response)));
    }

    let mut db_connection = db::initialize();

    let result = db::get_clip(&mut db_connection, code);
    match result {
        Ok(Some(clip)) => {
            let response = APIResponse {
                status: APIStatus::Success,
                result: clip.url,
            };
            Ok(Custom(Status::Created, Json(response)))
        }
        Ok(None) => {
            let response = APIResponse {
                status: APIStatus::Error,
                result: "Clip not found".to_string(),
            };
            Err(Custom(Status::NotFound, Json(response)))
        }
        Err(e) => {
            error!("{}", e);
            let response = APIResponse {
                status: APIStatus::Error,
                result: "A problem with the database has occurred".to_string(),
            };
            Err(Custom(Status::InternalServerError, Json(response)))
        }
    }
}

#[get("/get")]
fn get_clip_empty() -> Result<Custom<Json<APIResponse>>, Custom<Json<APIResponse>>> {
    Err(Custom(
        Status::BadRequest,
        Json(APIResponse {
            status: APIStatus::Error,
            result: "No clip code provided in the request.".to_string(),
        }),
    ))
}

#[derive(serde::Serialize)]
struct Version {
    commit: Option<String>,
}

#[get("/version")]
fn version(_rate_limiter: RateLimiter) -> Json<Version> {
    let repo = Repository::discover(".");
    let commit = match repo {
        Ok(r) => {
            let head = r.head();
            match head {
                Ok(reference) => {
                    let peeling = reference.peel_to_commit();
                    match peeling {
                        Ok(commit) => Some(format!("{}", commit.id())),
                        Err(_) => None,
                    }
                }
                Err(_) => None,
            }
        }
        Err(_) => None,
    };

    Json(Version { commit })
}

#[launch]
fn rocket() -> _ {
    match setup_logger() {
        Ok(path) => {
            println!("Logger setup at {}", path);
        }
        Err(e) => {
            println!("Error whilst setting up logger: {}", e);
        }
    };
    rocket::build()
        .mount(
            "/api",
            routes![
                status,
                get_clip,
                get_clip_empty,
                set_clip,
                set_clip_get,
                version
            ],
        )
        .register("/", catchers![too_many_requests, not_found])
        .manage(RateLimiter::new())
}
