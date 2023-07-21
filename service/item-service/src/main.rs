use axum::{http::StatusCode, routing::get, Router};
use tower_http::catch_panic::CatchPanicLayer;

mod config;
mod observe;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::Config::from_env();
    let shutdown_tracer = observe::init(&config.otel.schema_url, &config.otel.endpoint)
        .unwrap_or_else(|e| panic!("failed to init observer: {}", e));

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
        .route("/span", get(span))
        .layer(observe::trace_layer())
        .layer(CatchPanicLayer::new());

    let port = std::env::var("ITEM_SERVICE_PORT")
        .unwrap_or_else(|_| panic!("ITEM_SERVICE_PORT must be set"));
    let addr = format!("0.0.0.0:{}", port).parse()?;

    tracing::info!("ItemService listening on: {}", &addr);
    hyper::Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    shutdown_tracer();

    Ok(())
}

#[tracing::instrument]
async fn span() {
    tracing::event!(observe::LOG_LEVEL, "sleep event");
    tokio::join!(sleep_500ms(), sleep_1500ms());
}

#[tracing::instrument]
async fn sleep_500ms() {
    tracing::info!("start");
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    tracing::info!("finish");
}

#[tracing::instrument]
async fn sleep_1500ms() {
    tracing::info!("start");
    tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;
    tracing::info!("finish");
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
