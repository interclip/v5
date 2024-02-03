use std::time::Duration;

use crate::env;

use aws_config::meta::region::RegionProviderChain;
use aws_sdk_s3::presigning::config::PresigningConfig;
use aws_sdk_s3::Client;
use aws_sdk_s3::{Endpoint, Region};

pub async fn create_storage_client() -> Result<Client, aws_sdk_s3::Error> {
    let shared_config = aws_config::from_env()
        .region(RegionProviderChain::default_provider().or_else("eu-central-1"))
        .load()
        .await;

    if let Ok(endpoint_str) = env::var("CUSTOM_ENDPOINT") {
        let region = shared_config
            .region()
            .cloned()
            .unwrap_or_else(|| Region::new("eu-central-1"));
        let credentials_provider = shared_config.credentials_provider().unwrap().clone();
        let endpoint = Endpoint::immutable(endpoint_str.parse().expect("Invalid URI"));

        let client_config = aws_sdk_s3::Config::builder()
            .region(region)
            .endpoint_resolver(endpoint)
            .credentials_provider(credentials_provider)
            .build();

        return Ok(Client::from_conf(client_config));
    }

    Ok(Client::new(&shared_config))
}

pub async fn put_object(
    client: &Client,
    bucket: &str,
    object: &str,
    expires_in: u64,
) -> Result<String, String> {
    let expires_in = Duration::from_secs(expires_in);
    let presigning_config = PresigningConfig::expires_in(expires_in)
        .map_err(|e| format!("Failed to create presigning config: {}", e))?;

    match client
        .put_object()
        .bucket(bucket)
        .key(object)
        .presigned(presigning_config)
        .await
    {
        Ok(presigned_request) => Ok(presigned_request.uri().to_string()),
        Err(e) => Err(format!("Failed to create presigned URL: {}", e)),
    }
}
