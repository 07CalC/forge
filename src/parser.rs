use crate::config::ForgeConfig;
use std::fs;

pub fn load_config(path: &str) -> ForgeConfig {
    let data = fs::read_to_string(path).expect("Unable to read config file");
    serde_yaml::from_str(&data).expect("Invalid YAML format")
}
