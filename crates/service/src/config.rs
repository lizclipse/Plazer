use std::{env, fmt, fs, path::Path};

use anyhow::Context as _;
use cfg_if::cfg_if;
use name_variant::NamedVariant;
use ring::signature::{self, KeyPair as _};
use serde::{Deserialize, Serialize};
use tracing::Level;

// Defaults

pub static DEFAULT_ADDRESS: &str = "file:./data/db";
pub static DEFAULT_NAMESPACE: &str = "plazer";
pub static DEFAULT_DATABASE: &str = "plazer";
pub static DEFAULT_PRIVATE_KEY_PATH: &str = "./data/private_key.pem";
pub static DEFAULT_LOG_DIR: &str = "./data/logs";

cfg_if! {

if #[cfg(debug_assertions)] {
    pub static DEFAULT_LOG_LEVEL_STDOUT: LogLevel = LogLevel::Debug;
    pub static DEFAULT_LOG_LEVEL_FILE: LogLevel = LogLevel::Trace;
} else {
    pub static DEFAULT_LOG_LEVEL_STDOUT: LogLevel = LogLevel::Info;
    pub static DEFAULT_LOG_LEVEL_FILE: LogLevel = LogLevel::Info;
}

}

pub static DEFAULT_HOST: &str = "0.0.0.0";
pub const DEFAULT_PORT: u16 = 8080;

pub static DEFAULT_CONFIG_PATH: &str = "./config.toml";

// Env vars

pub static ENV_VAR_ADDRESS: &str = "PLAZER_DB_ADDRESS";
pub static ENV_VAR_NAMESPACE: &str = "PLAZER_DB_NAMESPACE";
pub static ENV_VAR_DATABASE: &str = "PLAZER_DB_DATABASE";
pub static ENV_VAR_PRIVATE_KEY: &str = "PLAZER_PRIVATE_KEY";
pub static ENV_VAR_PRIVATE_KEY_PATH: &str = "PLAZER_PRIVATE_KEY_PATH";
pub static ENV_VAR_LOG_DIR: &str = "PLAZER_LOG_DIR";
pub static ENV_VAR_LOG_LEVEL_STDOUT: &str = "PLAZER_LOG_LEVEL_STDOUT";
pub static ENV_VAR_LOG_LEVEL_FILE: &str = "PLAZER_LOG_LEVEL_FILE";
pub static ENV_VAR_HOST: &str = "PLAZER_HOST";
pub static ENV_VAR_PORT: &str = "PLAZER_PORT";

// Config

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, NamedVariant,
)]
#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    /// Only show errors
    Error,
    /// Show errors and warnings
    Warn,
    /// Show info and above
    Info,
    /// Show debug and above
    Debug,
    /// Show all logs
    Trace,
}

impl fmt::Display for LogLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.variant_name().to_ascii_lowercase())
    }
}

impl From<LogLevel> for tracing::Level {
    fn from(level: LogLevel) -> Self {
        match level {
            LogLevel::Error => Level::ERROR,
            LogLevel::Warn => Level::WARN,
            LogLevel::Info => Level::INFO,
            LogLevel::Debug => Level::DEBUG,
            LogLevel::Trace => Level::TRACE,
        }
    }
}

pub type PrivateKeyCreate = fn(&Path) -> anyhow::Result<String>;

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceConfigBuilder {
    address: Option<String>,
    namespace: Option<String>,
    database: Option<String>,
    private_key: Option<String>,
    private_key_path: Option<String>,
    #[serde(skip)]
    private_key_create: Option<PrivateKeyCreate>,
    log_dir: Option<String>,
    log_level_stdout: Option<LogLevel>,
    log_level_file: Option<LogLevel>,
    host: Option<String>,
    port: Option<u16>,
}

