use std::{
    fs::{self, File},
    io::Write as _,
    path::Path,
};

use anyhow::Context as _;
use clap::{Args, Parser, Subcommand, ValueEnum};
use pkcs8::der::Decode;
use plazer_service::{
    config::{
        ServiceConfig, DEFAULT_ADDRESS, DEFAULT_DATABASE, DEFAULT_HOST, DEFAULT_LOG_DIR,
        DEFAULT_NAMESPACE, DEFAULT_PORT, DEFAULT_PRIVATE_KEY_PATH,
    },
    init_logging, schema, serve,
};
use ring::{rand, signature};
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
    #[command(about = "Run server (default)")]
    Run(RunCommand),
    #[command(about = "Generate schema")]
    Schema(SchemaCommand),
    #[command(about = "Generate JWT signing key")]
    GenerateKey(GenerateKeyCommand),
}

#[derive(Args)]
#[command(about = "Starts the server")]
struct RunCommand {
    #[arg(
        short,
        long,
        help = format!("The port to listen on [default: {DEFAULT_PORT}]")
    )]
    port: Option<u16>,

    #[arg(long, help = format!("The host to listen on [default: {DEFAULT_HOST}]"))]
    host: Option<String>,

    #[arg(
        short,
        long,
        help = format!("The address of the remote database or the path to a local file [default: {DEFAULT_ADDRESS}]")
    )]
    address: Option<String>,

    #[arg(
        short,
        long,
        help = format!("The namespace to use in the database [default: {DEFAULT_NAMESPACE}]")
    )]
    namespace: Option<String>,

    #[arg(
        short,
        long,
        help = format!("The database to use in the namespace [default: {DEFAULT_DATABASE}]")
    )]
    database: Option<String>,

    #[arg(
        long,
        help = "The private key for authenticating (overrides --private-key-path)"
    )]
    private_key: Option<String>,

    #[arg(
        long,
        help = format!("The path to the private key for authenticating [default: {DEFAULT_PRIVATE_KEY_PATH}]")
    )]
    private_key_path: Option<String>,

    #[arg(
        short,
        long,
        help = format!("The directory to store logs in [default: {DEFAULT_LOG_DIR}]")
    )]
    log_dir: Option<String>,

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
        default_value = DEFAULT_PRIVATE_KEY_PATH
    )]
    output: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Commands::Run(cli.run)) {
        Commands::Run(cmd) => run(cmd).await?,
        Commands::Schema(cmd) => output_schema(cmd)?,
        Commands::GenerateKey(cmd) => {
            generate_key(cmd.output)?;
        }
    };

    Ok(())
}

async fn run(
    RunCommand {
        port,
        host,
        address,
        namespace,
        database,
        private_key,
        private_key_path,
        log_dir,
        log_level_stdout,
        log_level_file,
    }: RunCommand,
) -> anyhow::Result<()> {
    let (serve_config, log_config) = ServiceConfig::new()
        .set_port(port)
        .set_host(host)
        .set_address(address)
        .set_namespace(namespace)
        .set_database(database)
        .set_private_key(private_key)
        .set_private_key_path(private_key_path)
        .private_key_create(|path| generate_key(path))
        .set_log_dir(log_dir)
        .log_level_stdout(log_level_stdout)
        .log_level_file(log_level_file)
        .try_into()?;

    let _guard = init_logging(log_config);
    serve(serve_config).await?;

    Ok(())
}

fn output_schema(SchemaCommand { output }: SchemaCommand) -> anyhow::Result<()> {
    let schema = schema(|s| s).sdl();

    match output {
        Some(output) => {
            let mut file = File::create(output)?;
            file.write_all(schema.as_bytes())?;
        }
        None => println!("{}", schema),
    }

    Ok(())
}

fn generate_key(path: impl AsRef<Path>) -> anyhow::Result<String> {
    fn inner(path: &Path) -> anyhow::Result<String> {
        let rng = rand::SystemRandom::new();
        let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng)?;

        if let Some(path) = path.parent() {
            fs::create_dir_all(path)
                .context("Unable to create containing directories for private key")?;
        }

        let pem = pkcs8::Document::from_der(pkcs8_bytes.as_ref())
            .unwrap()
            .to_pem("PRIVATE KEY", pkcs8::LineEnding::LF)
            .unwrap();
        fs::write(path, &pem).context("Unable to write private key")?;

        println!("Private key written to {}", path.display());
        Ok(pem)
    }

    inner(path.as_ref())
}
