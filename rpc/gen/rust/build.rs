const PROTO_ROOT_DIR: &str = "../../proto";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR")?);
    tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .file_descriptor_set_path(out_dir.join("lib_descriptor.bin"))
        .compile(
            &[
                format!("{}/lib/v1/id.proto", PROTO_ROOT_DIR),
            ],
            &[PROTO_ROOT_DIR],
        )?;
    tonic_build::configure()
        .protoc_arg("--experimental_allow_proto3_optional")
        .file_descriptor_set_path(out_dir.join("tenant_service_descriptor.bin"))
        .compile(
            &[format!("{}/tenant/v1/tenant_service.proto", PROTO_ROOT_DIR)],
            &[PROTO_ROOT_DIR],
        )?;
    Ok(())
}
