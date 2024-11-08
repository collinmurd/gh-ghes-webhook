use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(version, about = "Webhook forwarding for GitHub Enterprise Server", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Forward webhooks to a local process
    Forward {
        /// Optional organization name
        #[arg(short, long)]
        arg: Option<String>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Forward { arg } => {
            println!("Forwarding to {}...", arg.unwrap_or("not provided".to_string()));
        }
    }
}


#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert();
}