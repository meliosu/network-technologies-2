use std::env;

use crate::types::GeocodingRequest;
use crate::types::GeocodingResponse;

#[derive(Clone)]
pub struct GeocodingClient {
    client: reqwest::Client,
    key: String,
}

impl GeocodingClient {
    pub fn from_env() -> Self {
        Self::from_key(env::var("GRAPHHOPPER_KEY").unwrap())
    }

    pub fn from_key(key: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            key,
        }
    }

    pub async fn fetch_locations(
        &self,
        query: String,
        count: u32,
    ) -> reqwest::Result<GeocodingResponse> {
        let request = GeocodingRequest {
            key: self.key.clone(),
            q: query,
            limit: count,
        };

        self.client
            .get("https://graphhopper.com/api/1/geocode")
            .query(&request)
            .send()
            .await?
            .json()
            .await
    }
}
