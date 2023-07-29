use std::collections::HashMap;
use std::sync::Arc;

use proto::tenant::v1::address::NormalizationLevel;
use proto::tenant::v1::tenant_service_server::TenantServiceServer;
use proto::tenant::v1::{
    tenant_service_server::TenantService, CreateTenantRequest, CreateTenantResponse,
    ListTenantsRequest, ListTenantsResponse, TENANT_SERVICE_FILE_DESCRIPTOR_SET,
};
use tonic::{Request, Response};

mod config;

#[derive(Clone)]
struct TenantAddress {
    level: NormalizationLevel,
    full: String,
    prefecture: Option<String>,
    city: Option<String>,
    town: Option<String>,
    other: Option<String>,
}

#[derive(Clone)]
struct Tenant {
    id: ulid::Ulid,
    name: String,
    address: TenantAddress,
}

struct InMemoryDatastore {
    tenants: Arc<std::sync::Mutex<HashMap<ulid::Ulid, Tenant>>>,
}

impl InMemoryDatastore {
    fn new() -> Self {
        Self {
            tenants: Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

    fn insert(&self, id: ulid::Ulid, tenant: Tenant) {
        let mut tenants = self.tenants.lock().unwrap();
        tenants.insert(id, tenant);
    }

    fn get(&self, id: ulid::Ulid) -> Option<Tenant> {
        let tenants = self.tenants.lock().unwrap();
        match tenants.get(&id) {
            Some(t) => Some(t.clone()),
            None => None,
        }
    }
}

struct TenantServiceImpl {
    datastore: InMemoryDatastore,
}

impl TenantServiceImpl {
    fn new(datastore: InMemoryDatastore) -> Self {
        Self { datastore }
    }
}

#[tonic::async_trait]
impl TenantService for TenantServiceImpl {
    async fn create_tenant(
        &self,
        _: Request<CreateTenantRequest>,
    ) -> Result<Response<CreateTenantResponse>, tonic::Status> {
        todo!()
    }

    async fn list_tenants(
        &self,
        _: Request<ListTenantsRequest>,
    ) -> Result<Response<ListTenantsResponse>, tonic::Status> {
        todo!()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let config = config::Config::from_env();

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(TENANT_SERVICE_FILE_DESCRIPTOR_SET)
        .build()?;
    let tenant_service = TenantServiceServer::new(TenantServiceImpl::new(InMemoryDatastore::new()));

    let addr = format!("0.0.0.0:{}", &config.port).parse()?;
    tracing::info!("TenentService listening on: {}", &addr);
    tonic::transport::Server::builder()
        .add_service(reflection_service)
        .add_service(tenant_service)
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
