mod model;
mod p1_client;

use async_std::task;
use jarvis_lib::config_client::{ConfigClient, ConfigClientConfig};
use jarvis_lib::exporter_service::{ExporterService, ExporterServiceConfig};
use jarvis_lib::nats_client::{NatsClient, NatsClientConfig};
use jarvis_lib::state_client::{StateClient, StateClientConfig};
use p1_client::{P1Client, P1ClientConfig};

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let p1_client_config = P1ClientConfig::from_env()?;
    let p1_client = P1Client::new(p1_client_config);

    let state_client_config = task::block_on(StateClientConfig::from_env())?;
    let state_client = StateClient::new(state_client_config);

    let nats_client_config = task::block_on(NatsClientConfig::from_env())?;
    let nats_client = NatsClient::new(nats_client_config);

    let config_client_config = ConfigClientConfig::from_env()?;
    let config_client = ConfigClient::new(config_client_config);

    let exporter_service_config = ExporterServiceConfig::new(
        config_client,
        nats_client,
        state_client,
        Box::new(p1_client),
    )?;
    let mut exporter_service = ExporterService::new(exporter_service_config);

    task::block_on(exporter_service.run())?;

    Ok(())
}
