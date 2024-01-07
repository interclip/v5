use crate::schema::*;
use diesel::prelude::*;
use serde::Serialize;

#[derive(Insertable)]
#[table_name = "clips"]
pub struct NewClip {
    pub code: String,
    pub url: String,
}

#[derive(Debug, Queryable, Serialize)]
pub struct Clip {
    pub id: i32,
    pub code: String,
    pub url: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}
