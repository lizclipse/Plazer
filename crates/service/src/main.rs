use std::env;

use cfg_if::cfg_if;
use plazer_service::{init_logging, read_key, serve, ServeConfig};
use tracing::Level;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let stdout_level = env::var("PLAZER_LOG_LEVEL_STDOUT")
        .unwrap_or_else(|_| "info".to_owned())
        .parse()
        .unwrap_or({
            cfg_if! {
                if #[cfg(debug_assertions)] {
                    Level::DEBUG
                } else {
                    Level::INFO
                }
            }
        });
    let file_level = env::var("PLAZER_LOG_LEVEL_FILE")
        .unwrap_or_else(|_| "info".to_owned())
        .parse()
        .unwrap_or({
            cfg_if! {
                if #[cfg(debug_assertions)] {
                    Level::TRACE
                } else {
                    Level::INFO
                }
            }
        });

    let log_dir = env::var("PLAZER_LOG_DIR").unwrap_or_else(|_| "./data/logs".to_owned());
    let _guard = init_logging(log_dir, stdout_level, file_level);

    let persist_address =
        env::var("PLAZER_DB_ADDRESS").unwrap_or_else(|_| "file://./data/db".to_owned());
    // TODO: if fs path make parent dirs

    let private_key =
        env::var("PLAZER_PRIVATE_KEY").unwrap_or_else(|_| "./data/private_key.pem".to_string());

    let (enc_key, dec_key) = read_key(private_key).await?;

    let host = env::var("PLAZER_HOST").ok();
    let port: Option<u16> = match env::var("PLAZER_PORT").ok() {
        Some(port) => Some(port.parse()?),
        None => None,
    };

    serve(
        ServeConfig::new(persist_address, enc_key, dec_key)
            .set_host(host)
            .set_port(port),
    )
    .await?;
    Ok(())
}
