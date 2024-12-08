use std::{error::Error, path::PathBuf};

fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());

    let root_path = PathBuf::from("./proto");
    let proto_paths: Vec<_> = ["sky/user.proto", "sky/error/error.proto"]
        .into_iter()
        .map(|proto_path| root_path.join(proto_path))
        .collect();

    tonic_build::configure()
        .file_descriptor_set_path(out_dir.join("sky_fd.bin"))
        .protoc_arg("--experimental_allow_proto3_optional")
        .extern_path(".google", "::bomboni::proto::google")
        .compile_protos(&proto_paths, &["./proto"])
        .unwrap();

    for proto_path in &proto_paths {
        println!("cargo:rerun-if-changed={}", proto_path.display());
    }

    Ok(())
}
