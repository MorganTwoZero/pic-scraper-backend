use std::net::TcpListener;
use std::time::Duration;

use axum::{routing::get, Router};
use hyper::{header, Server};
use reqwest::{Client, Error};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

use crate::config::{DatabaseSettings, Settings, BlackList};
use crate::etl::{fill_db, load_honkai_posts};
use crate::utils::proxy_image;

pub fn create_request_client() -> Result<Client, Error> {
    let headers = vec![("cookie", "__utmv=235335808.|2=login ever=no=1^9=p_ab_id=3=1^10=p_ab_id_2=8=1^11=lang=en=1; PHPSESSID=37028420_4PABfDM1JsDaGbTR1FVeZpB9abuTEwkq; auth_token=269bcbc47c601743694a83bf4d78306dd6a6f168; ct0=bd586d27427110176ba725b0a89d363e1683254ff3b05992815e58d80f8ee96de172a713b043d21c060a146da4ebcc2c0583c5b0916b9f797bc4bf3c41950873cf31f67d63d962cda77107e537d6be66"), ("x-user-id", "37028420"), ("authorization", "Bearer AAAAAAAAAAAAAAAAAAAAAF7aAAAAAAAASCiRjWvh7R5wxaKkFp7MM%2BhYBqM%3DbQ0JPmjU9F6ZoMhDfI4uTNAaQuTDm2uO9x3WFVr2xBZ2nhjdP0"), ("x-csrf-token", "bd586d27427110176ba725b0a89d363e1683254ff3b05992815e58d80f8ee96de172a713b043d21c060a146da4ebcc2c0583c5b0916b9f797bc4bf3c41950873cf31f67d63d962cda77107e537d6be66"), ("user-agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64)"), ("Referer", "https://www.pixiv.net/")];
    let mut headers_map = header::HeaderMap::new();
    for (key, val) in headers {
        let mut val = header::HeaderValue::from_static(val);
        val.set_sensitive(true);
        headers_map.insert(key, val);
    }
    Ok(Client::builder()
        .timeout(Duration::from_secs(20))
        .default_headers(headers_map)
        .build())?
}

#[derive(Clone)]
pub struct AppContext {
    pub db_pool: PgPool,
    pub reqwest_client: Client,
    pub blacklist: BlackList,
}

pub struct Application {
    pub port: u16,
    app: Router,
    listener: TcpListener,
}

impl Application {
    pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
        PgPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_secs(2))
            .connect_lazy_with(configuration.with_db())
    }

    pub async fn run_until_stopped(self) -> Result<(), hyper::Error> {
        Server::from_tcp(self.listener)
            .expect("Failed to bind a TcpListener")
            .serve(self.app.into_make_service())
            .await
    }

    pub async fn create_app(db_pool: PgPool, config: Settings) -> Router {
        let context = AppContext {
            db_pool,
            reqwest_client: create_request_client().unwrap(),
            blacklist: config.app.blacklist
        };

        let serve_dir =
            ServeDir::new("frontend").not_found_service(ServeFile::new("frontend/index.html"));

        Router::new()
            .route(
                "/api/update/last_update",
                get(|| async { "2023-03-11T12:47:01" }),
            ) // TODO
            .route("/api/honkai", get(load_honkai_posts))
            .route("/api/update", get(fill_db))
            .route("/api/jpg", get(proxy_image))
            .nest_service("/", serve_dir.clone())
            .fallback_service(serve_dir)
            // Enables logging. Use `RUST_LOG=tower_http=debug`
            .layer(TraceLayer::new_for_http())
            .with_state(context)
    }

    pub async fn build(config: Settings) -> Self {
        let db_pool = get_connection_pool(&config.database);
        sqlx::migrate!()
            .run(&db_pool)
            .await
            .expect("Failed to run migrations");
        let addr = format!("{}:{}", config.app.host, config.app.port);
        let listener = TcpListener::bind(&addr).expect("Failed to create a TcpListener");
        let port = listener.local_addr().unwrap().port();
        let app = Self::create_app(db_pool, config).await;
        Self {
            port,
            app,
            listener,
        }
    }
}

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db())
}
