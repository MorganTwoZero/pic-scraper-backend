pub fn setup_telemetry() {
    init_tracing_opentelemetry::tracing_subscriber_ext::init_subscribers().unwrap();
}

pub fn shutdown_tracing() {
    opentelemetry::global::shutdown_tracer_provider();
}
