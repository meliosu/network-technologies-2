#![allow(dead_code)]

use std::fmt::Display;

use axum::{
    extract::{Query, State},
    response::IntoResponse,
    routing, Router,
};

use askama::Template;
use lab3::{
    geocoding::GeocodingClient,
    opentrip::OpentripClient,
    types::{GeocodingLocation, GeocodingPoint},
};
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tower_http::services::ServeFile;

mod tests;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("localhost:1337").await?;

    let router = Router::new()
        .nest_service("/", ServeFile::new("assets/index.html"))
        .route("/search", routing::get(search_locations))
        .with_state(GeocodingClient::from_env())
        .with_state(OpentripClient::from_env());

    axum::serve(listener, router.into_make_service()).await
}

async fn search_locations(
    State(client): State<GeocodingClient>,
    Query(search): Query<Search>,
) -> impl IntoResponse {
    let response = client.fetch_locations(search.name, 10).await.unwrap();

    SearchResults {
        locations: response.hits,
    }
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
