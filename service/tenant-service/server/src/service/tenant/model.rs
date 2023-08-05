#[derive(Debug, serde::Deserialize)]
pub struct AddressValidatorResponse {
    level: u32,
    full: String,
    pref: Option<String>,
    city: Option<String>,
    town: Option<String>,
    addr: Option<String>,
}

impl Into<Address> for AddressValidatorResponse {
    fn into(self) -> Address {
        let normalized_address = match self.level {
            1 => Some(NormalizedAddress::Prefecture {
                prefecture: self.pref.unwrap(),
                other: self.addr.unwrap(),
            }),
            2 => Some(NormalizedAddress::City {
                prefecture: self.pref.unwrap(),
                city: self.city.unwrap(),
                other: self.addr.unwrap(),
            }),
            3 => Some(NormalizedAddress::Town {
                prefecture: self.pref.unwrap(),
                city: self.city.unwrap(),
                town: self.town.unwrap(),
                other: self.addr.unwrap(),
            }),
            _ => None,
        };
        Address {
            full: self.full,
            normalized_address,
        }
    }
}

#[derive(Debug, Clone)]
pub enum NormalizedAddress {
    Prefecture {
        prefecture: String,
        other: String,
    },
    City {
        prefecture: String,
        city: String,
        other: String,
    },
    Town {
        prefecture: String,
        city: String,
        town: String,
        other: String,
    },
}

#[derive(Debug, Clone)]
pub struct Address {
    full: String,
    normalized_address: Option<NormalizedAddress>,
}

impl Into<proto::tenant::v1::Address> for Address {
    fn into(self) -> proto::tenant::v1::Address {
        if self.normalized_address.is_none() {
            return proto::tenant::v1::Address {
                level: proto::tenant::v1::address::NormalizationLevel::NotNomalized.into(),
                full: self.full,
                prefecture: None,
                city: None,
                town: None,
                other: None,
            };
        }
        match self.normalized_address.unwrap() {
            NormalizedAddress::Prefecture { prefecture, other } => proto::tenant::v1::Address {
                level: proto::tenant::v1::address::NormalizationLevel::Prefecture.into(),
                full: self.full,
                prefecture: Some(prefecture),
                city: None,
                town: None,
                other: Some(other),
            },
            NormalizedAddress::City {
                prefecture,
                city,
                other,
            } => proto::tenant::v1::Address {
                level: proto::tenant::v1::address::NormalizationLevel::City.into(),
                full: self.full,
                prefecture: Some(prefecture),
                city: Some(city),
                town: None,
                other: Some(other),
            },
            NormalizedAddress::Town {
                prefecture,
                city,
                town,
                other,
            } => proto::tenant::v1::Address {
                level: proto::tenant::v1::address::NormalizationLevel::Town.into(),
                full: self.full,
                prefecture: Some(prefecture),
                city: Some(city),
                town: Some(town),
                other: Some(other),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct Tenant {
    pub id: ulid::Ulid,
    name: String,
    address: Address,
}

impl Tenant {
    pub fn new(name: String, address: Address) -> Self {
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
            address: Some(self.address.into()),
        }
    }
}
