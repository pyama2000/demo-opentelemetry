use opentelemetry_otlp::WithExportConfig as _;
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};

pub fn init(
    otel_schema_url: &str,
    otel_endpoint: &str,
) -> Result<impl Fn(), Box<dyn std::error::Error>> {
    opentelemetry::global::set_text_map_propagator(
        opentelemetry::sdk::propagation::TraceContextPropagator::new(),
    );
    let tracer = init_tracer(otel_schema_url, otel_endpoint)?;
    let metrics = init_metrics(otel_endpoint)?;
    init_subscriber(tracer, metrics)?;
    Ok(|| opentelemetry::global::shutdown_tracer_provider())
}

fn init_subscriber(
    tracer: opentelemetry::sdk::trace::Tracer,
    metrics: opentelemetry::sdk::metrics::controllers::BasicController,
) -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(tracing_subscriber::filter::LevelFilter::INFO)
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .with(tracing_opentelemetry::MetricsLayer::new(metrics))
        .try_init()?;
    Ok(())
}

fn init_tracer(
    otel_schema_url: impl Into<String>,
    otel_endpoint: impl Into<String>,
) -> Result<opentelemetry::sdk::trace::Tracer, opentelemetry::trace::TraceError> {
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
                    otel_schema_url.into(),
                )),
        )
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(otel_endpoint),
        )
        .install_batch(opentelemetry::sdk::runtime::Tokio)
}

// NOTE: metrics を送るには info を特定の形にする必要がある
// read mores: https://blog.ymgyt.io/entry/starting_opentelemetry_with_rust/#prometheus
fn init_metrics(
    otel_endpoint: impl Into<String>,
) -> Result<
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
                .with_endpoint(otel_endpoint),
        )
        .build()
}
