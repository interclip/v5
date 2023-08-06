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

use utils::db::{self, get_db_clip, get_db_clip_by_url, insert_db_clip};

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
    let result = get_db_clip("test".to_string());

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

    // Check for existence of the URL in the database
    let existing_clip = get_db_clip_by_url(url.to_string());

    match existing_clip {
        Ok(existing_clip) => {
            if let Some(existing_clip) = existing_clip {
                let response = APIResponse {
                    status: APIStatus::Success,
                    result: existing_clip,
                };
                return Ok(Json(response));
            }
        }
        Err(e) => {
            error!("{}", e);
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

    let result = get_db_clip(code);
    match result {
        Ok(result) => match result {
            Some(result) => {
                let response = APIResponse {
                    status: APIStatus::Success,
                    result,
                };

                Ok(Custom(Status::Created, Json(response)))
            }
            None => {
                let response = APIResponse {
                    status: APIStatus::Error,
                    result: "Clip not found".to_string(),
                };

                Err(Custom(Status::NotFound, Json(response)))
            }
        },
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
    match db::initialize() {
        Ok(_) => {
            println!("DB set up successfully")
        }
        Err(e) => {
            panic!("Error whilst setting up database: {}", e)
        }
    }
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
