mod client;
mod config;
mod datastore;
mod observe;
mod service;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::Config::from_env();
    let shutdown_tracer = observe::init(&config.otel.schema_url, &config.otel.endpoint)
        .unwrap_or_else(|e| panic!("failed to init observer: {}", e));

    let address_validator_url = format!(
        "http://{}:{}",
        &config.address_validator_host, &config.address_validator_port
    );

    let addr = format!("0.0.0.0:{}", &config.port).parse()?;
    tracing::info!("TenentService listening on: {}", &addr);
    tonic::transport::Server::builder()
        .layer(observe::middleware::trace_layer())
        .layer(tower_http::catch_panic::CatchPanicLayer::new())
        .add_service(service::reflection::reflection_service()?)
        .add_service(service::tenant::tenant_service(
            datastore::InMemory::new(),
            client::Client::new(),
            address_validator_url,
        ))
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;

    shutdown_tracer();
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
