use std::env;

use lab3::types::{GeocodingRequest, GeocodingResponse, WeatherRequest, WeatherResponse};
use reqwest::Client;

#[tokio::main]
async fn main() {
    let client = Client::new();

    let response = test_weather(&client).await.unwrap_or_else(|err| {
        panic!("error getting weather info: {err}");
    });

    println!("{response:#?}");
}

async fn test_weather(client: &Client) -> reqwest::Result<WeatherResponse> {
    let request = WeatherRequest {
        appid: env::var("OPENWEATHER_KEY").unwrap(),
        lat: 54.8618081,
        lon: 83.0809231,
    };

    let response = client
        .get("https://api.openweathermap.org/data/2.5/weather")
        .query(&request)
        .send()
        .await?;

    eprintln!("{}", response.text().await?);

    todo!()
}

async fn test_geocoding(client: &Client) -> reqwest::Result<GeocodingResponse> {
    let request = GeocodingRequest {
        key: env::var("GRAPHHOPPER_KEY").unwrap(),
        q: "Novosibirsk".to_string(),
        limit: 5,
    };

    client
        .get("https://graphhopper.com/api/1/geocode")
        .query(&request)
        .send()
        .await?
        .json()
        .await
}
