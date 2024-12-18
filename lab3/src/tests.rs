use std::env;

use reqwest::Client;

use lab3::types::GeocodingRequest;
use lab3::types::GeocodingResponse;
use lab3::types::PlaceInfo;
use lab3::types::PlaceRequest;
use lab3::types::PlaceResponse;
use lab3::types::PlacesRequest;
use lab3::types::WeatherRequest;
use lab3::types::WeatherResponse;

pub(crate) async fn test_trips(client: &Client) -> reqwest::Result<()> {
    let request = PlacesRequest {
        apikey: env::var("OPENTRIP_KEY").unwrap(),
        radius: 10000.0,
        lat: 54.8618081,
        lon: 83.0809231,
        format: "json".into(),
        limit: 10,
    };

    let response: Vec<PlaceInfo> = client
        .get("http://api.opentripmap.com/0.1/en/places/radius")
        .query(&request)
        .send()
        .await?
        .json()
        .await
        .unwrap();

    for place in response.into_iter().take(3) {
        let request = PlaceRequest {
            apikey: env::var("OPENTRIP_KEY").unwrap(),
        };

        let response: PlaceResponse = client
            .get(format!(
                "http://api.opentripmap.com/0.1/en/places/xid/{}",
                place.xid
            ))
            .query(&request)
            .send()
            .await?
            .json()
            .await?;

        println!("for xid {} : {response:#?}", place.xid);
    }

    Ok(())
}

pub(crate) async fn test_weather(client: &Client) -> reqwest::Result<WeatherResponse> {
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

pub(crate) async fn test_geocoding(client: &Client) -> reqwest::Result<GeocodingResponse> {
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