impl ServiceConfigBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn address(mut self, address: impl Into<String>) -> Self {
        self.address = Some(address.into());
        self
    }

    #[must_use]
    pub fn set_address(mut self, address: Option<String>) -> Self {
        self.address = address;
        self
    }

    #[must_use]
    pub fn namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = Some(namespace.into());
        self
    }

    #[must_use]
    pub fn set_namespace(mut self, namespace: Option<String>) -> Self {
        self.namespace = namespace;
        self
    }

    #[must_use]
    pub fn database(mut self, database: impl Into<String>) -> Self {
        self.database = Some(database.into());
        self
    }

    #[must_use]
    pub fn set_database(mut self, database: Option<String>) -> Self {
        self.database = database;
        self
    }

    #[must_use]
    pub fn private_key(mut self, private_key: impl Into<String>) -> Self {
        self.private_key = Some(private_key.into());
        self
    }

    #[must_use]
    pub fn set_private_key(mut self, private_key: Option<String>) -> Self {
        self.private_key = private_key;
        self
    }

    #[must_use]
    pub fn private_key_path(mut self, private_key_path: impl Into<String>) -> Self {
        self.private_key_path = Some(private_key_path.into());
        self
    }

    #[must_use]
    pub fn set_private_key_path(mut self, private_key_path: Option<String>) -> Self {
        self.private_key_path = private_key_path;
        self
    }

    #[must_use]
    pub fn private_key_create(mut self, private_key_create: PrivateKeyCreate) -> Self {
        self.private_key_create = Some(private_key_create);
        self
    }

    #[must_use]
    pub fn set_private_key_create(mut self, private_key_create: Option<PrivateKeyCreate>) -> Self {
        self.private_key_create = private_key_create;
        self
    }

    #[must_use]
    pub fn log_dir(mut self, log_dir: impl Into<String>) -> Self {
        self.log_dir = Some(log_dir.into());
        self
    }

    #[must_use]
    pub fn set_log_dir(mut self, log_dir: Option<String>) -> Self {
        self.log_dir = log_dir;
        self
    }

    #[must_use]
    pub fn log_level_stdout(mut self, log_level_stdout: impl Into<LogLevel>) -> Self {
        self.log_level_stdout = Some(log_level_stdout.into());
        self
    }

    #[must_use]
    pub fn set_log_level_stdout(mut self, log_level_stdout: Option<LogLevel>) -> Self {
        self.log_level_stdout = log_level_stdout;
        self
    }

    #[must_use]
    pub fn log_level_file(mut self, log_level_file: impl Into<LogLevel>) -> Self {
        self.log_level_file = Some(log_level_file.into());
        self
    }

    #[must_use]
    pub fn set_log_level_file(mut self, log_level_file: Option<LogLevel>) -> Self {
        self.log_level_file = log_level_file;
        self
    }

    #[must_use]
    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(host.into());
        self
    }

    #[must_use]
    pub fn set_host(mut self, host: Option<String>) -> Self {
        self.host = host;
        self
    }

    #[must_use]
    pub fn port(mut self, port: impl Into<u16>) -> Self {
        self.port = Some(port.into());
        self
    }

    #[must_use]
    pub fn set_port(mut self, port: Option<u16>) -> Self {
        self.port = port;
        self
    }

    pub fn build(self) -> anyhow::Result<ServiceConfig> {
        let file_config = match fs::read_to_string(DEFAULT_CONFIG_PATH) {
            Ok(file_config) => toml::from_str(&file_config).context("Config invalid")?,
            Err(err) => {
                if let std::io::ErrorKind::NotFound = err.kind() {
                    ServiceConfigBuilder::default()
                } else {
                    return Err(err).context("Unable to read config file");
                }
            }
        };

        Ok(ServiceConfig {
            address: config_str_value(
                self.address,
                ENV_VAR_ADDRESS,
                file_config.address,
                DEFAULT_ADDRESS,
            )?,
            namespace: config_str_value(
                self.namespace,
                ENV_VAR_NAMESPACE,
                file_config.namespace,
                DEFAULT_NAMESPACE,
            )?,
            database: config_str_value(
                self.database,
                ENV_VAR_DATABASE,
                file_config.database,
                DEFAULT_DATABASE,
            )?,
            private_key: match self.private_key {
                Some(private_key) => Some(private_key),
                None => env_value(ENV_VAR_PRIVATE_KEY)?.or(file_config.private_key),
            },
            private_key_path: config_str_value(
                self.private_key_path,
                ENV_VAR_PRIVATE_KEY_PATH,
                file_config.private_key_path,
                DEFAULT_PRIVATE_KEY_PATH,
            )?,
            private_key_create: self.private_key_create,
            log_dir: config_str_value(
                self.log_dir,
                ENV_VAR_LOG_DIR,
                file_config.log_dir,
                DEFAULT_LOG_DIR,
            )?,
            log_level_stdout: config_level_value(
                self.log_level_stdout,
                ENV_VAR_LOG_LEVEL_STDOUT,
                file_config.log_level_stdout,
                DEFAULT_LOG_LEVEL_STDOUT,
            )?,
            log_level_file: config_level_value(
                self.log_level_file,
                ENV_VAR_LOG_LEVEL_FILE,
                file_config.log_level_file,
                DEFAULT_LOG_LEVEL_FILE,
            )?,
            host: config_str_value(self.host, ENV_VAR_HOST, file_config.host, DEFAULT_HOST)?,
            port: config_parsed_value(self.port, ENV_VAR_PORT, file_config.port, DEFAULT_PORT)?,
        })
    }
}

