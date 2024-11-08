use clap::{Args, Parser, Subcommand};

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
        /// Names of the event types to forward. Use `*` to forward all events.
        #[arg(short='E', long, num_args=1.., value_delimiter=' ')]
        events: Vec<String>,

        /// GitHub host name (default "github.com")
        #[arg(short='H', long, default_value_t=String::from("github.com"))]
        github_host: String,

        #[command(flatten)]
        location: WebhookLocation,

        /// Webhook secret for incoming events
        #[arg(short='S', long)]
        secret: Option<String>,

        /// Address of the local server to receive events. If omitted, events will be printed to stdout
        #[arg(short='U', long)]
        url: Option<String>,
    },
}

#[derive(Args)]
#[group(required = true, multiple=false)]
struct WebhookLocation {
        /// Name of the org where the webhook is installed
        #[arg(short='O', long)]
        org: String,

        /// Name of the repo where the webhook is installed
        #[arg(short='R', long)]
        repo: String,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Forward {events, github_host, location, secret, url} => {
            println!("Forwarding...");
        }
    }
}


#[test]
fn verify_cli() {
    use clap::CommandFactory;
    Cli::command().debug_assert();
}