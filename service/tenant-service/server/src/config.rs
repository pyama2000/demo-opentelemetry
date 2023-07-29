pub struct Config {
    pub port: u32,
    pub address_validator_host: String,
    pub address_validator_port: String,
}

impl Config {
    pub fn from_env() -> Self {
        let port = std::env::var("TENANT_SERVICE_PORT")
            .unwrap_or_else(|_| panic!("TENANT_SERVICE_PORT must be set"))
            .parse()
            .unwrap();
        let address_validator_host = std::env::var("ADDRESS_VALIDATOR_HOST")
            .unwrap_or_else(|_| panic!("ADDRESS_VALIDATOR_HOST must be set"));
        let address_validator_port = std::env::var("ADDRESS_VALIDATOR_PORT")
            .unwrap_or_else(|_| panic!("ADDRESS_VALIDATOR_PORT must be set"));
        Self {
            port,
            address_validator_host,
            address_validator_port,
        }
    }
}
