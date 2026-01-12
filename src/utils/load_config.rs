use crate::utils::load_env::load_env;
use anyhow::{Context, Result};
use config::{Config, Environment, File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub app: AppSection,
    pub client_integrations: ClientIntegrationsSection,
    pub observability: ObservabilitySection,

    // Optional / currently commented-out sections
    pub server: Option<ServerSection>,
    // pub database: Option<DatabaseSection>,
    // pub auth: Option<AuthSection>,
    // pub security: Option<SecuritySection>,
}

#[derive(Debug, Deserialize)]
pub struct AppSection {
    pub name: String,

    // Commented out → optional
    pub environment: Option<String>,
    // pub log_level: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ClientIntegrationsSection {
    #[serde(default)]
    pub allow_access_middleware: bool,

    #[serde(default)]
    pub allow_sessions_middleware: bool,

    #[serde(default)]
    pub allow_logging_middleware: bool,

    #[serde(default)]
    pub allow_request_timeout_middleware: bool,

    #[serde(default)]
    pub allow_admin_routes_protector_middleware: bool,
}

#[derive(Debug, Deserialize)]
pub struct ObservabilitySection {
    pub enable_tracing: bool,
    pub enable_metrics: bool,
}

#[derive(Debug, Deserialize)]
pub struct ServerSection {
    pub host: String,
    pub port: u16,
    // pub graceful_shutdown_secs: u64,
}

// #[derive(Debug, Deserialize)]
// pub struct DatabaseSection {
//     pub engine: String,
//     pub host: String,
//     pub port: u16,
//     pub name: String,
//     pub max_connections: u32,
//     pub connect_timeout_secs: u64,
// }

// #[derive(Debug, Deserialize)]
// pub struct AuthSection {
//     pub access_token_ttl_secs: u64,
//     pub refresh_token_ttl_secs: u64,
// }

// #[derive(Debug, Deserialize)]
// pub struct SecuritySection {
//     pub bcrypt_cost: u32,
//     pub rate_limit_per_minute: u32,
// }

pub fn load_config() -> Result<AppConfig> {
    load_env();

    let env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".into());

    let builder = Config::builder()
        .add_source(File::with_name("config/base").required(true))
        .add_source(File::with_name(&format!("config/{}", env)).required(false))
        .add_source(File::with_name("config/local").required(false))
        .add_source(
            Environment::default()
                .separator("__")
                .prefix("APP")
                .try_parsing(true),
        );

    /**************** EXPLAINING THE MAPPING RULE FOR THE [ABOVE] FINAL ENV OVERRIDES ****************
    /**************** EXPLAINING THE MAPPING RULE FOR THE [ABOVE] FINAL ENV OVERRIDES ****************

    # Mapping Rule (exact)

    APP_<SECTION>__<FIELD>=value

    Lowercase / uppercase differences are normalized.

    So this TOML:

    [server]
    port = 8080

    will be overridden by:

    APP_SERVER__PORT=9000


    If the names don’t align, nothing happens.

    Example (❌ no override):

    SERVER_PORT=9000

    This does nothing unless you explicitly read it in code.

    **************** EXPLAINING THE MAPPING RULE FOR THE [ABOVE] FINAL ENV OVERRIDES ****************/
    **************** EXPLAINING THE MAPPING RULE FOR THE [ABOVE] FINAL ENV OVERRIDES ****************/

    builder
        .build()
        .context("Failed to build config")?
        .try_deserialize()
        .context("Invalid config shape")
}

impl AppConfig {
    pub fn validate(&self) -> Result<()> {
        if self.app.name.trim().is_empty() {
            anyhow::bail!("app.name cannot be empty");
        }

        if let Some(server) = &self.server {
            if server.port == 0 {
                anyhow::bail!("server.port cannot be 0");
            }
        }

        Ok(())
    }
}
