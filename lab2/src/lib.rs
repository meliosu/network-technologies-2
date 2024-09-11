use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Announce {
    pub len: usize,
    pub filename: String,
}

#[derive(Serialize, Deserialize)]
pub enum AnnounceResponse {
    Ready,
    NotEnoughSpace,
    TooBigFile,
    InvalidFilename,
}
