use std::{sync::mpsc, thread};

use clap::{Args, Parser, Subcommand};

pub mod gh;
pub mod pollster;
pub mod forwarder;

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
        org: Option<String>,

        /// Name of the repo where the webhook is installed
        #[arg(short='R', long)]
        repo: Option<String>,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Forward {events, github_host, location, secret, url} => {
            let gh = match location.repo {
                Some(r) => gh::GitHub::new_with_repo(github_host, r),
                None => gh::GitHub::new_with_org(github_host, location.org.unwrap())
            };

            let webhook = gh.create_webhook(secret, events).unwrap();
            println!("CLI Webhook created");

            let gh_clone = gh.clone();
            ctrlc::set_handler(move || {
                println!("Deleting webhook...");
                gh_clone.delete_webhook(webhook.id).unwrap();
                std::process::exit(0);
            }).unwrap();

            let (tx, rx) = mpsc::channel();

            thread::spawn(move || {
                pollster::poll(tx, &gh, &webhook);
            });

            loop {
                match rx.recv() {
                    Ok(payload) => {
                        forwarder::forward(payload);
                    },
                    Err(_e) => {
                        break;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::*;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }
}