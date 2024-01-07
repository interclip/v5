use crate::models::*;
use crate::schema::*;

use diesel::pg::PgConnection;
use diesel::prelude::*;

use dotenv::dotenv;
use std::env;

/// Tries to connect to the database and if it doesn't exist, it creates it from the current schema
/// Returns the connection
pub fn initialize() -> PgConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let connection = PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url));

    return connection;
}

/// Gets a clip from the database
pub fn get_clip(
    connection: &mut PgConnection,
    clip_code: String,
) -> Result<Option<Clip>, diesel::result::Error> {
    use crate::schema::clips::dsl::*;

    println!("Searching for clip code: {}", clip_code);

    clips
        .filter(code.eq(clip_code))
        .first::<Clip>(connection)
        .optional()
}

/// Looks for a clip in the database by its URL
/// Returns the clip if it exists
pub fn get_clip_by_url(
    connection: &mut PgConnection,
    url: String,
) -> Result<Option<Clip>, diesel::result::Error> {
    clips::table
        .filter(clips::url.eq(url))
        .first::<Clip>(connection)
        .optional()
}

/// Inserts a clip into the database
/// Returns the inserted clip
pub fn insert_clip(
    connection: &mut PgConnection,
    url: String,
    code: String,
) -> Result<Clip, diesel::result::Error> {
    let new_clip = NewClip {
        url,
        code,
        created_at: chrono::Local::now().naive_local(),
        expires_at: None,
    };

    diesel::insert_into(clips::table)
        .values(&new_clip)
        .get_result::<Clip>(connection)
}
