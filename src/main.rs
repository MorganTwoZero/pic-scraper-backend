use pic_scraper_backend::config::get_configuration;
use pic_scraper_backend::startup::Application;
use pic_scraper_backend::telemetry::{get_subscriber, init_subscriber};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = get_subscriber("pic_scraper_backend".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let config = get_configuration().unwrap();
    println!(
        "Server running on http://{}:{}",
        config.app.host, config.app.port
    );
    let app = Application::build(config).await;
    app.create_fill_db_task().await.unwrap();
    app.run_until_stopped().await.unwrap();

    Ok(())
}
