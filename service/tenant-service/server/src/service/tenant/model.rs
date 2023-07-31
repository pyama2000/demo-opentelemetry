#[derive(Debug, serde::Deserialize)]
pub struct AddressValidatorResponse {
    level: u32,
    full: String,
    pref: Option<String>,
    city: Option<String>,
    town: Option<String>,
    addr: Option<String>,
}

impl Into<proto::tenant::v1::Address> for AddressValidatorResponse {
    fn into(self) -> proto::tenant::v1::Address {
        let level = match self.level {
            1 => proto::tenant::v1::address::NormalizationLevel::Prefecture,
            2 => proto::tenant::v1::address::NormalizationLevel::City,
            3 => proto::tenant::v1::address::NormalizationLevel::Town,
            _ => proto::tenant::v1::address::NormalizationLevel::Unspecified,
        }
        .into();
        proto::tenant::v1::Address {
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
pub struct Tenant {
    pub id: ulid::Ulid,
    name: String,
    address: proto::tenant::v1::Address,
}

impl Tenant {
    pub fn new(name: String, address: proto::tenant::v1::Address) -> Self {
        let id = ulid::Ulid::new();
        Self { id, name, address }
    }
}

impl Into<proto::tenant::v1::list_tenants_response::Tenant> for Tenant {
    fn into(self) -> proto::tenant::v1::list_tenants_response::Tenant {
        let id = Some(proto::lib::v1::Ulid {
            value: self.id.to_string(),
        });
        proto::tenant::v1::list_tenants_response::Tenant {
            id,
            name: self.name,
            address: Some(self.address),
        }
    }
}
