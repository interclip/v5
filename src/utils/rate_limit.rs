use rocket::request::{self, FromRequest, Outcome};

use std::sync::atomic::{AtomicU32, Ordering};

extern crate serde;
extern crate serde_json;

use std::sync::Arc;
use std::time::{Duration, Instant};

use async_lock::RwLock;

#[derive(Clone)]
pub struct RateLimiter {
    requests: Arc<AtomicU32>,
    reset_time: Arc<RwLock<Instant>>,
}

impl RateLimiter {
    /// Create a new RateLimiter
    pub fn new() -> Self {
        RateLimiter {
            requests: Arc::new(AtomicU32::new(0)),
            reset_time: Arc::new(RwLock::new(Instant::now())),
        }
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

        if rate_limiter.should_limit(Duration::from_secs(10), 15).await {
            Outcome::Error((rocket::http::Status::TooManyRequests, ()))
        } else {
            Outcome::Success(rate_limiter.clone())
        }
    }
}
