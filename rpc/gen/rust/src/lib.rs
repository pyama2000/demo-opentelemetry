pub mod lib {
    pub mod v1 {
        tonic::include_proto!("lib.v1");
        pub const LIB_FILE_DESCRIPTOR_SET: &[u8] =
            tonic::include_file_descriptor_set!("lib_descriptor");
    }
}

pub mod tenant {
    pub mod v1 {
        tonic::include_proto!("tenant.v1");
        pub const TENANT_SERVICE_FILE_DESCRIPTOR_SET: &[u8] =
            tonic::include_file_descriptor_set!("tenant_service_descriptor");
    }
}
