use std::{
    fs::{self, File},
    io::Write as _,
    path::Path,
};

use anyhow::Context as _;
use clap::{Args, Parser, Subcommand};
use pkcs8::der::Decode;
use plazer_service::{
    config::{
        LogLevel, ServiceConfigBuilder, DEFAULT_ADDRESS, DEFAULT_CONFIG_PATH, DEFAULT_DATABASE,
        DEFAULT_HOST, DEFAULT_LOG_DIR, DEFAULT_LOG_LEVEL_FILE, DEFAULT_LOG_LEVEL_STDOUT,
        DEFAULT_NAMESPACE, DEFAULT_PORT, DEFAULT_PRIVATE_KEY_PATH,
    },
    init_logging, schema, serve,
};
use ring::{rand, signature};

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
        help = format!("The port to listen on\n\n[default: {DEFAULT_PORT}]")
    )]
    port: Option<u16>,

    #[arg(
        long,
        help = format!("The host to listen on\n\n[default: {DEFAULT_HOST}]")
    )]
    host: Option<String>,

    #[arg(
        short,
        long,
        help = format!("The address of the remote database or the path to a local file\n\n[default: {DEFAULT_ADDRESS}]")
    )]
    address: Option<String>,

    #[arg(
        short,
        long,
        help = format!("The namespace to use in the database\n\n[default: {DEFAULT_NAMESPACE}]")
    )]
    namespace: Option<String>,

    #[arg(
        short,
        long,
        help = format!("The database to use in the namespace\n\n[default: {DEFAULT_DATABASE}]")
    )]
    database: Option<String>,

    #[arg(
        long,
        help = "The private key for authenticating (overrides --private-key-path)"
    )]
    private_key: Option<String>,

    #[arg(
        long,
        help = format!("The path to the private key for authenticating\n\n[default: {DEFAULT_PRIVATE_KEY_PATH}]")
    )]
    private_key_path: Option<String>,

    #[arg(
        short,
        long,
        help = format!("The directory to store logs in\n\n[default: {DEFAULT_LOG_DIR}]")
    )]
    log_dir: Option<String>,

    #[arg(
        long,
        help = format!("The level of logs to show on stdout\n\n[default: {}]", DEFAULT_LOG_LEVEL_STDOUT),
        value_enum
    )]
    log_level_stdout: Option<LogLevel>,

    #[arg(
        long,
        help = format!("The level of logs to show in log files\n\n[default: {}]", DEFAULT_LOG_LEVEL_FILE),
        value_enum
    )]
    log_level_file: Option<LogLevel>,

    #[arg(
        short,
        long,
        help = "Write the resolved configuration to the config file and exit",
        default_value_t = false
    )]
    write_config: bool,
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
        write_config,
    }: RunCommand,
) -> anyhow::Result<()> {
    let config = ServiceConfigBuilder::new()
        .set_port(port)
        .set_host(host)
        .set_address(address)
        .set_namespace(namespace)
        .set_database(database)
        .set_private_key(private_key)
        .set_private_key_path(private_key_path)
        .private_key_create(|path| generate_key(path))
        .set_log_dir(log_dir)
        .set_log_level_stdout(log_level_stdout)
        .set_log_level_file(log_level_file)
        .build()?;

    if write_config {
        fs::write(DEFAULT_CONFIG_PATH, toml::to_string(&config)?)?;
        return Ok(());
    }

    let (serve_config, log_config) = config.try_into()?;

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
