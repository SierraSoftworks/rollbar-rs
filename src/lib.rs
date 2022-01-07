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
#[cfg(not(feature = "async"))]
lazy_static::lazy_static! {
    pub (in crate) static ref TRANSPORT: ThreadedTransport = ThreadedTransport::new(&TransportConfig::default()).unwrap();
}

/// Removes any configured access token, disabling Rollbar.
/// 
/// This method can be used to disable Rollbar reporting at runtime
/// without having to recompile your application.
pub fn unset_token() {
    CONFIG.write().map(|mut c| c.access_token = None).unwrap();
}

pub fn set_token<S: Into<String>>(token: S) {
    CONFIG.write().map(|mut c| c.access_token = Some(token.into())).unwrap();
}

pub fn set_environment<S: Into<String>>(environment: S) {
    CONFIG.write().map(|mut c| c.environment = Some(environment.into())).unwrap();
}

pub fn set_host<S: Into<String>>(host: S) {
    CONFIG.write().map(|mut c| c.host = Some(host.into())).unwrap();
}

pub fn set_code_version<S: Into<String>>(code_version: S) {
    CONFIG.write().map(|mut c| c.code_version = Some(code_version.into())).unwrap();
}

pub fn set_log_level(level: types::Level) {
    CONFIG.write().map(|mut c| c.log_level = level).unwrap();
}

pub fn set_platform<S: Into<String>>(platform: S) {
    CONFIG.write().map(|mut c| c.platform = Some(platform.into())).unwrap();
}

pub fn set_framework<S: Into<String>>(framework: S) {
    CONFIG.write().map(|mut c| c.framework = Some(framework.into())).unwrap();
}

pub fn set_context<S: Into<String>>(context: S) {
    CONFIG.write().map(|mut c| c.context = Some(context.into())).unwrap();
}

pub fn set_custom<S: Into<String>>(key: S, value: serde_json::Value) {
    CONFIG.write().map(|mut c| {
        match c.custom {
            Some(ref mut custom) => {
                custom.insert(key.into(), value);
            },
            None => {
                c.custom = Some(HashMap::new());
                c.custom.as_mut().unwrap().insert(key.into(), value);
            }
        }
    }).unwrap();
}

pub fn report(data: types::Data) {
    lazy_static::initialize(&TRANSPORT);

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