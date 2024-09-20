use serde::Deserialize;
use serde::Serialize;
use serde_binary::binary_stream::Endian;
use socket2::SockAddr;

#[derive(Serialize, Deserialize)]
pub struct TransferComplete {
    pub len: u64,
}

impl TransferComplete {
    pub fn new(len: u64) -> Self {
        Self { len }
    }
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

pub fn format_sockaddr(addr: &SockAddr) -> String {
    if let Some(ipv4) = addr.as_socket_ipv4() {
        format!("{ipv4}")
    } else if let Some(ipv6) = addr.as_socket_ipv6() {
        format!("{ipv6}")
    } else {
        format!("UNKNOWN")
    }
}

pub fn bytes_to_hr(bytes: f64) -> String {
    if bytes < 1024.0 {
        format!("{:.2}B", bytes)
    } else if bytes < 1024. * 1024. {
        format!("{:.2}KiB", bytes / 1024.0)
    } else if bytes < 1024. * 1024. * 1024. {
        format!("{:.2}MiB", bytes / (1024.0 * 1024.0))
    } else {
        format!("{:.2}GiB", bytes / (1024.0 * 1024.0 * 1024.0))
    }
}
