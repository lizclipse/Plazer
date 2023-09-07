use std::{env, fs, path::Path};

use anyhow::Context as _;
use cfg_if::cfg_if;
use ring::signature::{self, KeyPair as _};
use tracing::Level;

// Defaults

pub static DEFAULT_ADDRESS: &str = "file:./data/db";
pub static DEFAULT_NAMESPACE: &str = "plazer";
pub static DEFAULT_DATABASE: &str = "plazer";
pub static DEFAULT_PRIVATE_KEY_PATH: &str = "./data/private_key.pem";
pub static DEFAULT_LOG_DIR: &str = "./data/logs";

cfg_if! {

if #[cfg(debug_assertions)] {
    pub static DEFAULT_LOG_LEVEL_STDOUT: Level = Level::DEBUG;
    pub static DEFAULT_LOG_LEVEL_FILE: Level = Level::TRACE;
} else {
    pub static DEFAULT_LOG_LEVEL_STDOUT: Level = Level::INFO;
    pub static DEFAULT_LOG_LEVEL_FILE: Level = Level::INFO;
}

}

pub static DEFAULT_HOST: &str = "0.0.0.0";
pub const DEFAULT_PORT: u16 = 8080;

// Env vars

pub static ENV_VAR_ADDRESS: &str = "PLAZER_DB_ADDRESS";
pub static ENV_VAR_NAMESPACE: &str = "PLAZER_DB_NAMESPACE";
pub static ENV_VAR_DATABASE: &str = "PLAZER_DB_DATABASE";
pub static ENV_VAR_PRIVATE_KEY: &str = "PLAZER_PRIVATE_KEY";
pub static ENV_VAR_PRIVATE_KEY_FILE: &str = "PLAZER_PRIVATE_KEY_FILE";
pub static ENV_VAR_LOG_PATH: &str = "PLAZER_LOG_PATH";
pub static ENV_VAR_LOG_LEVEL_STDOUT: &str = "PLAZER_LOG_LEVEL_STDOUT";
pub static ENV_VAR_LOG_LEVEL_FILE: &str = "PLAZER_LOG_LEVEL_FILE";
pub static ENV_VAR_HOST: &str = "PLAZER_HOST";
pub static ENV_VAR_PORT: &str = "PLAZER_PORT";

// Config

pub type PrivateKeyCreate = fn(&Path) -> anyhow::Result<String>;

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ServiceConfig {
    address: Option<String>,
    namespace: Option<String>,
    database: Option<String>,
    private_key: Option<String>,
    private_key_path: Option<String>,
    private_key_create: Option<PrivateKeyCreate>,
    log_dir: Option<String>,
    log_level_stdout: Option<Level>,
    log_level_file: Option<Level>,
    host: Option<String>,
    port: Option<u16>,
}

impl ServiceConfig {
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
    pub fn log_level_stdout(mut self, log_level_stdout: impl Into<Level>) -> Self {
        self.log_level_stdout = Some(log_level_stdout.into());
        self
    }

    #[must_use]
    pub fn set_log_level_stdout(mut self, log_level_stdout: Option<Level>) -> Self {
        self.log_level_stdout = log_level_stdout;
        self
    }

    #[must_use]
    pub fn log_level_file(mut self, log_level_file: impl Into<Level>) -> Self {
        self.log_level_file = Some(log_level_file.into());
        self
    }

    #[must_use]
    pub fn set_log_level_file(mut self, log_level_file: Option<Level>) -> Self {
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
}

impl TryFrom<ServiceConfig> for (ServeConfig, LogConfig) {
    type Error = anyhow::Error;

    fn try_from(value: ServiceConfig) -> Result<Self, Self::Error> {
        let private_key = match value.private_key {
            Some(private_key) => Some(private_key),
            None => env_value(ENV_VAR_PRIVATE_KEY)?,
        };

        let private_key = if let Some(private_key) = private_key {
            private_key
        } else {
            let private_key_path = config_str_value(
                value.private_key_path,
                ENV_VAR_PRIVATE_KEY_FILE,
                DEFAULT_PRIVATE_KEY_PATH,
            )?;

            match fs::read_to_string(&private_key_path) {
                Ok(private_key) => private_key,
                Err(err) => {
                    if let std::io::ErrorKind::NotFound = err.kind() {
                        match value.private_key_create {
                            Some(create) => create(private_key_path.as_ref())?,
                            None => {
                                return Err(anyhow::anyhow!(
                                    "Private key not found at {}",
                                    private_key_path
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
            address: config_str_value(value.address, ENV_VAR_ADDRESS, DEFAULT_ADDRESS)?,
            namespace: config_str_value(value.namespace, ENV_VAR_NAMESPACE, DEFAULT_NAMESPACE)?,
            database: config_str_value(value.database, ENV_VAR_DATABASE, DEFAULT_DATABASE)?,
            jwt_enc_key: enc_key,
            jwt_dec_key: dec_key,
            host: config_str_value(value.host, ENV_VAR_HOST, DEFAULT_HOST)?,
            port: value.port.unwrap_or(DEFAULT_PORT),
        };

        let log_config = LogConfig {
            dir: config_str_value(value.log_dir, ENV_VAR_LOG_PATH, DEFAULT_LOG_DIR)?,
            level_stdout: config_level_value(
                value.log_level_stdout,
                ENV_VAR_LOG_LEVEL_STDOUT,
                DEFAULT_LOG_LEVEL_STDOUT,
            )?,
            level_file: config_level_value(
                value.log_level_file,
                ENV_VAR_LOG_LEVEL_FILE,
                DEFAULT_LOG_LEVEL_FILE,
            )?,
        };

        Ok((serve_config, log_config))
    }
}

fn config_str_value(arg: Option<String>, env_var: &str, default: &str) -> anyhow::Result<String> {
    let value = match arg {
        Some(arg) => arg,
        None => env_value(env_var)?.unwrap_or_else(|| default.to_owned()),
    };

    Ok(value)
}

fn config_level_value(arg: Option<Level>, env_var: &str, default: Level) -> anyhow::Result<Level> {
    let value = match arg {
        Some(arg) => arg,
        None => match env_value(env_var)? {
            Some(env_var) => env_var.parse()?,
            None => default,
        },
    };

    Ok(value)
}

fn env_value(env_var: &str) -> anyhow::Result<Option<String>> {
    match env::var(env_var) {
        Ok(env_var) => Ok(Some(env_var)),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(env::VarError::NotUnicode(_)) => Err(anyhow::anyhow!(
            "Environment variable {} is not valid unicode",
            env_var
        )),
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
        .map_err(|err| anyhow::anyhow!("Failed to parse private key: {:?}", err))?;
    let key_pair = signature::Ed25519KeyPair::from_pkcs8(doc.as_ref())?;
    let enc_key =
        jsonwebtoken::EncodingKey::from_ed_pem(pem.as_bytes()).context("Private key is invalid")?;
    let dec_key = jsonwebtoken::DecodingKey::from_ed_der(key_pair.public_key().as_ref());

    Ok((enc_key, dec_key))
}
