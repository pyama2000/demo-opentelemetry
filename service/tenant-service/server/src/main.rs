use std::collections::HashMap;
use std::sync::Arc;

use proto::tenant::v1::address::NormalizationLevel;
use proto::tenant::v1::tenant_service_server::TenantServiceServer;
use proto::tenant::v1::Address;
use proto::tenant::v1::{
    tenant_service_server::TenantService, CreateTenantRequest, CreateTenantResponse,
    ListTenantsRequest, ListTenantsResponse, TENANT_SERVICE_FILE_DESCRIPTOR_SET,
};
use serde::Deserialize;
use tonic::{Request, Response};

mod config;

#[derive(Debug, Deserialize)]
struct AddressValidatorResponse {
    level: u32,
    full: String,
    pref: Option<String>,
    city: Option<String>,
    town: Option<String>,
    addr: Option<String>,
}

impl Into<Address> for AddressValidatorResponse {
    fn into(self) -> Address {
        let level = match self.level {
            1 => NormalizationLevel::Prefecture,
            2 => NormalizationLevel::City,
            3 => NormalizationLevel::Town,
            _ => NormalizationLevel::Unspecified,
        }
        .into();
        Address {
            level,
            full: self.full,
            prefecture: self.pref,
            city: self.city,
            town: self.town,
            other: self.addr,
        }
    }
}

#[derive(Debug, Clone)]
struct Tenant {
    id: ulid::Ulid,
    name: String,
    address: Address,
}

impl Tenant {
    fn new(name: String, address: Address) -> Self {
        let id = ulid::Ulid::new();
        Self { id, name, address }
    }
}

#[derive(Debug)]
struct InMemoryDatastore {
    tenants: Arc<std::sync::Mutex<HashMap<ulid::Ulid, Tenant>>>,
}

impl InMemoryDatastore {
    fn new() -> Self {
        Self {
            tenants: Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

    fn insert_tenant(&self, id: ulid::Ulid, tenant: Tenant) {
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
    address_validator_url: String,
}

impl TenantServiceImpl {
    fn new(datastore: InMemoryDatastore, address_validator_url: impl Into<String>) -> Self {
        Self {
            datastore,
            address_validator_url: address_validator_url.into(),
        }
    }
}

#[tonic::async_trait]
impl TenantService for TenantServiceImpl {
    async fn create_tenant(
        &self,
        req: Request<CreateTenantRequest>,
    ) -> Result<Response<CreateTenantResponse>, tonic::Status> {
        let req = req.into_inner();
        let result: AddressValidatorResponse = reqwest::Client::new()
            .get(format!(
                "{}/address/{}",
                &self.address_validator_url, req.address
            ))
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        let tenant = Tenant::new(req.name, result.into());
        let id = tenant.id;
        self.datastore.insert_tenant(id, tenant);
        let res = CreateTenantResponse {
            id: Some(proto::lib::v1::Ulid {
                value: id.to_string(),
            }),
        };
        Ok(Response::new(res))
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
    let address_validator_url = format!(
        "http://{}:{}",
        &config.address_validator_host, &config.address_validator_port
    );

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(TENANT_SERVICE_FILE_DESCRIPTOR_SET)
        .build()?;
    let tenant_service = TenantServiceServer::new(TenantServiceImpl::new(
        InMemoryDatastore::new(),
        address_validator_url,
    ));

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
