const OPEN_TELEMETRY_SCHEMA_URL_KEY: &str = "OPEN_TELEMETRY_SCHEMA_URL";
const OPEN_TELEMETRY_ENDPOINT_KEY: &str = "OPEN_TELEMETRY_ENDPOINT";

pub struct Config {
    pub otel: OpenTelemetry,
}

impl Config {
    pub fn from_env() -> Self {
        Self {
            otel: OpenTelemetry::from_env(),
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
