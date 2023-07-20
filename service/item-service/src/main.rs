use axum::{http::StatusCode, routing::get, Router};
use opentelemetry_otlp::WithExportConfig as _;
use tower_http::{
    catch_panic::CatchPanicLayer,
    trace::{self, TraceLayer},
};
use tracing::Level;
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let tracer = init_tracer()?;
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer())
        .with(
            tracing_subscriber::EnvFilter::builder()
                .with_default_directive(tracing_subscriber::filter::LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .try_init()?;

    let app = Router::new()
        .route("/healthz", get(|| async { StatusCode::OK }))
        .route("/panic", get(|| async { panic!("panic occured") }))
        .route(
            "/error",
            get(|| async {
                tracing::error!("error occured");
                StatusCode::INTERNAL_SERVER_ERROR
            }),
        )
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_request(trace::DefaultOnRequest::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(CatchPanicLayer::new());

    let port = std::env::var("ITEM_SERVICE_PORT")
        .unwrap_or_else(|_| panic!("ITEM_SERVICE_PORT must be set"));
    let addr = format!("0.0.0.0:{}", port).parse()?;

    tracing::info!("ItemService listening on: {}", &addr);
    hyper::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .unwrap_or_else(|e| panic!("failed to install Ctrl+C handler: {}", e))
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .unwrap_or_else(|e| panic!("failed to install singal handler: {}", e))
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {tracing::info!("receive ctrl_c signal")},
        _ = terminate => {tracing::info!("receive terminate")},
    }

    tracing::info!("signal received, starting graceful shutdown");
}

fn init_tracer() -> Result<opentelemetry_sdk::trace::Tracer, opentelemetry_api::trace::TraceError> {
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint("http://localhost:4317"),
        )
        .with_trace_config(opentelemetry_sdk::trace::config().with_resource(
            opentelemetry_sdk::Resource::new(vec![opentelemetry_api::KeyValue::new(
                "service.name",
                "item-service",
            )]),
        ))
        .install_batch(opentelemetry_sdk::runtime::Tokio)
}
