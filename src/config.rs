use serde_derive::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ForgeConfig {
    pub services: Vec<Service>,
}

#[derive(Debug, Deserialize)]
pub struct Service {
    pub name: String,
    pub dir: String,
    pub cmd: String,
}
