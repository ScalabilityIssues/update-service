mod flights_consumer;
mod tickets_consumer;

use std::error::Error;

use self::flights_consumer::FlightsConsumer;
use crate::{config, dependencies::Dependencies, rabbitmq::tickets_consumer::TicketsConsumer};
use amqprs::{
    callbacks::{DefaultChannelCallback, DefaultConnectionCallback},
    channel::{BasicConsumeArguments, Channel, QueueBindArguments, QueueDeclareArguments},
    connection::{Connection, OpenConnectionArguments},
};
use base64::Engine;
use lettre::SmtpTransport;
use lettre::{
    message::header::{self},
    Message, Transport,
};

pub struct Rabbit {
    connection: Connection,
    channel: Channel,
}

pub enum UpdatesConsumer {
    FlightsConsumer,
    TicketsConsumer,
}

impl Rabbit {
    pub async fn new(
        rabbitmq_host: &str,
        rabbitmq_port: u16,
        rabbitmq_user: &str,
        rabbitmq_password: &str,
        flights_exchange_name: String,
        flights_queue_name: String,
        tickets_exchange_name: String,
        tickets_queue_name: String,
        clients: Dependencies,
    ) -> Result<Self, Box<dyn Error>> {
        tracing::info!("opening connection...");
        // open a connection to RabbitMQ server
        let rabbitmq = Connection::open(&OpenConnectionArguments::new(
            rabbitmq_host,
            rabbitmq_port,
            rabbitmq_user,
            rabbitmq_password,
        ))
        .await?;

        tracing::info!("registering callback...");
        // Register connection level callbacks.
        // TODO: In production, user should create its own type and implement trait `ConnectionCallback`.
        rabbitmq
            .register_callback(DefaultConnectionCallback)
            .await?;

        tracing::info!("opening channel...");
        // open a channel on the connection
        let rabbitmq_channel = rabbitmq.open_channel(None).await?;
        rabbitmq_channel
            .register_callback(DefaultChannelCallback)
            .await?;

        // declare, bind and consume from flights queue
        declare_bind_consume(
            &rabbitmq_channel,
            &flights_exchange_name,
            &flights_queue_name,
            UpdatesConsumer::FlightsConsumer,
            clients.clone(),
            "consumer-flights",
        )
        .await?;

        // declare, bind and consume from tickets queue
        declare_bind_consume(
            &rabbitmq_channel,
            &tickets_exchange_name,
            &tickets_queue_name,
            UpdatesConsumer::TicketsConsumer,
            clients,
            "consumer-tickets",
        )
        .await?;

        Ok(Rabbit {
            connection: rabbitmq,
            channel: rabbitmq_channel,
        })
    }

    pub async fn close(self) {
        self.channel.close().await.unwrap();
        self.connection.close().await.unwrap();
    }
}

async fn declare_bind_consume(
    channel: &Channel,
    exchange_name: &str,
    queue_name: &str,
    consumer: UpdatesConsumer,
    clients: Dependencies,
    consumer_tag: &str,
) -> Result<(), Box<dyn Error>> {
    tracing::info!("declaring queue {}...", &queue_name);
    // declare queue
    let (queue, _, _) = channel
        .queue_declare(QueueDeclareArguments::durable_client_named(&queue_name))
        .await?
        .unwrap();

    tracing::info!(
        "binding queue {} to exchange {}...",
        &queue_name,
        &exchange_name
    );
    // bind queue to exchange
    channel
        .queue_bind(QueueBindArguments::new(&queue, &exchange_name, ""))
        .await?;

    tracing::info!("consuming messages from queue {}...", &queue_name);
    // consume messages from queue
    let args = BasicConsumeArguments::new(&queue, consumer_tag)
        .manual_ack(false)
        .finish();

    match consumer {
        UpdatesConsumer::FlightsConsumer => channel
            .basic_consume(FlightsConsumer::new(clients), args)
            .await
            .unwrap(),
        UpdatesConsumer::TicketsConsumer => channel
            .basic_consume(TicketsConsumer::new(clients), args)
            .await
            .unwrap(),
    };

    Ok(())
}

// TODO: make it async
pub async fn send_mail(
    recipient_name: &str,
    reicpient_address: &str,
    url: &str,
    qr: Vec<u8>,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = envy::from_env::<config::MailContent>().unwrap();
    let config = envy::from_env::<config::MailConfig>().unwrap();

    // build qr code
    // let qr = to_png_to_vec(
    //     prost::Message::encode_to_vec(&qr),
    //     QrCodeEcc::Medium,
    //     1024,
    // )?;
    tracing::info!("Building email...");
    let qr_img = base64::engine::general_purpose::STANDARD.encode(qr);
    let email = Message::builder()
        .from(format!("{} <{}>", content.sender_name, content.sender_address).parse()?)
        .to(format!("{} <{}>", recipient_name, reicpient_address).parse()?)
        .subject(content.flight_update_subject)
        .header(header::ContentType::TEXT_HTML)
        .body(format!("{} {} <img style=\"image-rendering: pixelated; height: auto; width: 25%;\"src=\"data:image/png;base64,{}\" />", content.flight_update_body, url, qr_img))?;

    tracing::info!("Connecting to smtp server...");
    let mailer = SmtpTransport::builder_dangerous(config.smtp_host)
        .port(config.smtp_port)
        .build();

    // Send the email
    tracing::info!("Sending email...");
    match mailer.send(&email) {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(e)),
    }
}
