use std::{sync::mpsc, thread, time::Duration};

use chrono::{DateTime, Utc};
use clap::{Args, Parser, Subcommand};

pub mod gh;

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

            let (tx, rx) = mpsc::channel();

            thread::spawn(move || {
                let start_time: DateTime<Utc> = Utc::now();
                println!("Start time: {:?}", start_time);
                let mut last_id: Option<u64> = None;
                loop {
                    thread::sleep(Duration::from_secs(5)); // Sleeps for 5 seconds
                    println!("Polling for payloads");
                    let deliveries = gh.get_webhook_deliveries(webhook.id);
                    println!("Deliveries: {:?}", deliveries);
                    match deliveries {
                        Ok(deliveries) => {
                            // Iterate over deliveries in reverse order
                            // skip events before the start time and only process new events
                            for delivery in deliveries.iter().rev() {
                                if let Some(last_delivery_id) = last_id {
                                    if delivery.id > last_delivery_id {
                                        last_id = Some(delivery.id);
                                        let details = gh.get_webhook_delivery_details(webhook.id, delivery.id).unwrap();
                                        tx.send(details).unwrap();
                                    }
                                } else if delivery.delivered_at > start_time {
                                    last_id = Some(delivery.id);
                                    let details = gh.get_webhook_delivery_details(webhook.id, delivery.id).unwrap();
                                    tx.send(details).unwrap();
                                }
                            }
                        },
                        Err(e) => {
                            println!("Error polling for payloads: {:?}", e);
                            break;
                        }
                    }
                }
            });

            loop {
                match rx.recv() {
                    Ok(payload) => {
                        println!("Received payload: {:?}", payload);
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