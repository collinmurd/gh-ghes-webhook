use std::{process::exit, sync::mpsc, thread};

use clap::{Args, Parser, Subcommand};
use gh::GitHub;
use simplelog::{ConfigBuilder, TermLogger};

pub mod gh;
pub mod pollster;
pub mod forwarder;

#[derive(Parser)]
#[command(version, about = "Webhook forwarding for GitHub Enterprise Server", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable debug logging
    #[arg(short='X', long, default_value_t=false)]
    debug: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Forward webhooks to a local process
    Forward {
        /// Names of the event types to forward. Use `*` to forward all events default: push
        #[arg(short='E', long, num_args=1.., value_delimiter=',')]
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
        /// NOT SUPPORTED - organization where the webhook is installed
        #[arg(short='O', long)]
        org: Option<String>,

        /// Name of the repo where the webhook is installed
        #[arg(short='R', long)]
        repo: Option<String>,
}

fn main() {
    let cli = Cli::parse();
    configure_logger(cli.debug);

    match cli.command {
        Commands::Forward {events, github_host, location, secret, url} => {
            let gh = match location.org {
                Some(_) => {
                    log::error!("Organization webhooks are not supported.");
                    exit(1);
                }
                None => gh::GitHub::new_with_repo(github_host, location.repo.unwrap()),
            };

            let webhook = gh.create_webhook(secret, events).unwrap();
            let webhook_id = webhook.id;
            log::info!("CLI Webhook created");

            // Set up a handler to delete the webhook when the user presses Ctrl-C
            set_ctrlc_listener(&gh, webhook_id);

            // spawn thread to poll for events
            let (tx, rx) = mpsc::channel();
            let gh_clone = gh.clone();
            thread::spawn(move || {
                pollster::poll(tx, &gh_clone, &webhook);
            });

            // forward events
            let forwarder: Box<dyn forwarder::Forwarder> = match url {
                Some(u) => Box::new(forwarder::LocalForwarder::new(u)),
                None => Box::new(forwarder::StdOutForwarder::new())
            };
            loop {
                if let Ok(details) = rx.recv() {
                    match details {
                        pollster::PollMessage::TimedOut => {
                            log::warn!("Polling timed out after 10 minutes of inactivity. Shutting down...");
                            gh.delete_webhook(webhook_id).unwrap();
                            break;
                        }
                        pollster::PollMessage::Delivery(details) => {
                            log::info!("Forwarding event: {}", details.id);
                            forwarder.forward(details.request);
                        }
                    }
                }
            }
        }
    }
}

fn configure_logger(verbose: bool) {
    TermLogger::init(
        if verbose { simplelog::LevelFilter::Debug } else { simplelog::LevelFilter::Info },
        ConfigBuilder::new()
            .set_time_format_rfc3339()
            .set_time_offset_to_local().unwrap()
            .build(),
        simplelog::TerminalMode::Mixed,
        simplelog::ColorChoice::Auto
    ).unwrap();
}

fn set_ctrlc_listener(gh: &GitHub, webhook_id: u32) {
    let gh_clone = gh.clone();
    ctrlc::set_handler(move || {
        log::info!("Deleting CLI webhook");
        gh_clone.delete_webhook(webhook_id).unwrap();
        std::process::exit(0);
    }).unwrap();
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