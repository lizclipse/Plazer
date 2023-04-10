use std::env;

use c11ity_service::{read_key, serve, ServeConfig};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let data_dir = env::var("C11ITY_DATA_DIR").unwrap_or_else(|_| "data".to_owned());
    let private_key =
        env::var("C11ITY_PRIVATE_KEY").unwrap_or_else(|_| format!("{data_dir}/private_key.pem"));

    let (enc_key, dec_key) = read_key(private_key).await?;

    let host = env::var("C11ITY_HOST").ok();
    let port: Option<u16> = match env::var("C11ITY_PORT").ok() {
        Some(port) => Some(port.parse()?),
        None => None,
    };

    serve(
        ServeConfig::new(data_dir, enc_key, dec_key)
            .set_host(host)
            .set_port(port),
    )
    .await?;
    Ok(())
}
