use serde::{Deserialize, Serialize};
use serde_binary::binary_stream::Endian;

pub struct TransferComplete {
    pub len: u64,
}

#[derive(Serialize, Deserialize)]
pub struct TransferRequest {
    pub len: u64,
    pub name: String,
}

impl TransferRequest {
    pub fn new(name: String, len: u64) -> Self {
        Self { name, len }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        serde_binary::to_vec(&self, Endian::Big).unwrap()
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        serde_binary::from_slice(bytes, Endian::Big).unwrap()
    }
}

#[derive(Serialize, Deserialize)]
pub enum TransferResponse {
    Success,
    Failure,
}

impl TransferResponse {
    pub fn to_bytes(&self) -> Vec<u8> {
        serde_binary::to_vec(&self, Endian::Big).unwrap()
    }

    pub fn from_bytes(bytes: &[u8]) -> Self {
        serde_binary::from_slice(bytes, Endian::Big).unwrap()
    }
}

pub mod client;
pub mod server;
