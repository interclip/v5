use crate::models::*;
use crate::schema::*;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::result::DatabaseErrorKind;
use diesel::result::Error;

use std::env;
use std::fmt;

use super::id::gen_id;

/// Tries to connect to the database and if it doesn't exist, creates it from the current schema
/// Returns the connection
pub fn initialize() -> Result<PgConnection, ConnectionError> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
}

/// Returns a clip from the database
pub fn get_clip(
    connection: &mut PgConnection,
    clip_code: String,
) -> Result<Option<Clip>, diesel::result::Error> {
    use crate::schema::clips::dsl::*;

    println!("Searching for clip code: {}", clip_code);

    clips
        .filter(code.eq(clip_code))
        .filter(
            expires_at
                .is_null()
                .or(expires_at.gt(chrono::Local::now().naive_local())),
        )
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
        .filter(
            clips::expires_at
                .is_null()
                .or(clips::expires_at.gt(chrono::Local::now().naive_local())),
        )
        .first::<Clip>(connection)
        .optional()
}

#[derive(Debug)]
pub enum InsertClipError {
    DieselError(diesel::result::Error),
    MaxAttemptsExceeded,
}
impl From<diesel::result::Error> for InsertClipError {
    fn from(error: diesel::result::Error) -> Self {
        InsertClipError::DieselError(error)
    }
}
impl fmt::Display for InsertClipError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InsertClipError::DieselError(e) => write!(f, "Database error: {}", e),
            InsertClipError::MaxAttemptsExceeded => write!(f, "Exceeded maximum attempts to generate a unique code."),
        }
    }
}

/// Inserts a clip into the database
/// Returns the inserted clip
pub fn insert_clip(
    connection: &mut PgConnection,
    url: String,
) -> Result<Clip, InsertClipError> {
    let expiry_date = chrono::Local::now().naive_local() + chrono::Duration::days(7);
    let mut attempts = 0;
    const MAX_ATTEMPTS: usize = 10; // Maximum attempts to generate a unique code

    while attempts < MAX_ATTEMPTS {
        let code = gen_id(5);
        let new_clip = NewClip {
            url: url.clone(),
            code: code.clone(),
            created_at: chrono::Local::now().naive_local(),
            expires_at: Some(expiry_date),
        };

        match diesel::insert_into(clips::table)
            .values(&new_clip)
            .get_result::<Clip>(connection).map_err(InsertClipError::from) {
                Ok(clip) => return Ok(clip),
                Err(InsertClipError::DieselError(Error::DatabaseError(DatabaseErrorKind::UniqueViolation, _))) => {
                    attempts += 1;
                    continue;
                },
                Err(e) => return Err(e), // For any other diesel error, return immediately
        }
    }

    Err(InsertClipError::MaxAttemptsExceeded)
}


/// Deletes expired clips from the database
pub fn collect_garbage(connection: &mut PgConnection) -> Result<usize, diesel::result::Error> {
    use crate::schema::clips::dsl::*;

    diesel::delete(clips.filter(expires_at.is_not_null().and(expires_at.lt(chrono::Local::now().naive_local())))).execute(connection)
}