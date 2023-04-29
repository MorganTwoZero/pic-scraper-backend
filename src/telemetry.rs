use std::str::FromStr;

use opentelemetry_otlp::WithExportConfig;
use tracing::Subscriber;
use tracing_subscriber::{
    fmt::format::FmtSpan, layer::SubscriberExt, prelude::*, registry::LookupSpan, Layer,
};

pub fn build_logger_text<S>() -> Box<dyn Layer<S> + Send + Sync + 'static>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    if cfg!(debug_assertions) {
        Box::new(
            tracing_subscriber::fmt::layer()
                .pretty()
                .with_line_number(true)
                .with_thread_names(true)
                .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                .with_timer(tracing_subscriber::fmt::time::uptime()),
        )
    } else {
        Box::new(
            tracing_subscriber::fmt::layer()
                .json()
                .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
                .with_timer(tracing_subscriber::fmt::time::uptime()),
        )
    }
}

pub fn setup_telemetry() {
    let tracer = create_otlp_tracer();
    let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(build_logger_text())
        .with(telemetry_layer)
        .init();
}

fn create_otlp_tracer() -> opentelemetry::sdk::trace::Tracer {
    let protocol = std::env::var("OTEL_EXPORTER_OTLP_PROTOCOL").unwrap_or("grpc".to_string());

    let mut tracer = opentelemetry_otlp::new_pipeline().tracing();
    let headers = parse_otlp_headers_from_env();

    match protocol.as_str() {
        "grpc" => {
            let mut exporter = opentelemetry_otlp::new_exporter()
                .tonic()
                .with_metadata(metadata_from_headers(headers))
                .with_env();

            // Check if we need TLS
            if let Ok(endpoint) = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT") {
                if endpoint.starts_with("https") {
                    exporter = exporter.with_tls_config(Default::default());
                }
            }
            tracer = tracer.with_exporter(exporter)
        }
        "http/protobuf" => {
            let exporter = opentelemetry_otlp::new_exporter()
                .http()
                .with_headers(headers.into_iter().collect())
                .with_env();
            tracer = tracer.with_exporter(exporter)
        }
        p => panic!("Unsupported protocol {}", p),
    };

    tracer.install_batch(opentelemetry::runtime::Tokio).unwrap()
}

fn metadata_from_headers(headers: Vec<(String, String)>) -> tonic::metadata::MetadataMap {
    use tonic::metadata;

    let mut metadata = metadata::MetadataMap::new();
    headers.into_iter().for_each(|(name, value)| {
        let value = value
            .parse::<metadata::MetadataValue<metadata::Ascii>>()
            .expect("Header value invalid");
        metadata.insert(metadata::MetadataKey::from_str(&name).unwrap(), value);
    });
    metadata
}

fn parse_otlp_headers_from_env() -> Vec<(String, String)> {
    let mut headers = Vec::new();

    if let Ok(hdrs) = std::env::var("OTEL_EXPORTER_OTLP_HEADERS") {
        hdrs.split(',')
            .map(|header| {
                header
                    .split_once('=')
                    .expect("Header should contain '=' character")
            })
            .for_each(|(name, value)| headers.push((name.to_owned(), value.to_owned())));
    }
    headers
}
