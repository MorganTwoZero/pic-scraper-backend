use std::{sync::Arc, time::Duration};

use axum::{extract::FromRef, routing::get, Router};
use axum_tracing_opentelemetry::middleware::{OtelAxumLayer, OtelInResponseLayer};
use reqwest::{header, Client};
use secrecy::ExposeSecret;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::net::TcpListener;
use tower_http::services::{ServeDir, ServeFile};

use crate::{
    load::{load_honkai_posts, load_twitter_home_posts},
    site_routes::{last_update, like},
    Error,
};
use config_structs::{ApiState, DatabaseSettings, Settings};
use embed::{embed, proxy_image_route};

#[derive(Clone)]
pub struct StateWrapper(Arc<ApiState>);

impl FromRef<StateWrapper> for ApiState {
    fn from_ref(wrapper: &StateWrapper) -> ApiState {
        wrapper.0.clone().into()
    }
}

pub struct Application {
    pub port: u16,
    router: Router,
    listener: TcpListener,
}

impl Application {
    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        axum::serve(self.listener, self.router).await
    }

    pub fn create_api_client(config: &Settings) -> Result<Client, Error> {
        let headers = vec![
            (
                "cookie",
                config.app.headers.cookie.expose_secret().to_owned(),
            ),
            ("x-user-id", "37028420".to_owned()),
            (
                "authorization",
                config.app.headers.authorization.expose_secret().to_owned(),
            ),
            (
                "x-csrf-token",
                config.app.headers.csrf_token.expose_secret().to_owned(),
            ),
            (
                "user-agent",
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:109.0) Gecko/20100101 Firefox/111.0"
                    .to_owned(),
            ),
            ("Referer", "https://www.pixiv.net/".to_owned()),
        ];
        let mut headers_map = header::HeaderMap::new();
        for (key, val) in headers {
            let mut val = header::HeaderValue::from_str(&val)
                .unwrap_or_else(|_| panic!("Failed to parse a header: {}", val));
            val.set_sensitive(true);
            headers_map.insert(key, val);
        }
        Ok(Client::builder()
            .timeout(Duration::from_secs(10))
            .default_headers(headers_map)
            .build()?)
    }

    fn get_connection_pool(config: &DatabaseSettings) -> PgPool {
        PgPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_secs(2))
            .connect_lazy_with(config.with_db())
    }

    async fn create_router(state: StateWrapper) -> Router {
        let serve_dir = ServeDir::new("frontend/dist/")
            .not_found_service(ServeFile::new("frontend/dist/index.html"));

        Router::new()
            .route("/api/like", get(like))
            .route("/api/update/last_update", get(last_update))
            .route("/api/honkai", get(load_honkai_posts))
            .route("/api/myfeed", get(load_twitter_home_posts))
            .route("/api/jpg", get(proxy_image_route))
            .route("/en/artworks/:path", get(embed))
            .route("/en/artworks/:path/:pic_num", get(embed))
            .nest_service("/", serve_dir.clone())
            .fallback_service(serve_dir)
            .layer(OtelInResponseLayer)
            .layer(OtelAxumLayer::default())
            .with_state(state)
    }

    pub async fn build(config: Settings) -> Self {
        let db_pool = Self::get_connection_pool(&config.database);

        let addr = format!("{}:{}", config.app.host, config.app.port);
        let listener = TcpListener::bind(&addr)
            .await
            .expect("Failed to create a TcpListener");
        let port = listener
            .local_addr()
            .expect("Failed to read the listener's address")
            .port();

        let state = Arc::new(ApiState {
            db_pool,
            api_client: Self::create_api_client(&config).expect("Failed to create the api client"),
            sources_urls: config.app.sources_urls,
        });
        let router = Self::create_router(StateWrapper(state)).await;

        Self {
            port,
            router,
            listener,
        }
    }
}
