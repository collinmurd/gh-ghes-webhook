use std::{sync::mpsc::Sender, thread, time::Duration};

use chrono::{DateTime, Utc};

use crate::gh::{CreateWebhookResponse, GitHub, WebhookDeliveryDetails};


pub fn poll(tx: Sender<WebhookDeliveryDetails>, gh: &GitHub, webhook: &CreateWebhookResponse) {
    let start_time: DateTime<Utc> = Utc::now();
    let mut last_id: Option<u64> = None;

    loop {
        thread::sleep(Duration::from_secs(5)); // Sleep for 5 seconds

        let deliveries = gh.get_webhook_deliveries(webhook.id);
        if let Ok(deliveries) = deliveries {
            log::debug!("Received {} deliveries", deliveries.len());
            // Iterate over deliveries in reverse order
            // skip events before the start time and only process new events
            for delivery in deliveries.iter().rev() {
                if let Some(last_delivery_id) = last_id {
                    if delivery.id > last_delivery_id {
                        last_id = Some(delivery.id);
                        log::debug!("Getting details for delivery: {:?}", delivery.id);
                        let details = gh.get_webhook_delivery_details(webhook.id, delivery.id).unwrap();
                        tx.send(details).unwrap();
                    }
                } else if delivery.delivered_at > start_time {
                    last_id = Some(delivery.id);
                    log::debug!("Getting details for delivery: {:?}", delivery.id);
                    let details = gh.get_webhook_delivery_details(webhook.id, delivery.id).unwrap();
                    tx.send(details).unwrap();
                }
            }
        } else if let Err(e) = deliveries {
            log::error!("Error polling for payloads: {:?}", e);
            break;
        }
    }
}