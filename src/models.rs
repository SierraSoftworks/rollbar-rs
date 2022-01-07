use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct Item {
    pub data: rollbar_rust::types::Data,
}

#[allow(dead_code)]
pub fn new_uuid() -> String {
    rollbar_rust::Uuid::new().to_string()
}