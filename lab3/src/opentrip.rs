use std::env;

use crate::types::PlaceInfo;
use crate::types::PlaceRequest;
use crate::types::PlaceResponse;
use crate::types::PlacesRequest;

#[derive(Clone)]
pub struct OpentripClient {
    client: reqwest::Client,
    key: String,
}

impl OpentripClient {
    pub fn from_env() -> Self {
        Self {
            client: reqwest::Client::new(),
            key: env::var("OPENTRIP_KEY").unwrap(),
        }
    }

    pub async fn fetch_places(
        &self,
        lat: f64,
        lon: f64,
        radius: f32,
        count: u32,
    ) -> reqwest::Result<Vec<PlaceInfo>> {
        let request = PlacesRequest {
            apikey: self.key.clone(),
            radius,
            lon,
            lat,
            format: "json".to_string(),
            limit: count,
        };

        self.client
            .get("http://api.opentripmap.com/0.1/en/places/radius")
            .query(&request)
            .send()
            .await?
            .json()
            .await
    }

    pub async fn fetch_place(&self, id: String) -> reqwest::Result<PlaceResponse> {
        let request = PlaceRequest {
            apikey: self.key.clone(),
        };

        self.client
            .get(format!("http://api.opentripmap.com/0.1/en/places/xid/{id}"))
            .query(&request)
            .send()
            .await?
            .json()
            .await
    }
}
