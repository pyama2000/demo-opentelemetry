use tracing_opentelemetry::OpenTelemetrySpanExt as _;

pub mod model;

pub fn tenant_service(
    datastore: crate::datastore::InMemory,
    address_validator_url: impl Into<String>,
) -> proto::tenant::v1::tenant_service_server::TenantServiceServer<TenantService> {
    proto::tenant::v1::tenant_service_server::TenantServiceServer::new(TenantService::new(
        datastore,
        address_validator_url,
    ))
}

#[derive(Debug)]
pub struct TenantService {
    datastore: crate::datastore::InMemory,
    address_validator_url: String,
}

impl TenantService {
    pub fn new(
        datastore: crate::datastore::InMemory,
        address_validator_url: impl Into<String>,
    ) -> Self {
        Self {
            datastore,
            address_validator_url: address_validator_url.into(),
        }
    }
}

#[tonic::async_trait]
impl proto::tenant::v1::tenant_service_server::TenantService for TenantService {
    #[tracing::instrument]
    async fn create_tenant(
        &self,
        req: tonic::Request<proto::tenant::v1::CreateTenantRequest>,
    ) -> Result<tonic::Response<proto::tenant::v1::CreateTenantResponse>, tonic::Status> {
        let req = req.into_inner();
        let client = reqwest_middleware::ClientBuilder::new(reqwest::Client::new())
            .with(reqwest_tracing::TracingMiddleware::<
                reqwest_tracing::SpanBackendWithUrl,
            >::new())
            .build();

        let mut headers = http::HeaderMap::new();
        opentelemetry::global::get_text_map_propagator(|p| {
            p.inject_context(&tracing::Span::current().context(), &mut opentelemetry_http::HeaderInjector(&mut headers))
        });

        let result: model::AddressValidatorResponse = client
            .get(format!(
                "{}/address/{}",
                &self.address_validator_url, req.address
            ))
            .headers(headers)
            .send()
            .await
            .map_err(|e| {
                tracing::error!("{}", e.to_string());
                tonic::Status::unknown(e.to_string())
            })?
            .json()
            .await
            .map_err(|e| {
                tracing::error!("{}", e.to_string());
                tonic::Status::internal(e.to_string())
            })?;
        let tenant = model::Tenant::new(req.name, result.into());
        let id = tenant.id;
        self.datastore.insert_tenant(id, tenant).await;
        let res = proto::tenant::v1::CreateTenantResponse {
            id: Some(proto::lib::v1::Ulid {
                value: id.to_string(),
            }),
        };
        Ok(tonic::Response::new(res))
    }

    #[tracing::instrument]
    async fn list_tenants(
        &self,
        _: tonic::Request<proto::tenant::v1::ListTenantsRequest>,
    ) -> Result<tonic::Response<proto::tenant::v1::ListTenantsResponse>, tonic::Status> {
        let tenants = self.datastore.list_tenants().await;
        let tenants: Vec<proto::tenant::v1::list_tenants_response::Tenant> =
            tenants.into_iter().map(|t| t.into()).collect();
        let res = proto::tenant::v1::ListTenantsResponse {
            tenants,
            next_page_token: String::new(),
        };
        Ok(tonic::Response::new(res))
    }
}
