use std::net::SocketAddr;

use axum::{routing::get, Router};
use hyper::Server;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

use crate::config::{DatabaseSettings, Settings};
use crate::etl::{get_honkai_posts_from_db, fill_db};

pub struct Application {
    addr: SocketAddr,
    pub port: u16,
    app: Router,
}

impl Application {
    pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
        PgPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_secs(2))
            .connect_lazy_with(configuration.with_db())
    }

    pub async fn run_until_stopped(self) -> Result<(), hyper::Error> {
        Server::bind(&self.addr)
            .serve(self.app.into_make_service())
            .await
    }

    pub async fn create_app(db_pool: PgPool) -> Router {
        let serve_dir =
            ServeDir::new("frontend").not_found_service(ServeFile::new("frontend/index.html"));

        Router::new()
            .route(
                "/api/update/last_update",
                get(|| async { "2023-03-11T12:47:01" }),
            ) // TODO
            .route("/api/honkai", get(get_honkai_posts_from_db))
            .route("/api/update", get(fill_db))
            .nest_service("/", serve_dir.clone())
            .fallback_service(serve_dir)
            // Enables logging. Use `RUST_LOG=tower_http=debug`
            .layer(TraceLayer::new_for_http())
            .with_state(db_pool)
    }

    pub fn parse_addr(config: &Settings) -> SocketAddr {
        format!("{}:{}", config.app.host, config.app.port)
            .parse::<SocketAddr>()
            .unwrap()
    }

    pub async fn build(config: Settings) -> Self {
        let db_pool = get_connection_pool(&config.database);
        sqlx::migrate!().run(&db_pool).await.expect("Failed to run migrations");
        let addr = Self::parse_addr(&config);
        let port = addr.port();
        let app = Self::create_app(db_pool).await;
        Self { addr, port, app }
    }
}

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db())
}
