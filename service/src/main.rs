use tonic::transport::Server;
use tracing::info;

use grpc_sky_api::proto::{user_service_server::UserServiceServer, FILE_DESCRIPTOR_SET};
use grpc_sky_service::{
    config::AppConfig, error::AppResult, tracing::tracer::Tracer, user::adapter::UserAdapter,
};

#[tokio::main]
async fn main() -> AppResult<()> {
    let config = AppConfig::get();

    Tracer::install_stdout()?;

    info!(
        "Starting {} v{}",
        config.distribution.name, config.distribution.version
    );
    info!("Listening gRPC on {}", config.server.grpc_address);

    let user_adapter = UserAdapter::new();

    Server::builder()
        .add_service(
            tonic_reflection::server::Builder::configure()
                .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
                // Some clients only work with v1alpha
                .build_v1alpha()
                .unwrap(),
        )
        .add_service(UserServiceServer::new(user_adapter))
        .serve(config.server.grpc_address)
        .await?;

    Ok(())
}
