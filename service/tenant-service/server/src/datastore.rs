use std::collections::HashMap;
use std::sync::Arc;

pub struct InMemory {
    tenants: Arc<std::sync::Mutex<HashMap<ulid::Ulid, crate::service::tenant::model::Tenant>>>,
}

impl InMemory {
    pub fn new() -> Self {
        Self {
            tenants: Arc::new(std::sync::Mutex::new(HashMap::new())),
        }
    }

    pub fn insert_tenant(&self, id: ulid::Ulid, tenant: crate::service::tenant::model::Tenant) {
        let mut tenants = self.tenants.lock().unwrap();
        tenants.insert(id, tenant);
    }

    pub fn list_tenant(&self) -> Vec<crate::service::tenant::model::Tenant> {
        let tenants = self.tenants.lock().unwrap();
        tenants.values().cloned().collect()
    }
}
