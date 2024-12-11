use std::{error::Error, path::PathBuf};

use bomboni_prost::config::{ApiConfig, CompileConfig};

fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let fd_path = out_dir.join("sky_fd.bin");

    let root_path = PathBuf::from("./proto");
    let proto_paths: Vec<_> = [
        "sky/resources.proto",
        "sky/user_service.proto",
        "sky/post_service.proto",
        "sky/error/error.proto",
    ]
    .into_iter()
    .map(|proto_path| root_path.join(proto_path))
    .collect();

    let mut prost_config = prost_build::Config::default();
    prost_config
        .file_descriptor_set_path(&fd_path)
        .type_name_domain(&["."], "example.com")
        .enable_type_names()
        .protoc_arg("--experimental_allow_proto3_optional");

    // Regression? https://github.com/tokio-rs/prost/issues/932
    for type_name in ["Timestamp"] {
        prost_config.extern_path(
            format!(".google.protobuf.{type_name}"),
            format!("::bomboni_proto::google::protobuf::{type_name}"),
        );
    }

    tonic_build::configure()
        // .file_descriptor_set_path(&fd_path)
        // .protoc_arg("--experimental_allow_proto3_optional")
        // .extern_path(".google", "::bomboni::proto::google")
        .compile_protos_with_config(prost_config, &proto_paths, &["./proto"])
        .unwrap();

    bomboni_prost::compile(CompileConfig {
        api: ApiConfig {
            domain: Some("example.com".into()),
            helpers_mod: Some("helpers".into()),
            ..Default::default()
        },
        file_descriptor_set_path: fd_path,
        ..Default::default()
    })?;

    for proto_path in &proto_paths {
        println!("cargo:rerun-if-changed={}", proto_path.display());
    }

    Ok(())
}
