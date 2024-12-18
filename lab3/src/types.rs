use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
pub struct GeocodingRequest {
    pub key: String,
    pub q: String,
    pub limit: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeocodingPoint {
    pub lat: f64,
    pub lng: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeocodingLocation {
    pub point: GeocodingPoint,
    pub osm_id: u64,
    pub osm_type: String,
    pub osm_key: String,

    #[serde(default)]
    pub name: Option<String>,

    #[serde(default)]
    pub country: Option<String>,

    #[serde(default)]
    pub city: Option<String>,

    #[serde(default)]
    pub state: Option<String>,

    #[serde(default)]
    pub street: Option<String>,

    #[serde(default)]
    pub housenumber: Option<String>,

    #[serde(default)]
    pub postcode: Option<String>,
}

impl std::fmt::Display for GeocodingPoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.5}, {:.5}", self.lat, self.lng)
    }
}

impl std::fmt::Display for Coord {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:.5}, {:.5}", self.lat, self.lon)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeocodingResponse {
    pub hits: Vec<GeocodingLocation>,

    #[serde(default)]
    pub took: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeatherRequest {
    pub lat: f64,
    pub lon: f64,
    pub appid: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Coord {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeatherInfo {
    pub id: u64,
    pub main: String,
    pub description: String,
    pub icon: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeatherMain {
    pub temp: f32,
    pub feels_like: f32,
    pub temp_min: f32,
    pub temp_max: f32,
    pub pressure: u32,
    pub humidity: u32,
    pub sea_level: u32,
    pub grnd_level: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeatherWind {
    pub speed: f32,
    pub gust: f32,
    pub deg: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeatherClouds {
    pub all: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeatherRain {
    #[serde(rename = "1h")]
    pub hour: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeatherSnow {
    #[serde(rename = "1h")]
    pub hour: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WeatherResponse {
    pub coord: Coord,
    pub weather: WeatherInfo,
    pub main: WeatherMain,
    pub wind: WeatherWind,
    pub clouds: WeatherClouds,
    pub visibility: u32,

    #[serde(default)]
    pub rain: Option<WeatherRain>,

    #[serde(default)]
    pub snow: Option<WeatherSnow>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlacesRequest {
    pub apikey: String,
    pub radius: f32,
    pub lon: f64,
    pub lat: f64,
    pub format: String,
    pub limit: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaceRequest {
    pub apikey: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaceInfo {
    pub xid: String,
    pub name: String,
    pub dist: f32,
    pub point: Coord,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaceImagePreview {
    pub source: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaceDetails {
    pub descr: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(from = "String")]
pub struct Rating {
    pub stars: u32,
    pub is_heritage: bool,
}

impl From<String> for Rating {
    fn from(value: String) -> Self {
        let is_heritage = value.ends_with('h');
        let stars = match value.chars().next() {
            Some('1') => 1,
            Some('2') => 2,
            Some('3') => 3,

            _ => 0,
        };

        Self { stars, is_heritage }
    }
}

impl std::fmt::Display for Rating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for _ in 0..self.stars {
            write!(f, "★")?;
        }

        if self.is_heritage {
            write!(f, " Cultural Heritage")?;
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PlaceResponse {
    pub xid: String,
    pub name: String,

    pub rate: Rating,

    #[serde(default)]
    pub info: Option<PlaceDetails>,

    #[serde(default)]
    pub image: Option<String>,

    #[serde(default)]
    pub preview: Option<PlaceImagePreview>,

    #[serde(default)]
    pub wikipedia: Option<String>,
    pub point: Coord,
}
