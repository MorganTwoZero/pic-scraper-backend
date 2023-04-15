use std::net::TcpListener;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use axum::extract::{State, FromRef};
use axum::{routing::get, Router};
use chrono::{DateTime, Utc};
use delay_timer::prelude::{DelayTimerBuilder, TaskBuilder};
use hyper::{header, Server};
use reqwest::Client;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tower_http::services::{ServeDir, ServeFile};
use tower_http::trace::TraceLayer;

use crate::config::{BlackList, DatabaseSettings, Settings, SourcesUrls};
use crate::embed::embed;
use crate::errors::Error;
use crate::etl::{fill_db, load_honkai_posts, load_twitter_home_posts};
use crate::utils::proxy_image_route;

#[derive(Clone)]
pub struct AppState {
    pub db_pool: PgPool,
    pub api_client: Client,
    pub blacklist: BlackList,
    pub sources_urls: SourcesUrls,
    pub last_update_time: Arc<Mutex<i64>>,
}

#[derive(Clone)]
pub struct StateWrapper(Arc<AppState>);

impl FromRef<StateWrapper> for AppState {
    fn from_ref(wrapper: &StateWrapper) -> AppState {
        wrapper.0.clone().into()
    }
}

impl From<Arc<AppState>> for AppState {
    fn from(arc: Arc<AppState>) -> Self {
        (*arc).clone()
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
            .serve(self.router.into_make_service())
            .await
    }

    pub fn create_api_client() -> Result<Client, Error> {
        let headers = vec![...];
        let mut headers_map = header::HeaderMap::new();
        for (key, val) in headers {
            let mut val = header::HeaderValue::from_static(val);
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
                fill_db(&state).await?;
                *state.last_update_time.lock().unwrap() = chrono::Utc::now().timestamp();
                Ok::<(), Error>(())
            }
        };
    
        let initial_task = task_builder
            .set_task_id(0)
            .set_frequency_once_by_seconds(2)
            .set_maximum_parallel_runnable_num(1)
            .spawn_async_routine(body.clone());
    
        let task = task_builder
            .set_task_id(1)
            .set_frequency_repeated_by_cron_str("0 */20 * * * *")
            .set_maximum_parallel_runnable_num(1)
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
        let serve_dir =
            ServeDir::new("frontend").not_found_service(ServeFile::new("frontend/index.html"));

        Router::new()
            .route("/api/update/last_update", get(last_update))
            .route("/api/honkai", get(load_honkai_posts))
            .route("/api/myfeed", get(load_twitter_home_posts))
            .route("/api/jpg", get(proxy_image_route))
            .route("/en/artworks/:path", get(embed))
            .nest_service("/", serve_dir.clone())
            .fallback_service(serve_dir)
            // Enables logging. Use `RUST_LOG=tower_http=debug`
            .layer(TraceLayer::new_for_http())
            .with_state(state)
    }

    pub async fn build(config: Settings) -> Self {
        let db_pool = Self::get_connection_pool(&config.database);
        sqlx::migrate!()
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
            api_client: Self::create_api_client().expect("Failed to create the api client"),
            blacklist: config.app.blacklist,
            sources_urls: config.app.sources_urls,
            last_update_time: Arc::new(Mutex::new(0)),
        });
        let router = Self::create_router(StateWrapper(state.clone())).await;

        Self {
            port,
            router,
            listener,
            state
        }
    }
}

async fn last_update(State(state): State<AppState>) -> String {
    DateTime::<Utc>::from_utc(
        chrono::NaiveDateTime::from_timestamp_opt(*state.last_update_time.lock().unwrap(), 0)
            .unwrap(),
        Utc,
    )
    .to_rfc3339()
}