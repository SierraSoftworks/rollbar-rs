use std::collections::HashMap;

use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct Configuration {
    pub access_token: Option<String>,
    pub environment: Option<String>,
    pub host: Option<String>,
    pub code_version: Option<String>,
    pub log_level: crate::types::Level,
    pub platform: Option<String>,
    pub framework: Option<String>,
    pub context: Option<String>,
    pub custom: Option<HashMap<String, serde_json::Value>>,
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            access_token: None,
            environment: None,
            host: None,
            platform: Some(std::env::consts::OS.to_string()),
            framework: None,
            context: None,
            custom: None,
            code_version: None,
            log_level: crate::types::Level::Info,
        }
    }
}