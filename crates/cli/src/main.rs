use std::path::Path;

use anyhow::Context as _;
use c11ity_service::{init_logging, read_key, schema, serve, ServeConfig};
use clap::{Args, Parser, Subcommand, ValueEnum};
use pkcs8::der::Decode;
use ring::{rand, signature};
use tokio::{fs::File, io::AsyncWriteExt};
use tracing::Level;

#[derive(Parser)]
#[command(author, version, about)]
#[command(propagate_version = true)]
#[command(args_conflicts_with_subcommands = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    #[clap(flatten)]
    run: RunCommand,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Run server")]
    Run(RunCommand),
    #[command(about = "Generate schema")]
    Schema(SchemaCommand),
    #[command(about = "Generate JWT signing key")]
    GenerateKey(GenerateKeyCommand),
}

#[derive(Args)]
#[command(about = "Starts the server")]
struct RunCommand {
    #[arg(short, long, help = "The port to listen on [default: 8080]")]
    port: Option<u16>,

    #[arg(long, help = "The host to listen on")]
    host: Option<String>,

    #[arg(
        short,
        long,
        help = "The directory to store logs in",
        default_value = "./data/logs"
    )]
    log_dir: String,

    #[arg(
        short,
        long,
        help = "The address of the remote database or the path to a local file",
        default_value = "file://./data/db"
    )]
    db_address: String,

    #[arg(
        long,
        help = "The private key for authenticating",
        default_value = "./data/private_key.pem"
    )]
    private_key: String,

    #[arg(long, help = "The level of logs to show on stdout", value_enum)]
    #[cfg_attr(debug_assertions, arg(default_value = "debug"))]
    #[cfg_attr(not(debug_assertions), arg(default_value = "info"))]
    log_level_stdout: LogLevel,

    #[arg(long, help = "The level of logs to show in log files", value_enum)]
    #[cfg_attr(debug_assertions, arg(default_value = "trace"))]
    #[cfg_attr(not(debug_assertions), arg(default_value = "info"))]
    log_level_file: LogLevel,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum LogLevel {
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

#[derive(Args)]
#[command(about = "Generate schema")]
struct SchemaCommand {
    #[arg(short, long, help = "The output file")]
    output: Option<String>,
}

#[derive(Args)]
#[command(about = "Generate JWT signing key")]
struct GenerateKeyCommand {
    #[arg(
        short,
        long,
        help = "The output file",
        default_value = "data/private_key.pem"
    )]
    output: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Commands::Run(cli.run)) {
        Commands::Run(cmd) => run(cmd).await?,
        Commands::Schema(cmd) => output_schema(cmd).await?,
        Commands::GenerateKey(cmd) => generate_key(&cmd.output).await?,
    };

    Ok(())
}

async fn run(
    RunCommand {
        port,
        host,
        log_dir,
        db_address,
        private_key,
        log_level_stdout,
        log_level_file,
    }: RunCommand,
) -> anyhow::Result<()> {
    let _guard = init_logging(log_dir, log_level_stdout.into(), log_level_file.into());

    if !tokio::fs::try_exists(&private_key).await? {
        generate_key(&private_key).await?;
    }

    let (enc_key, dec_key) = read_key(private_key).await?;

    let config = ServeConfig::new(db_address, enc_key, dec_key)
        .set_host(host)
        .set_port(port);

    serve(config).await?;

    Ok(())
}

async fn output_schema(SchemaCommand { output }: SchemaCommand) -> anyhow::Result<()> {
    let schema = schema(|s| s).sdl();

    match output {
        Some(output) => {
            let mut file = File::create(output).await?;
            file.write_all(schema.as_bytes()).await?;
        }
        None => println!("{}", schema),
    }

    Ok(())
}

async fn generate_key(path: impl AsRef<Path>) -> anyhow::Result<()> {
    async fn inner(path: &Path) -> anyhow::Result<()> {
        let rng = rand::SystemRandom::new();
        let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng)?;

        if let Some(path) = path.parent() {
            tokio::fs::create_dir_all(path)
                .await
                .context("Unable to create containing directories for private key")?;
        }

        tokio::fs::write(
            path,
            pkcs8::Document::from_der(pkcs8_bytes.as_ref())
                .unwrap()
                .to_pem("PRIVATE KEY", pkcs8::LineEnding::LF)
                .unwrap(),
        )
        .await
        .context("Unable to write private key")?;

        println!("Private key written to {}", path.display());
        Ok(())
    }

    inner(path.as_ref()).await
}
