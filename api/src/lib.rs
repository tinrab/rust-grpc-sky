pub mod dto;
pub mod error;

pub mod proto {
    tonic::include_proto!("sky");
    tonic::include_proto!("sky.error");

    pub const FILE_DESCRIPTOR_SET: &[u8] = tonic::include_file_descriptor_set!("sky_fd");
}
