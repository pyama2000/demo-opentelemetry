use std::collections::HashMap;
use std::sync::Arc;

mod config;
mod service;

#[derive(Debug)]
pub struct InMemoryDatastore {
    tenants: Arc<std::sync::Mutex<HashMap<ulid::Ulid, service::tenant::model::Tenant>>>,
}

impl InMemoryDatastore {
    fn new() -> Self {
        Self {
            tenants: Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

    fn insert_tenant(&self, id: ulid::Ulid, tenant: service::tenant::model::Tenant) {
        let mut tenants = self.tenants.lock().unwrap();
        tenants.insert(id, tenant);
    }

    fn list_tenant(&self) -> Vec<service::tenant::model::Tenant> {
        let tenants = self.tenants.lock().unwrap();
        tenants.values().cloned().collect()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let config = config::Config::from_env();
    let address_validator_url = format!(
        "http://{}:{}",
        &config.address_validator_host, &config.address_validator_port
    );

    let addr = format!("0.0.0.0:{}", &config.port).parse()?;
    tracing::info!("TenentService listening on: {}", &addr);
    tonic::transport::Server::builder()
        .add_service(service::reflection::reflection_service()?)
        .add_service(service::tenant::tenant_service(
            InMemoryDatastore::new(),
            address_validator_url,
        ))
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;
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
