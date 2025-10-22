use std::collections::HashMap;

use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ForgeConfig {
    pub services: Vec<Service>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Service {
    pub name: String,
    pub dir: String,
    pub cmd: String,
    pub env: Option<HashMap<String, String>>,
    pub watch: Option<bool>,
}
