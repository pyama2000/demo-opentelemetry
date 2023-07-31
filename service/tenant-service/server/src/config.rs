const TENANT_SERVICE_PORT_KEY: &str = "TENANT_SERVICE_PORT";
const ADDRESS_VALIDATOR_HOST_KEY: &str = "ADDRESS_VALIDATOR_HOST";
const ADDRESS_VALIDATOR_PORT_KEY: &str = "ADDRESS_VALIDATOR_PORT";
const OPEN_TELEMETRY_SCHEMA_URL_KEY: &str = "OPEN_TELEMETRY_SCHEMA_URL";
const OPEN_TELEMETRY_ENDPOINT_KEY: &str = "OPEN_TELEMETRY_ENDPOINT";

pub struct Config {
    pub port: u32,
    pub otel: OpenTelemetry,
    pub address_validator_host: String,
    pub address_validator_port: String,
}

impl Config {
    pub fn from_env() -> Self {
        let port = from_env(TENANT_SERVICE_PORT_KEY).parse().unwrap();
        let address_validator_host = from_env(ADDRESS_VALIDATOR_HOST_KEY);
        let address_validator_port = from_env(ADDRESS_VALIDATOR_PORT_KEY);
        let otel = OpenTelemetry::from_env();
        Self {
            port,
            otel,
            address_validator_host,
            address_validator_port,
        }
    }
}

pub struct OpenTelemetry {
    pub schema_url: String,
    pub endpoint: String,
}

impl OpenTelemetry {
    fn from_env() -> Self {
        Self {
            schema_url: from_env(OPEN_TELEMETRY_SCHEMA_URL_KEY),
            endpoint: from_env(OPEN_TELEMETRY_ENDPOINT_KEY),
        }
    }
}

fn from_env(key: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| panic!("{} must be set", key))
}
