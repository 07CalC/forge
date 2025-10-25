use std::collections::HashMap;

use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FyrerConfig {
    pub installers: Option<Vec<Installer>>,
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

#[derive(Debug, Deserialize, Clone)]
pub struct Installer {
    pub dir: String,
    pub cmd: String,
}
