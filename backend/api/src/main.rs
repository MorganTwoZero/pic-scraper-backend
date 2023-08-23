use api::startup::Application;
use config_builder::get_configuration;
use tele::setup_telemetry;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = get_configuration().unwrap();
    setup_telemetry();

    tracing::info!(
        "Server running on http://{}:{}",
        config.app.host,
        config.app.port
    );
    let app = Application::build(config).await;
    app.run_until_stopped().await.unwrap();

    Ok(())
}
