use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use crate::schema::*;

#[derive(Insertable)]
#[diesel(table_name = clips)]
pub struct NewClip {
    pub url: String,
    pub code: String,
    pub created_at: NaiveDateTime, // Include if not set by default in the database
    pub expires_at: Option<NaiveDateTime>, // Optional field
}

#[derive(Debug, Queryable, Serialize, Deserialize)]
pub struct Clip {
    pub id: i32,
    pub url: String,
    pub code: String,
    pub created_at: NaiveDateTime,
    pub expires_at: Option<NaiveDateTime>,
}
