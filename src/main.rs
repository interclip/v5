mod models;
mod schema;
mod utils;

use clokwerk::{Scheduler, TimeUnits};
use regex::Regex;
use rocket::http::{Header, Status};
use rocket::response::status::Custom;
use rocket::State;
use utils::files::{create_storage_client, put_object};
use utils::id::gen_id;
use utils::log::setup_logger;
use utils::rate_limit::RateLimitConfig;

use dotenv::dotenv;

use std::env;
use std::result::Result;
use std::result::Result::Ok;
use std::string::String;
use std::time::Duration;

use rocket::form::Form;
use rocket::serde::json::Json;

use utils::db::{self, collect_garbage};

use crate::utils::rate_limit::RateLimiter;
use crate::utils::structs::{APIResponse, APIStatus};

use aws_sdk_s3::Client;

extern crate rand;
extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate rocket;
extern crate fern;
extern crate log;

include!(concat!(env!("OUT_DIR"), "/git_commit.rs"));
#[derive(rocket::FromForm, serde::Deserialize)]
#[serde(crate = "rocket::serde")]
struct UploadQuery {
    name: String,
    size: Option<usize>,
}

#[get("/upload-file?<query..>")]
async fn upload_file(
    _rate_limiter: RateLimiter,
    s3_client: &State<Client>,
    query: UploadQuery,
) -> Result<Json<APIResponse>, Custom<Json<APIResponse>>> {
    if query.name.is_empty() {
        let response = APIResponse {
            status: APIStatus::Error,
            result: "File name is empty".to_string(),
        };
        return Err(Custom(Status::BadRequest, Json(response)));
    }

    let max_size = 100 * 1024 * 1024; // 100MB
    if let Some(size) = query.size {
        if size > max_size {
            let response = APIResponse {
                status: APIStatus::Error,
                result: "File is too large".to_string(),
            };
            return Err(Custom(Status::PayloadTooLarge, Json(response)));
        }
    }

    let bucket = "iclip";
    let object_key = format!("{}/{}", gen_id(10), query.name);

    match put_object(s3_client, bucket, &object_key, 60).await {
        Ok(presigned_url) => {
            let response = APIResponse {
                status: APIStatus::Success,
                result: presigned_url,
            };
            Ok(Json(response))
        }
        Err(err) => {
            error!("{}", err);
            let response = APIResponse {
                status: APIStatus::Error,
                result: "A server-side problem has occurred".to_string(),
            };
            Err(Custom(Status::InternalServerError, Json(response)))
        }
    }
}

#[get("/status")]
fn status(_rate_limiter: RateLimiter) -> Result<Json<APIResponse>, Custom<Json<APIResponse>>> {
    let mut db_connection = match db::initialize() {
        Ok(conn) => conn,
        Err(err) => {
            error!("{}", err);
            let response = APIResponse {
                status: APIStatus::Error,
                result: "A problem with the database has occurred".to_string(),
            };
            return Err(Custom(Status::InternalServerError, Json(response)));
        }
    };

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

#[derive(FromForm)]
struct SetClipRequest {
    url: String,
}

#[post("/set", data = "<form_data>")]
fn set_clip(
    form_data: Form<SetClipRequest>,
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

    if url.scheme() != "http" && url.scheme() != "https" {
        let response = APIResponse {
            status: APIStatus::Error,
            result: "Invalid URL scheme".to_string(),
        };
        return Err(Custom(Status::BadRequest, Json(response)));
    }

    let mut db_connection = match db::initialize() {
        Ok(conn) => conn,
        Err(err) => {
            error!("{}", err);
            let response = APIResponse {
                status: APIStatus::Error,
                result: "A problem with the database has occurred".to_string(),
            };
            return Err(Custom(Status::InternalServerError, Json(response)));
        }
    };

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

    let code_pattern = Regex::new(r"^(?i)[A-Z0-9]{5}$").unwrap();
    if !code_pattern.is_match(code.as_str()) {
        let response = APIResponse {
            status: APIStatus::Error,
            result: "Invalid clip code format".to_string(),
        };
        return Err(Custom(Status::BadRequest, Json(response)));
    }

    let mut db_connection = match db::initialize() {
        Ok(conn) => conn,
        Err(err) => {
            error!("{}", err);
            let response = APIResponse {
                status: APIStatus::Error,
                result: "A problem with the database has occurred".to_string(),
            };
            return Err(Custom(Status::InternalServerError, Json(response)));
        }
    };

    let result = db::get_clip(&mut db_connection, code);
    match result {
        Ok(Some(clip)) => {
            let response = APIResponse {
                status: APIStatus::Success,
                result: clip.url,
            };
            Ok(Custom(Status::Ok, Json(response)))
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
    Json(Version {
        commit: Some(GIT_COMMIT.to_string()),
    })
}

#[launch]
async fn rocket() -> _ {
    dotenv().ok();

    match setup_logger() {
        Ok(path) => {
            println!("Logger setup at {}", path);
        }
        Err(e) => {
            println!("Error whilst setting up logger: {}", e);
        }
    };

    let rate_limiter = RateLimiter::new();
    rate_limiter
        .add_config(
            "/api/get",
            RateLimitConfig::new(Duration::from_secs(30), 100),
        )
        .await;
    rate_limiter
        .add_config(
            "/api/set",
            RateLimitConfig::new(Duration::from_secs(60), 20),
        )
        .await;
    rate_limiter
        .add_config(
            "/api/status",
            RateLimitConfig::new(Duration::from_secs(30), 20),
        )
        .await;

    let s3_client = create_storage_client().await.unwrap();

    let mut db_connection = db::initialize().expect("Failed to connect to the database");
    let mut scheduler = Scheduler::with_tz(chrono::Utc);
    scheduler.every(1.hour()).run(move || {
        let _ = collect_garbage(&mut db_connection);
    });

    rocket::build()
        .mount(
            "/api",
            routes![
                status,
                get_clip,
                get_clip_empty,
                set_clip,
                set_clip_get,
                version,
                upload_file
            ],
        )
        .register("/", catchers![too_many_requests, not_found])
        .manage(rate_limiter)
        .attach(rocket::fairing::AdHoc::on_response(
            "CORS headers",
            |_, res| {
                Box::pin(async move {
                    res.set_header(Header::new("Access-Control-Allow-Origin", "*"));
                })
            },
        ))
        .manage(s3_client)
}
