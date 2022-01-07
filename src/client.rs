use std::sync::Arc;

use crate::*;

#[derive(Debug, Clone)]
pub struct Client<T: Transport> {
    transport: T,
    config: Arc<Configuration>,
}

impl<T: Transport> Client<T> {
    /// Creates a new `Client` instance with the provided configuration.
    /// 
    /// This method allows you to construct a custom client using your
    /// chosen transport and a specific configuration. It may then be
    /// used to send errors to Rollbar instead of the default client.
    pub fn new(transport: T, config: Configuration) -> Self {
        Client { transport, config: Arc::new(config) }
    }

    /// Reports a new event to Rollbar using this client.
    /// 
    /// This method is the equivalent of the `rollbar_rs::report` method, but
    /// uses the custom client to send the request instead of the default on.
    /// This allows you to use a custom transport, or a custom configuration
    /// for different portions of your application.
    /// 
    /// # Example
    /// ```rust
    /// use rollbar_rs::*;
    /// 
    /// let client = Client::with_default_transport(Configuration::default()).unwrap();
    /// client.report(rollbar_format!(message = "This is a test"));
    /// ```
    pub fn report(&self, data: crate::types::Data) {
        let payload: models::Item = (data, self.config.as_ref()).into();

        if let Some(level) = payload.data.level.clone() {
            if level < self.config.log_level {
                return;
            }
        }
        
        self.transport.send(TransportEvent {
            config: &self.config,
            payload,
        });
    }
}


#[cfg(feature = "async")]
impl Client<TokioTransport> {
    pub fn with_default_transport(config: Configuration) -> Result<Self, Error> {
        Ok(Client::new(TokioTransport::new(&TransportConfig::default())?, config))
    }
}

#[cfg(feature = "threaded")]
impl Client<ThreadedTransport> {
    pub fn with_default_transport(config: Configuration) -> Result<Self, Error> {
        Ok(Client::new(ThreadedTransport::new(&TransportConfig::default())?, config))
    }
}