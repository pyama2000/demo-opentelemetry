pub fn reflection_service() -> Result<
    tonic_reflection::server::ServerReflectionServer<
        impl tonic_reflection::server::ServerReflection,
    >,
    tonic_reflection::server::Error,
> {
    tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(proto::tenant::v1::TENANT_SERVICE_FILE_DESCRIPTOR_SET)
        .build()
}
