use crate::gh::WebhookDeliveryDetails;


pub fn forward(payload: WebhookDeliveryDetails) {
    println!("Received webhook delivery: {:?}", payload);
}