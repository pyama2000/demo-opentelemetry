use proto::tenant::v1::TENANT_SERVICE_FILE_DESCRIPTOR_SET;

mod config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = config::Config::from_env();

    let reflection_service = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(TENANT_SERVICE_FILE_DESCRIPTOR_SET)
        .build()?;

    let addr = format!("0.0.0.0:{}", &config.port).parse()?;
    tonic::transport::Server::builder()
        .add_service(reflection_service)
        .serve(addr)
        .await?;
    Ok(())
}
