use std::{
    net::{SocketAddr, TcpListener},
    sync::{Arc, Mutex},
    time::Duration,
};

use axum::{extract::FromRef, routing::get, Router};
use delay_timer::prelude::{DelayTimerBuilder, TaskBuilder};

use hyper::{header, Server};
use reqwest::Client;
use secrecy::ExposeSecret;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::signal;
use tower_http::services::{ServeDir, ServeFile};

use crate::{
    embed::{embed, proxy_image_route},
    site_routes::{last_update, like},
    telemetry::opentelemetry_tracing_layer,
    Error,
};
use config_structs::{AppState, DatabaseSettings, Settings};
use etl::{fill_db, load_honkai_posts, load_twitter_home_posts};

#[derive(Clone)]
pub struct StateWrapper(Arc<AppState>);

impl FromRef<StateWrapper> for AppState {
    fn from_ref(wrapper: &StateWrapper) -> AppState {
        wrapper.0.clone().into()
    }
}

pub struct Application {
    pub port: u16,
    router: Router,
    listener: TcpListener,
    state: Arc<AppState>,
}

impl Application {
    pub async fn run_until_stopped(self) -> Result<(), hyper::Error> {
        Server::from_tcp(self.listener)
            .expect("Failed to bind a TcpListener")
            .serve(
                self.router
                    .into_make_service_with_connect_info::<SocketAddr>(),
            )
            .with_graceful_shutdown(shutdown_signal())
            .await
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

    pub async fn create_fill_db_task(&self) -> Result<(), Error> {
        let state = self.state.clone();
        let mut task_builder = TaskBuilder::default();

        let body = move || {
            let state = state.clone();
            async move {
                match fill_db(&state).await {
                    Err(e) => tracing::error!("Failed to fill db. Err: {:?}", e),
                    Ok(_) => tracing::info!("DB filled"),
                };
                *state.last_update_time.lock().unwrap() = chrono::Utc::now().timestamp();
                Ok::<(), Error>(())
            }
        };

        let initial_task = task_builder
            .set_task_id(0)
            .set_frequency_once_by_seconds(1)
            .set_maximum_parallel_runnable_num(1)
            .set_maximum_running_time(30)
            .spawn_async_routine(body.clone());

        let task = task_builder
            .set_task_id(1)
            .set_frequency_repeated_by_cron_str("0 */20 * * * *")
            .set_maximum_parallel_runnable_num(1)
            .set_maximum_running_time(30)
            .spawn_async_routine(body);

        let timer = DelayTimerBuilder::default().build();
        timer.insert_task(initial_task?)?;
        timer.insert_task(task?)?;
        Ok(())
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
            .layer(opentelemetry_tracing_layer())
            .with_state(state)
    }

    pub async fn build(config: Settings) -> Self {
        let db_pool = Self::get_connection_pool(&config.database);
        sqlx::migrate!("../migrations")
            .run(&db_pool)
            .await
            .expect("Failed to run migrations");

        let addr = format!("{}:{}", config.app.host, config.app.port);
        let listener = TcpListener::bind(&addr).expect("Failed to create a TcpListener");
        let port = listener
            .local_addr()
            .expect("Failed to read the listener's address")
            .port();

        let state = Arc::new(AppState {
            db_pool,
            api_client: Self::create_api_client(&config).expect("Failed to create the api client"),
            blacklist: config.app.blacklist,
            sources_urls: config.app.sources_urls,
            last_update_time: Arc::new(Mutex::new(0)),
        });
        let router = Self::create_router(StateWrapper(state.clone())).await;

        Self {
            port,
            router,
            listener,
            state,
        }
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("signal received, starting graceful shutdown");

    opentelemetry::global::shutdown_tracer_provider();
}
