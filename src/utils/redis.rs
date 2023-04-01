use redis::{Commands, Connection, RedisResult};

static REDIS_URL: &str = "redis://localhost/";

fn get_redis_conn() -> RedisResult<Connection> {
    let client = redis::Client::open(REDIS_URL)?;
    match client.get_connection() {
        Ok(conn) => Ok(conn),
        Err(e) => {
            println!("Error: {}", e);
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
    let result: Option<String> = redis_conn.get(code)?;
    Ok(result)
}