fn config_str_value(
    arg: Option<String>,
    env_var: &str,
    file: Option<String>,
    default: &str,
) -> anyhow::Result<String> {
    let value = match arg {
        Some(arg) => arg,
        None => env_value(env_var)?
            .or(file)
            .unwrap_or_else(|| default.to_owned()),
    };

    Ok(value)
}

fn config_parsed_value<T>(
    arg: Option<T>,
    env_var: &str,
    file: Option<T>,
    default: T,
) -> anyhow::Result<T>
where
    T: std::str::FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    let value = match arg {
        Some(arg) => arg,
        None => match env_value(env_var)? {
            Some(value) => value.parse().with_context(|| {
                format!("Invalid value `{value}` in environment variable {env_var}")
            })?,
            None => file.unwrap_or(default),
        },
    };

    Ok(value)
}

fn config_level_value(
    arg: Option<LogLevel>,
    env_var: &str,
    file: Option<LogLevel>,
    default: LogLevel,
) -> anyhow::Result<LogLevel> {
    let value = match arg {
        Some(arg) => arg,
        None => match env_value(env_var)? {
            Some(value) => match &*value.to_ascii_lowercase() {
                "error" => LogLevel::Error,
                "warn" => LogLevel::Warn,
                "info" => LogLevel::Info,
                "debug" => LogLevel::Debug,
                "trace" => LogLevel::Trace,
                value => {
                    return Err(anyhow::anyhow!(
                        "Invalid log level {value:?} in environment variable {env_var}"
                    ))
                }
            },
            None => file.unwrap_or(default),
        },
    };

    Ok(value)
}

fn env_value(env_var: &str) -> anyhow::Result<Option<String>> {
    match env::var(env_var) {
        Ok(value) => Ok(Some(value)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(_)) => Err(anyhow::anyhow!(
            "Environment variable {env_var} is not valid unicode",
        )),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceConfig {
    address: String,
    namespace: String,
    database: String,
    private_key: Option<String>,
    private_key_path: String,
    #[serde(skip)]
    private_key_create: Option<PrivateKeyCreate>,
    log_dir: String,
    log_level_stdout: LogLevel,
    log_level_file: LogLevel,
    host: String,
    port: u16,
}

impl TryFrom<ServiceConfig> for (ServeConfig, LogConfig) {
    type Error = anyhow::Error;

    fn try_from(value: ServiceConfig) -> Result<Self, Self::Error> {
        let private_key = if let Some(private_key) = value.private_key {
            private_key
        } else {
            match fs::read_to_string(&value.private_key_path) {
                Ok(private_key) => private_key,
                Err(err) => {
                    if let std::io::ErrorKind::NotFound = err.kind() {
                        match value.private_key_create {
                            Some(create) => create(value.private_key_path.as_ref())?,
                            None => {
                                return Err(anyhow::anyhow!(
                                    "Private key not found at {}",
                                    value.private_key_path
                                ))
                            }
                        }
                    } else {
                        return Err(err).context("Unable to read private key");
                    }
                }
            }
        };

        let (enc_key, dec_key) = create_key_pair(&private_key)?;

        let serve_config = ServeConfig {
            address: value.address,
            namespace: value.namespace,
            database: value.database,
            jwt_enc_key: enc_key,
            jwt_dec_key: dec_key,
            host: value.host,
            port: value.port,
        };

        let log_config = LogConfig {
            dir: value.log_dir,
            level_stdout: value.log_level_stdout.into(),
            level_file: value.log_level_file.into(),
        };

        Ok((serve_config, log_config))
    }
}

#[derive(Clone)]
pub struct ServeConfig {
    pub address: String,
    pub namespace: String,
    pub database: String,
    pub jwt_enc_key: jsonwebtoken::EncodingKey,
    pub jwt_dec_key: jsonwebtoken::DecodingKey,
    pub host: String,
    pub port: u16,
}

pub struct LogConfig {
    pub dir: String,
    pub level_stdout: Level,
    pub level_file: Level,
}

pub fn create_key_pair(
    pem: &str,
) -> anyhow::Result<(jsonwebtoken::EncodingKey, jsonwebtoken::DecodingKey)> {
    let (_, doc) = pkcs8::Document::from_pem(pem)
        .map_err(|err| anyhow::anyhow!("Failed to parse private key: {err:?}"))?;
    let key_pair = signature::Ed25519KeyPair::from_pkcs8(doc.as_ref())?;
    let enc_key =
        jsonwebtoken::EncodingKey::from_ed_pem(pem.as_bytes()).context("Private key is invalid")?;
    let dec_key = jsonwebtoken::DecodingKey::from_ed_der(key_pair.public_key().as_ref());

    Ok((enc_key, dec_key))
}
