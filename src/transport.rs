#[cfg(feature = "async")]
use std::sync::Arc;

#[cfg(feature = "threaded")]
use std::sync::{Arc, Condvar, mpsc::{sync_channel, SyncSender, Receiver}};

use std::{time::Duration, sync::Mutex};
use serde::{Deserialize, Serialize};
use crate::models::Item;
use crate::{Configuration, Error};

#[cfg(feature = "tokio")]
use reqwest::Client;

#[cfg(not(feature = "async"))]
use reqwest::blocking::Client;

use crate::errors::*;

#[derive(Debug, Clone)]
pub struct TransportConfig {
    pub endpoint: String,
    pub timeout: Duration,
    pub proxy: Option<String>,
}

impl Default for TransportConfig {
    fn default() -> Self {
        TransportConfig {
            endpoint: "https://api.rollbar.com/api/1/item/".to_string(),
            timeout: Duration::from_millis(10000),
            proxy: None,
        }
    }
}

pub trait Transport: Send + Sync + Sized {
    fn new(config: &TransportConfig) -> Result<Self, Error>;
    fn send(&self, event: TransportEvent);
}

pub struct TransportEvent<'a> {
    pub config: &'a Configuration,
    pub payload: Item,
}

#[cfg(feature = "async")]
#[derive(Debug, Clone)]
pub struct TokioTransport {
    endpoint: Arc<String>,
    client: Arc<Client>,
}

#[cfg(feature = "async")]
impl Transport for TokioTransport {
    fn new(config: &TransportConfig) -> Result<Self, Error> {
        let mut client = Client::builder()
            .gzip(true)
            .timeout(config.timeout)
            .user_agent(concat!("SierraSoftworks/rollbar-rs v", env!("CARGO_PKG_VERSION")));
        
        if let Some(proxy) = &config.proxy {
            client = client.proxy(reqwest::Proxy::all(proxy).map_err(|e| user_with_internal(
                "We could not configure Rollbar to use the proxy you provided.",
                "Make sure that you have specified a valid proxy URL in your configuration and try again.",
                e
            ))?);
        }

        let client = client.build().map_err(|e| user_with_internal(
            "We could not configure Rollbar based on the configuration you have provided.",
            "Make sure that you have specified a valid configuration and try again.",
            e
        ))?;

        Ok(Self {
            endpoint: Arc::new(config.endpoint.clone()),
            client: Arc::new(client),
        })
    }

    fn send(&self, event: TransportEvent) {
        let client = self.client.clone();
        let endpoint = self.endpoint.clone();
        let access_token = event.config.access_token.clone();

        match access_token {
            Some(access_token) => {
                tokio::spawn(async move {
                    let mut req = client
                        .post(endpoint.as_str())
                        .json(&event.payload);
        
                    if let Some(mut access_token) = reqwest::header::HeaderValue::from_str(&access_token).ok() {
                        access_token.set_sensitive(true);
                        req = req.header("X-Rollbar-Access-Token", access_token);
                    }
        
                    match req.send().await {
                        Ok(resp) => debug!("Successfully sent payload to Rollbar: {}", resp.json().await.ok().and_then(|r: RollbarResponse| serde_json::to_string_pretty(&r).ok()).unwrap_or_default()),
                        Err(e) => error!("We could not send the payload to Rollbar: {}", e),
                    };
                });
            },
            None => {}
        }        
    }
}

#[cfg(feature = "threaded")]
#[derive(Debug)]
pub struct ThreadedTransport {
    chan: SyncSender<Option<(String, Item)>>,
    running: Arc<Mutex<bool>>,
    running_changed: Arc<Condvar>,
    _thread: std::thread::JoinHandle<()>,
}

