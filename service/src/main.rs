use std::sync::Arc;
use tonic::transport::Server;
use tracing::info;

use grpc_sky_api::proto::{
    post_service_server::PostServiceServer, user_service_server::UserServiceServer,
    FILE_DESCRIPTOR_SET,
};
use grpc_sky_service::{
    config::{AppConfig, DatabaseConfig, MySqlDatabaseConfig},
    error::AppResult,
    post::{
        adapter::PostAdapter, create_command::CreatePostCommand, query_manager::PostQueryManager,
        repository::mysql::PostMySqlRepository,
    },
    tracing::tracer::Tracer,
    user::{
        adapter::UserAdapter, repository::mysql::UserMySqlRepository,
        sign_up_command::SignUpCommand,
    },
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

    let (user_repository, post_repository) = match &config.database {
        DatabaseConfig::MySql(MySqlDatabaseConfig { connection }) => {
            let pool = mysql_async::Pool::new(connection.as_str());
            let user_repository = Arc::new(UserMySqlRepository::new(pool.clone()));
            let post_repository = Arc::new(PostMySqlRepository::new(pool));
            (user_repository, post_repository)
        }
        DatabaseConfig::Memory => todo!(),
    };

    let sign_up_command = SignUpCommand::new(user_repository.clone());
    let user_adapter = UserAdapter::new(user_repository, sign_up_command);

    let create_post_command = CreatePostCommand::new(post_repository.clone());
    let post_query_manager = PostQueryManager::new(post_repository);
    let post_adapter = PostAdapter::new(create_post_command, post_query_manager);

    Server::builder()
        .add_service(
            tonic_reflection::server::Builder::configure()
                .register_encoded_file_descriptor_set(FILE_DESCRIPTOR_SET)
                // Some clients only work with v1alpha
                .build_v1alpha()
                .unwrap(),
        )
        .add_service(UserServiceServer::new(user_adapter))
        .add_service(PostServiceServer::new(post_adapter))
        .serve(config.server.grpc_address)
        .await?;

    Ok(())
}
