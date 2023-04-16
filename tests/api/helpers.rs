use std::sync::{Arc, Mutex};

use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use tokio;
use uuid::Uuid;
use wiremock::MockServer;

use pic_scraper_backend::config::{get_configuration, DatabaseSettings, SourcesUrls};
use pic_scraper_backend::startup::{AppState, Application};
use pic_scraper_backend::telemetry::{get_subscriber, init_subscriber};

// Ensure that the `tracing` stack is only initialised once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    // We cannot assign the output of `get_subscriber` to a variable based on the value of `TEST_LOG`
    // because the sink is part of the type returned by `get_subscriber`, therefore they are not the
    // same type. We could work around it, but this is the most straight-forward way of moving forward.
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

pub struct TestApp {
    pub addr: String,
    pub port: u16,
    pub mock_server: MockServer,
    pub state: AppState,
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let mock_server = MockServer::builder().start().await;
    let sources_urls = SourcesUrls {
        pixiv: format!("{}/pixiv", mock_server.uri()),
        pixiv_details: format!("{}/pixiv_details", mock_server.uri()),
        pixiv_image: format!("{}/pixiv_image", mock_server.uri()),
        bcy: format!("{}/bcy", mock_server.uri()),
        twitter_home: format!("{}/twitter_home", mock_server.uri()),
        twitter_honkai: format!("{}/twitter_honkai", mock_server.uri()),
        mihoyo: format!("{}/mihoyo", mock_server.uri()),
        lofter: format!("{}/lofter", mock_server.uri()),
    };

    let config = {
        let mut c = get_configuration().expect("Failed to read config");

        c.database.database_name = Uuid::new_v4().to_string();
        c.app.port = 0;
        c.app.sources_urls = sources_urls;
        c
    };

    let db_pool = configure_db(&config.database).await;

    let app = Application::build(config.clone()).await;
    let port = app.port;
    let addr = format!("http://127.0.0.1:{}", port);

    let state = AppState {
        db_pool,
        api_client: Application::create_api_client(&config)
            .expect("Failed to create the api client"),
        blacklist: config.app.blacklist,
        sources_urls: config.app.sources_urls,
        last_update_time: Arc::new(Mutex::new(0)),
    };

    tokio::spawn(app.run_until_stopped());

    TestApp {
        addr,
        port,
        mock_server,
        state,
    }
}

async fn configure_db(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}

pub fn assert_is_redirect_to(response: &reqwest::Response, location: &str) {
    assert_eq!(response.status().as_u16(), 303);
    assert_eq!(response.headers().get("Location").unwrap(), location);
}
