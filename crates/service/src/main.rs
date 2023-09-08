use plazer_service::{config::ServiceConfigBuilder, init_logging, serve};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let (serve_config, log_config) = ServiceConfigBuilder::default().build()?.try_into()?;

    let _guard = init_logging(log_config);
    serve(serve_config).await?;

    Ok(())
}