#[cfg(feature = "threaded")]
impl Transport for ThreadedTransport {
    fn new(config: &TransportConfig) -> Result<Self, Error> {
        let mut client = Client::builder()
            .gzip(true)
            .timeout(config.timeout)
            .user_agent(concat!("SierraSoftworks/rollbar-rs v", env!("CARGO_PKG_VERSION")));
        
        if let Some(proxy) = &config.proxy {
            client = client.proxy(reqwest::Proxy::all(proxy).map_err(|e| user_with_internal(
                "We could not configure Rollbar to use the proxy you provided.",
                "Make sure that you have specified a valid proxy URL in your configuration and try again.",
                e
            ))?);
        }

        let client = client.build().map_err(|e| user_with_internal(
            "We could not configure Rollbar based on the configuration you have provided.",
            "Make sure that you have specified a valid configuration and try again.",
            e
        ))?;
        let endpoint = config.endpoint.clone();
        
        let (tx, rx): (SyncSender<Option<(String, Item)>>, Receiver<Option<(String, Item)>>) = sync_channel(100);
        let running = Arc::new(Mutex::new(true));
        let running_changed = Arc::new(Condvar::new());


        
        let thread = {
            let running = running.clone();
            let running_changed = running_changed.clone();

            std::thread::spawn(move || {
                while let Some((access_token, item)) = rx.recv().unwrap_or(None) {
                    debug!("ThreadedTransport: Received item to send to Rollbar");
                    let mut req = client
                        .post(endpoint.as_str())
                        .json(&item);
            
                    if let Some(mut access_token) = reqwest::header::HeaderValue::from_str(access_token.as_str()).ok() {
                        access_token.set_sensitive(true);
                        req = req.header("X-Rollbar-Access-Token", access_token);
                    }
            
                    debug!("ThreadedTransport: Sending item to Rollbar");
                    match req.send() {
                        Ok(resp) => debug!("Successfully sent payload to Rollbar: {}", resp.json().ok().and_then(|r: RollbarResponse| serde_json::to_string_pretty(&r).ok()).unwrap_or_default()),
                        Err(e) => error!("We could not send the payload to Rollbar: {}", e),
                    };
                }

                let mut is_running = running.lock().unwrap();
                *is_running = false;
                running_changed.notify_all();

                info!("ThreadedTransport: Exiting thread");
            })
        };

        Ok(Self {
            chan: tx,
            running,
            running_changed,
            _thread: thread,
        })
    }

    fn send(&self, event: TransportEvent) {
        if let Some(access_token) = event.config.access_token.clone() {
            self.chan.send(Some((access_token, event.payload))).unwrap_or_else(|e| {
                error!("We could not send the payload to Rollbar: {}", e);
            });
        } else {
            debug!("Skipping sending payload to Rollbar since there is no access token");
        }
    }
}

#[cfg(feature = "threaded")]
impl Drop for ThreadedTransport {
    fn drop(&mut self) {
        self.chan.send(None).ok();

        let is_running = self.running.lock().unwrap();
        self.running_changed.wait_timeout(is_running, Duration::from_secs(5)).ok();
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RollbarResponse {
    err: u8,
    result: Option<RollbarResultSuccess>,
    message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RollbarResultSuccess {
    id: Option<String>,
    uuid: Option<String>,
}

#[cfg(test)]
mod tests {
    use crate::*;
    use super::*;
    use httptest::{Server, Expectation, matchers::*, responders::*};

    #[test_log::test]
    #[cfg(feature = "threaded")]
    fn test_threaded_transport() {
        let server = Server::run();
        server.expect(
            Expectation::matching(request::method_path("POST", "/api/1/item/"))
                .respond_with(status_code(200))
        );

        let transport = ThreadedTransport::new(&TransportConfig {
            endpoint: server.url("/api/1/item/").to_string(),
            timeout: Duration::from_millis(100),
            proxy: None,
        }).unwrap();

        let config = Configuration {
            access_token: Some("12345".to_string()),
            ..Default::default()
        };

        debug!("Queueing item to send to Rollbar");

        transport.send(TransportEvent {
            config: &config,
            payload: models::Item {
                data: rollbar_format!(message = "Test message"),
            },
        });

        debug!("Item queued for send to Rollbar");
    }
}