use tokio::sync::Notify;

use crate::{dependencies::Dependencies, rabbitmq::Rabbit};
mod config;
mod dependencies;
mod proto;
mod rabbitmq;
mod email;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt().init();

    let opt = envy::from_env::<config::Options>()?;

    let clients = Dependencies::new(&opt.ticketsrvc_url, &opt.validationsvc_url)?;

    tracing::info!("connecting to rabbitmq broker, creating queues and binding them to broker...");
    let r = Rabbit::new(
        rabbitmq::RabbitConfig {
            host: opt.rabbitmq_host,
            port: opt.rabbitmq_port,
            username: opt.rabbitmq_username,
            password: opt.rabbitmq_password,
        },
        "flight-update".to_string(),
        "flight-queue".to_string(),
        "ticket-update".to_string(),
        "ticket-queue".to_string(),
        clients,
    )
    .await?;

    // consume forever
    tracing::info!("updatesvc is consuming..., ctrl+c to exit");
    let guard = Notify::new();
    guard.notified().await;

    r.close().await; // to keep the connection and the channel open up to here

    Ok(())
}
