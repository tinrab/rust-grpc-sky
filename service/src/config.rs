use std::path::Path;
use std::sync::OnceLock;
use std::{env, net::SocketAddr};

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub distribution: DistributionConfig,
    pub server: ServerConfig,
    pub database: DatabaseConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ServerConfig {
    pub grpc_address: SocketAddr,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DistributionConfig {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind")]
pub enum DatabaseConfig {
    Memory,
    MySql(MySqlDatabaseConfig),
}

#[derive(Debug, Clone, Deserialize)]
pub struct MySqlDatabaseConfig {
    pub connection: String,
}

const VERSION: &str = env!("CARGO_PKG_VERSION");
const CONFIG_PATH_ENV: &str = "FLINECT_PLATFORM_CONFIG_PATH";
const ENV_PREFIX: &str = "FLINECT_PLATFORM";
const CONFIG_PATH: &str = "config";
const DISTRIBUTION_VERSION_KEY: &str = "distribution.version";

impl AppConfig {
    pub fn get() -> &'static Self {
        static INSTANCE: OnceLock<AppConfig> = OnceLock::new();
        INSTANCE.get_or_init(|| Self::load().unwrap())
    }

    fn load() -> Result<Self, ConfigError> {
        let mut config_builder =
            Config::builder().set_default(DISTRIBUTION_VERSION_KEY, VERSION)?;

        // Initial "default" configuration file
        let default_path = Path::new(CONFIG_PATH).join("default");
        config_builder = config_builder.add_source(File::with_name(default_path.to_str().unwrap()));

        // Add in a local configuration file
        // This file shouldn't be checked in to git
        let local_path = Path::new(CONFIG_PATH).join("local");
        config_builder = config_builder
            .add_source(File::with_name(local_path.to_str().unwrap()).required(false));

        // Add override settings file.
        let override_path = env::var(CONFIG_PATH_ENV).ok();
        if let Some(override_path) = override_path {
            config_builder =
                config_builder.add_source(File::with_name(&override_path).required(false));
        }

        // Add in settings from the environment (with a prefix of APP)
        config_builder =
            config_builder.add_source(Environment::with_prefix(ENV_PREFIX).separator("__"));

        let config = config_builder.build()?;

        Config::builder()
            .add_source(config)
            .build()?
            .try_deserialize()
    }
}
