pub mod proto {
    include!(concat!(env!("OUT_DIR"), "/snakes.rs"));
}

pub mod bridge;
pub mod config;
pub mod logic;
pub mod net;
