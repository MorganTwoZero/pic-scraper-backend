use config_builder::get_configuration;
use etl::startup::{shutdown_signal, Application};
use tele::setup_telemetry;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = get_configuration().expect("Failed to read config");
    setup_telemetry();

    Application::build(config)
        .await
        .run()
        .await
        .expect("Failed to create fill db task");
    tracing::info!("Scraper task started");

    shutdown_signal().await;

    Ok(())
}
