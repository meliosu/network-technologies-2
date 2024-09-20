use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct TransferRequest {
    pub len: u64,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct TransferError;
