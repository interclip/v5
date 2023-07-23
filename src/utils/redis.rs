use redis::{Commands, Connection, RedisResult};
extern crate dotenv;
use dotenv::dotenv;
use std::env;

/// Load environment variables and create the Redis connection string
fn load_redis_url() -> String {
    dotenv().ok();

    let server = env::var("REDIS_HOST").unwrap_or("localhost".to_string());
    let username = env::var("REDIS_USERNAME").unwrap_or("".to_string());
    let password = env::var("REDIS_PASSWORD").unwrap_or("".to_string());

    format!("redis://{}:{}@{}/", username, password, server)
}

fn get_redis_conn() -> RedisResult<Connection> {
    trace!("Establishing Redis connection");
    let client = redis::Client::open(load_redis_url())?;
    match client.get_connection() {
        Ok(conn) => Ok(conn),
        Err(e) => {
            error!("Error with Redis: {}", e);
            Err(e)
        }
    }
}

pub fn cache_clip(code: &str, url: &str) -> Result<(), redis::RedisError> {
    let mut redis_conn = get_redis_conn()?;
    let _: () = redis_conn.set_ex(code, url, 2592000)?; // Cache for 30 days (2592000 seconds)
    Ok(())
}

pub fn get_cached_clip(code: &String) -> Result<Option<String>, redis::RedisError> {
    let mut redis_conn = get_redis_conn()?;
    let result: RedisResult<Option<String>> = redis_conn.get(code);
    match result {
        Ok(Some(url)) => Ok(Some(url)),
        Ok(None) => Ok(None),
        Err(e) => {
            error!("Error with Redis: {}", e);
            Err(e)
        }
    }
}
