use rocket::request::{self, FromRequest, Outcome};

use std::collections::HashMap;
use std::sync::atomic::{AtomicU32, Ordering};

extern crate serde;
extern crate serde_json;

use std::sync::Arc;
use std::time::{Duration, Instant};

use async_lock::RwLock;

#[derive(Clone)]
pub struct RateLimitConfig {
    interval: Duration,
    max_requests: u32,
}

impl RateLimitConfig {
    pub fn new(interval: Duration, max_requests: u32) -> Self {
        RateLimitConfig {
            interval,
            max_requests,
        }
    }
}

#[derive(Clone)]
pub struct RateLimiter {
    requests: Arc<AtomicU32>,
    reset_time: Arc<RwLock<Instant>>,
    config: Arc<RwLock<HashMap<String, RateLimitConfig>>>,
}

impl RateLimiter {
    /// Create a new RateLimiter
    pub fn new() -> Self {
        RateLimiter {
            requests: Arc::new(AtomicU32::new(0)),
            reset_time: Arc::new(RwLock::new(Instant::now())),
            config: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_config(&self, path: &str, config: RateLimitConfig) {
        let mut configs = self.config.write().await;
        configs.insert(path.to_string(), config);
    }

    async fn should_limit(&self, interval: Duration, max_requests: u32) -> bool {
        let mut reset_time = self.reset_time.write().await;
        let requests = self.requests.load(Ordering::Relaxed);

        if reset_time.elapsed() < interval {
            if requests < max_requests {
                self.requests.fetch_add(1, Ordering::Relaxed);
                false
            } else {
                true
            }
        } else {
            *reset_time = Instant::now();
            self.requests.store(1, Ordering::Relaxed);
            false
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RateLimiter {
    type Error = ();

    async fn from_request(request: &'r rocket::Request<'_>) -> request::Outcome<Self, ()> {
        let rate_limiter = request
            .rocket()
            .state::<RateLimiter>()
            .expect("RateLimiter registered as state");

        let uri = request.uri();
        let path = uri.path().to_string();

        let config = {
            let configs = rate_limiter.config.read().await;
            configs.get(&path).cloned().unwrap_or_else(
                || RateLimitConfig::new(Duration::from_secs(60), 20), // By default, allow 20 requests per minute
            )
        };

        if rate_limiter
            .should_limit(config.interval, config.max_requests)
            .await
        {
            Outcome::Error((rocket::http::Status::TooManyRequests, ()))
        } else {
            Outcome::Success(rate_limiter.clone())
        }
    }
}
