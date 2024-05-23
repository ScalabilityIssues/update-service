use tokio::sync::Notify;
use tracing::info;

use crate::rabbitmq::Rabbit;

mod config;
mod rabbitmq;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().init();

    let opt = envy::from_env::<config::Options>()?;

    tracing::info!("connecting to rabbitmq broker, creating queues and binding them to broker...");
    let r = Rabbit::new(
        &opt.rabbitmq_host,
        opt.rabbitmq_port,
        &opt.rabbitmq_username,
        &opt.rabbitmq_password,
        "flight-update".to_string(),
        "flight-queue".to_string(),
        "ticket-update".to_string(),
        "ticket-queue".to_string(),
    )
    .await?;

    // consume forever
    tracing::info!("updatesvc is consuming..., ctrl+c to exit");
    let guard = Notify::new();
    guard.notified().await;

    r.close().await; // to keep the connection and the channel open up to here

    Ok(())
}
