use std::path::Path;

use anyhow::Context as _;
use c11ity_service::{read_key, schema, serve, ServeConfig};
use clap::{Args, Parser, Subcommand};
use pkcs8::der::Decode;
use ring::{rand, signature};
use tokio::{fs::File, io::AsyncWriteExt};

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
#[command(about = "Run server")]
struct RunCommand {
    #[arg(short, long, help = "The port to listen on [default: 8080]")]
    port: Option<u16>,

    #[arg(long, help = "The host to listen on")]
    host: Option<String>,

    #[arg(
        short,
        long,
        help = "The directory for storing data",
        default_value = "data"
    )]
    data: String,

    #[arg(
        long,
        help = "The private key for authenticating",
        default_value = "data/private_key.pem"
    )]
    private_key: String,
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

async fn run(cmd: RunCommand) -> anyhow::Result<()> {
    tokio::fs::create_dir_all(Path::new(&cmd.data))
        .await
        .context("Failed to create data directory")?;

    if !tokio::fs::try_exists(&cmd.private_key).await? {
        generate_key(&cmd.private_key).await?;
    }

    let (enc_key, dec_key) = read_key(cmd.private_key).await?;

    let config = ServeConfig::new(cmd.data, enc_key, dec_key)
        .set_host(cmd.host)
        .set_port(cmd.port);

    serve(config).await?;

    Ok(())
}

async fn output_schema(cmd: SchemaCommand) -> anyhow::Result<()> {
    let schema = schema(|s| s).sdl();

    match cmd.output {
        Some(output) => {
            let mut file = File::create(output).await?;
            file.write_all(schema.as_bytes()).await?;
        }
        None => println!("{}", schema),
    }

    Ok(())
}

async fn generate_key(path: impl AsRef<Path>) -> anyhow::Result<()> {
    let rng = rand::SystemRandom::new();
    let pkcs8_bytes = signature::Ed25519KeyPair::generate_pkcs8(&rng)?;

    if let Some(path) = path.as_ref().parent() {
        tokio::fs::create_dir_all(path)
            .await
            .context("Unable to create containing directories for private key")?;
    }

    tokio::fs::write(
        &path,
        pkcs8::Document::from_der(pkcs8_bytes.as_ref())
            .unwrap()
            .to_pem("PRIVATE KEY", pkcs8::LineEnding::LF)
            .unwrap(),
    )
    .await
    .context("Unable to write private key")?;

    println!("Private key written to {}", path.as_ref().display());
    Ok(())
}
