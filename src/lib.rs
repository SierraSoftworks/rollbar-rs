#[macro_use] extern crate log;
extern crate serde;

mod client;
mod configuration;
mod errors;
pub mod helpers;
mod macros;
mod models;
mod transport;

use std::{sync::RwLock, collections::HashMap};

pub use client::Client;
pub use configuration::Configuration;
pub use errors::Error;
pub use transport::*;
pub use rollbar_rust::types::{self, Level, Person, Server, Request, };

/// The version of the rollbar-rs crate that is being used.
pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");

lazy_static::lazy_static! {
    pub (in crate) static ref CONFIG: RwLock<Configuration> = RwLock::new(Configuration::default());
}

#[cfg(feature = "async")]
lazy_static::lazy_static! {
    pub (in crate) static ref TRANSPORT: TokioTransport = TokioTransport::new(&TransportConfig::default()).unwrap();
}

#[cfg(feature = "threaded")]
lazy_static::lazy_static! {
    pub (in crate) static ref TRANSPORT: ThreadedTransport = ThreadedTransport::new(&TransportConfig::default()).unwrap();
}

pub fn set_token(token: &str) {
    CONFIG.write().unwrap().access_token = Some(token.to_string());
}

pub fn set_environment(environment: &str) {
    CONFIG.write().unwrap().environment = Some(environment.to_string());
}

pub fn set_host(host: &str) {
    CONFIG.write().unwrap().host = Some(host.to_string());
}

pub fn set_code_version(code_version: &str) {
    CONFIG.write().unwrap().code_version = Some(code_version.to_string());
}

pub fn set_log_level(level: types::Level) {
    CONFIG.write().unwrap().log_level = level;
}

pub fn set_platform(platform: &str) {
    CONFIG.write().unwrap().platform = Some(platform.to_string());
}

pub fn set_framework(framework: &str) {
    CONFIG.write().unwrap().framework = Some(framework.to_string());
}

pub fn set_context(context: &str) {
    CONFIG.write().unwrap().context = Some(context.to_string());
}

pub fn set_custom(key: &str, value: serde_json::Value) {
    let mut config = CONFIG.write().unwrap();

    match config.custom {
        Some(ref mut custom) => {
            custom.insert(key.to_string(), value);
        },
        None => {
            config.custom = Some(HashMap::new());
            config.custom.as_mut().unwrap().insert(key.to_string(), value);
        }
    }
}

pub fn report(data: types::Data) {
    let config = CONFIG.read().unwrap();

    let cfg: &Configuration = &config;

    let payload: models::Item = (data, cfg).into();

    if let Some(level) = payload.data.level.clone() {
        if level < config.log_level {
            return;
        }
    }

    TRANSPORT.send(TransportEvent {
        config: &config,
        payload,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_global_config() {
        set_token("test_token");
        assert_eq!(CONFIG.read().unwrap().access_token, Some("test_token".to_string()));
    }
}