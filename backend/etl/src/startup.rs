use std::{fs, sync::Arc, time::Duration};

use axum::extract::FromRef;
use delay_timer::prelude::{DelayTimerBuilder, TaskBuilder, TaskInstancesChain};

use reqwest::{Client, header};
use secrecy::ExposeSecret;
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::signal;

use crate::{extract::fill_db, Error};
use config_structs::{DatabaseSettings, ScraperState, Settings};

#[derive(Clone)]
pub struct StateWrapper(Arc<ScraperState>);

impl FromRef<StateWrapper> for ScraperState {
    fn from_ref(wrapper: &StateWrapper) -> ScraperState {
        wrapper.0.clone().into()
    }
}

pub struct Application {
    state: Arc<ScraperState>,
}

impl Application {
    pub fn create_api_client(config: &Settings) -> Result<Client, Error> {
        let file_headers = read_headers().ok();
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
                "User-Agent",
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
        if let Some(file_headers) = file_headers {
            headers_map.extend(file_headers)
        }

        Ok(Client::builder()
            .timeout(Duration::from_secs(10))
            .redirect(reqwest::redirect::Policy::limited(30))
            .default_headers(headers_map)
            .build()?)
    }

    pub async fn run(self) -> Result<(TaskInstancesChain, TaskInstancesChain), Error> {
        let body = move || {
            let state = self.state.clone();
            async move {
                match fill_db(&state).await {
                    Err(e) => tracing::error!("Failed to fill db. Err: {:?}", e),
                    Ok(_) => tracing::info!("DB filled"),
                };
                Ok::<(), Error>(())
            }
        };

        let mut task_builder = TaskBuilder::default();

        let initial_task = task_builder
            .set_task_id(0)
            .set_frequency_once_by_seconds(1)
            .set_maximum_parallel_runnable_num(1)
            .set_maximum_running_time(30)
            .spawn_async_routine(body.clone());

        let continious_task = task_builder
            .set_task_id(1)
            .set_frequency_repeated_by_cron_str("0 */20 * * * *")
            .set_maximum_parallel_runnable_num(1)
            .set_maximum_running_time(30)
            .spawn_async_routine(body);

        let timer = DelayTimerBuilder::default().build();
        let initial_task = timer.insert_task(initial_task?)?;
        let continious_task_chain = timer.insert_task(continious_task?)?;
        Ok((initial_task, continious_task_chain))
    }

    fn get_connection_pool(config: &DatabaseSettings) -> PgPool {
        PgPoolOptions::new()
            .acquire_timeout(std::time::Duration::from_secs(2))
            .connect_lazy_with(config.with_db())
    }

    pub async fn build(config: Settings) -> Self {
        let db_pool = Self::get_connection_pool(&config.database);

        let state = Arc::new(ScraperState {
            db_pool,
            api_client: Self::create_api_client(&config).expect("Failed to create the api client"),
            blacklist: config.app.blacklist,
            sources_urls: config.app.sources_urls,
        });

        Self { state }
    }
}

fn read_headers() -> Result<header::HeaderMap, Error> {
    let contents = fs::read_to_string("headers.txt").map_err(|e| anyhow::anyhow!(e))?;

    let mut headers = header::HeaderMap::new();
    for line in contents.lines() {
        let (key, value) = line.split_once(": ").expect("Invalid header line");
        headers.insert(
            header::HeaderName::from_bytes(key.as_bytes()).map_err(|e| anyhow::anyhow!(e))?,
            header::HeaderValue::from_str(value).map_err(|e| anyhow::anyhow!(e))?,
        );
    }

    Ok(headers)
}

pub async fn shutdown_signal() {
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

    tele::shutdown_tracing();
}
