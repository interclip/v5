use crate::models::*;
use crate::schema::*;

use diesel::prelude::*;
use diesel::sqlite::SqliteConnection;

use std::env;

/// Tries to connect to the database and if it doesn't exist, it creates it from the current schema
pub fn initialize() {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let mut connection = SqliteConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url));

    create(&mut connection);
}

fn create(connection: &mut SqliteConnection) {
    let new_log = NewClip {
        url: "https://github.com".to_string(),
        code: "test".to_string(),
    };

    let inserted_row = diesel::insert_into(clips::table)
        .values(&new_log)
        .get_result::<Clip>(connection);

    println!("{:?}", inserted_row);
}