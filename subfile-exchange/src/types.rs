use serde::{Deserialize, Serialize};

pub const CHUNK_SIZE: usize = 1024 * 1024; // Define the chunk size, e.g., 1 MB

#[derive(Serialize, Deserialize)]
pub struct Health {
    pub healthy: bool,
}

#[derive(Serialize, Deserialize)]
pub struct Operator {
    #[serde(alias = "publicKey")]
    pub public_key: String,
}
