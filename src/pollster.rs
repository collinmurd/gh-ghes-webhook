use std::{sync::mpsc::Sender, thread, time::Duration};

use chrono::{DateTime, Utc};

use crate::gh::{CreateWebhookResponse, GitHub, WebhookDelivery, WebhookDeliveryDetails};


pub fn poll(tx: Sender<WebhookDeliveryDetails>, gh: &GitHub, webhook: &CreateWebhookResponse) {
    let start_time: DateTime<Utc> = Utc::now();
    let mut last_id: Option<u64> = None;

    loop {
        log::debug!("Polling for webhook deliveries");
        let deliveries = gh.get_webhook_deliveries(webhook.id);
        if let Ok(deliveries) = deliveries {
            log::debug!("Received {} deliveries", deliveries.len());

            for delivery in deliveries.iter().rev() {
                if let Some(last_delivery_id) = last_id { // If we have a last_id, only get deliveries that are newer
                    if delivery.id > last_delivery_id {
                        last_id = Some(delivery.id);
                        send_details(&tx, webhook, delivery, gh);
                    }
                } else if delivery.delivered_at > start_time { // If we don't have a last_id, only get deliveries that are newer than the start time
                    last_id = Some(delivery.id);
                    send_details(&tx, webhook, delivery, gh);
                }
            }
        } else if let Err(e) = deliveries {
            log::error!("Error polling for payloads: {:?}", e);
            break;
        }

        thread::sleep(Duration::from_secs(5)); // Sleep for 5 seconds
    }
}

fn send_details(
    tx: &Sender<WebhookDeliveryDetails>,
    webhook: &CreateWebhookResponse,
    delivery: &WebhookDelivery,
    gh: &GitHub,
) {
    log::debug!("Getting details for delivery: {:?}", delivery.id);
    let details_resp = gh.get_webhook_delivery_details(webhook.id, delivery.id);

    if let Ok(details) = details_resp {
        tx.send(details).unwrap();
    } else {
        log::error!("Error getting delivery details: {:?}", delivery.id);
    }
}