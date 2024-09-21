#![allow(dead_code)]

use axum::{
    extract::{FromRef, Query, State},
    response::IntoResponse,
    routing, Router,
};

use askama::Template;
use lab3::{
    geocoding::GeocodingClient,
    opentrip::OpentripClient,
    types::{Coord, GeocodingLocation, PlaceResponse},
};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tower_http::services::ServeFile;

mod tests;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("localhost:1337").await?;

    let state = AppState {
        geocoding: GeocodingClient::from_env(),
        opentrip: OpentripClient::from_env(),
    };

    let router = Router::new()
        .nest_service("/", ServeFile::new("assets/index.html"))
        .route("/search", routing::get(search_locations))
        .route("/places", routing::get(explore_location))
        .with_state(state);

    axum::serve(listener, router.into_make_service()).await
}

async fn search_locations(
    State(client): State<GeocodingClient>,
    Query(search): Query<Search>,
) -> impl IntoResponse {
    let response = client.fetch_locations(search.name, 30).await.unwrap();

    SearchResults {
        locations: response.hits,
    }
}

async fn explore_location(
    State(client): State<OpentripClient>,
    Query(coord): Query<Coord>,
) -> impl IntoResponse {
    let response = client
        .fetch_places(coord.lat, coord.lon, 50000.0, 20)
        .await
        .unwrap();

    let mut results = Vec::new();

    for place in response {
        if let Ok(details) = client.fetch_place(place.xid).await {
            if !details.name.is_empty() {
                results.push(details);
            }
        }
    }

    PlacesSearchResults { places: results }
}

#[derive(Serialize, Deserialize)]
struct Search {
    name: String,
}

#[derive(Template)]
#[template(path = "search-results.html")]
struct SearchResults {
    locations: Vec<GeocodingLocation>,
}

#[derive(Clone, FromRef)]
struct AppState {
    geocoding: GeocodingClient,
    opentrip: OpentripClient,
}

#[derive(Template)]
#[template(path = "places-results.html", escape = "none")]
struct PlacesSearchResults {
    places: Vec<PlaceResponse>,
}

pub fn place_description(place: &PlaceResponse) -> Option<&str> {
    if let Some(ref info) = place.info {
        info.descr.as_deref()
    } else {
        None
    }
}
