use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use tokio;
use uuid::Uuid;

use pic_scraper_backend::config::{get_configuration, DatabaseSettings, BlackList};
use pic_scraper_backend::etl::extract::create_vec_posts;
use pic_scraper_backend::etl::save_honkai_posts;
use pic_scraper_backend::startup::{create_request_client, Application};
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
    pub db_pool: PgPool,
    pub port: u16,
    pub api_client: reqwest::Client,
    pub blacklist: BlackList,
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let config = {
        let mut c = get_configuration().expect("Failed to read config");

        c.database.database_name = Uuid::new_v4().to_string();
        c.app.port = 0;
        c
    };

    let blacklist = config.app.blacklist.clone();

    let db_pool = configure_db(&config.database).await;

    let app = Application::build(config).await;
    let port = app.port;
    let addr = format!("http://127.0.0.1:{}", port);

    let api_client = create_request_client().unwrap();

    tokio::spawn(app.run_until_stopped());

    let test_app = TestApp {
        addr,
        db_pool,
        port,
        api_client,
        blacklist,
    };
    test_app
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

#[tokio::test]
async fn honkai_get_returns_200() {
    let app = spawn_app().await;
    let addr = format!("{}/api/honkai", &app.addr);
    let r = app.api_client.get(&addr).send().await.unwrap();
    assert_eq!(r.status().as_u16(), 200);
}

#[tokio::test]
async fn update_returns_200() {
    let app = spawn_app().await;
    let addr = format!("{}/api/update", &app.addr);
    let r = app.api_client.get(&addr).send().await.unwrap();
    assert_eq!(r.status().as_u16(), 200);
}

#[tokio::test]
async fn test_request_and_parse_response() {
    let app = spawn_app().await;
    let posts = create_vec_posts(&app.api_client, &app.blacklist).await.unwrap();
    save_honkai_posts(&app.db_pool, posts).await.unwrap();
}
