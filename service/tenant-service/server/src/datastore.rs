use std::collections::HashMap;
use std::sync::Arc;

pub struct InMemory {
    tenants: Arc<tokio::sync::Mutex<HashMap<ulid::Ulid, crate::service::tenant::model::Tenant>>>,
}

impl InMemory {
    pub fn new() -> Self {
        Self {
            tenants: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
        }
    }

    pub async fn insert_tenant(
        &self,
        id: ulid::Ulid,
        tenant: crate::service::tenant::model::Tenant,
    ) {
        let mut tenants = self.tenants.lock().await;
        tenants.insert(id, tenant);
    }

    pub async fn list_tenant(&self) -> Vec<crate::service::tenant::model::Tenant> {
        let tenants = self.tenants.lock().await;
        tenants.values().cloned().collect()
    }
}
