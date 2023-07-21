use opentelemetry_otlp::WithExportConfig as _;
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};

pub const LOG_LEVEL: tracing::Level = tracing::Level::INFO;

pub fn init() -> Result<impl Fn(), Box<dyn std::error::Error>> {
    opentelemetry::global::set_text_map_propagator(
        opentelemetry::sdk::propagation::TraceContextPropagator::new(),
    );
    init_subscriber()?;
    Ok(|| opentelemetry::global::shutdown_tracer_provider())
}

pub fn trace_layer() -> tower_http::trace::TraceLayer<
    tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsAsFailures>,
> {
    tower_http::trace::TraceLayer::new_for_http()
        .make_span_with(tower_http::trace::DefaultMakeSpan::new().level(LOG_LEVEL))
        .on_request(tower_http::trace::DefaultOnRequest::new().level(LOG_LEVEL))
        .on_response(tower_http::trace::DefaultOnResponse::new().level(LOG_LEVEL))
}

fn init_subscriber() -> Result<(), Box<dyn std::error::Error>> {
    let tracer = init_tracer()?;
    let metrics = init_metrics()?;
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::filter::LevelFilter::INFO)
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .with(tracing_opentelemetry::MetricsLayer::new(metrics))
        .try_init()?;
    Ok(())
}

fn init_tracer() -> Result<opentelemetry::sdk::trace::Tracer, opentelemetry::trace::TraceError> {
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_trace_config(
            opentelemetry::sdk::trace::config()
                .with_id_generator(opentelemetry::sdk::trace::RandomIdGenerator::default())
                .with_resource(opentelemetry::sdk::Resource::from_schema_url(
                    [
                        opentelemetry::KeyValue::new(
                            opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                            env!("CARGO_PKG_NAME"),
                        ),
                        opentelemetry::KeyValue::new(
                            opentelemetry_semantic_conventions::resource::SERVICE_VERSION,
                            env!("CARGO_PKG_VERSION"),
                        ),
                    ],
                    "https://opentelemetry.io/schemas/1.20.0",
                )),
        )
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317"),
        )
        .install_batch(opentelemetry::sdk::runtime::Tokio)
}

// NOTE: metrics を送るには info を特定の形にする必要がある
// read mores: https://blog.ymgyt.io/entry/starting_opentelemetry_with_rust/#prometheus
fn init_metrics() -> Result<
    opentelemetry::sdk::metrics::controllers::BasicController,
    opentelemetry::metrics::MetricsError,
> {
    opentelemetry_otlp::new_pipeline()
        .metrics(
            opentelemetry::sdk::metrics::selectors::simple::inexpensive(),
            opentelemetry::sdk::export::metrics::aggregation::cumulative_temporality_selector(),
            opentelemetry::sdk::runtime::Tokio,
        )
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317"),
        )
        .build()
}
