use c11ity_service::{schema, serve};
use clap::{Args, Parser, Subcommand};
use tokio::{fs::File, io::AsyncWriteExt};

#[derive(Parser)]
#[command(author, version, about)]
#[command(propagate_version = true)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Schema(SchemaCommand),
}

#[derive(Args)]
#[command(about = "Generate code")]
struct SchemaCommand {
    #[arg(short, long, help = "The output directory")]
    output: Option<String>,
}

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let cli = Cli::parse();

    match cli.command {
        None => serve().await,
        Some(Commands::Schema(command)) => output_schema(command.output).await?,
    };

    Ok(())
}

async fn output_schema(output: Option<String>) -> Result<(), anyhow::Error> {
    let schema = schema().as_schema_language();

    match output {
        Some(output) => {
            let mut file = File::create(output).await?;
            file.write_all(schema.as_bytes()).await?;
        }
        None => {
            println!("{}", schema);
        }
    }

    Ok(())
}
