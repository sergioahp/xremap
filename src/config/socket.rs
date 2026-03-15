use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct SocketConfig {
    pub path: String,
}

