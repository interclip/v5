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
#[get("/get?<code>")]
fn get_clip(code: String) -> String {
    format!("get: {}", code)
}

#[get("/get")]
fn get_clip_empty() -> String {
    format!("get: {}", "no code")
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/api", routes![index, get_clip, get_clip_empty])
}
