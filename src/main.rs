use pic_scraper_backend::{
    config::get_configuration, startup::Application, telemetry::setup_telemetry,
};

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
    app.create_fill_db_task().await.unwrap();
    app.run_until_stopped().await.unwrap();

    Ok(())
}
