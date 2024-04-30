use serde_derive::{Deserialize,Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub redis: RedisConfig,
    pub postgres: PostgresConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RedisConfig {
    pub host: String,
    pub port: u16,
    pub db: u16,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PostgresConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub db_name: String,
}

pub async fn load_config() -> Config {
    let config = std::fs::read_to_string("config.toml").expect("Failed to read config file");
    toml::from_str(&config).expect("Failed to parse config file")
}

pub async fn redis_config() -> RedisConfig {
    let config = load_config().await;
    config.redis
}

pub async fn postgres_config() -> PostgresConfig {
    let config = load_config().await;
    config.postgres
}

pub async fn db_connection_string() -> String {
    let config = postgres_config().await;
    format!(
        "postgresql://{}:{}@{}:{}/{}",
        config.user, config.password, config.host, config.port, config.db_name
    )
}

pub async fn redis_connection_string() -> String {
    let config = redis_config().await;
    format!("redis://{}:{}/{}", config.host, config.port, config.db)
}