mod flights_consumer;
mod tickets_consumer;

use std::error::Error;

use self::flights_consumer::FlightsConsumer;
use crate::{
    dependencies::Dependencies, email::EmailSender, rabbitmq::tickets_consumer::TicketsConsumer,
};
use amqprs::{
    callbacks::{DefaultChannelCallback, DefaultConnectionCallback},
    channel::{BasicConsumeArguments, Channel, QueueBindArguments, QueueDeclareArguments},
    connection::{Connection, OpenConnectionArguments},
};

pub struct Rabbit {
    connection: Connection,
    channel: Channel,
}

pub struct RabbitConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

pub enum UpdatesConsumer {
    FlightsConsumer,
    TicketsConsumer,
}

impl Rabbit {
    pub async fn new(
        config: RabbitConfig,
        flights_exchange_name: String,
        flights_queue_name: String,
        tickets_exchange_name: String,
        tickets_queue_name: String,
        clients: Dependencies,
    ) -> Result<Self, Box<dyn Error>> {
        tracing::info!("opening connection...");
        // open a connection to RabbitMQ server
        let rabbitmq = Connection::open(&OpenConnectionArguments::new(
            &config.host,
            config.port,
            &config.username,
            &config.password,
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
    tracing::info!(queue_name, "declaring queue");
    // declare queue
    let (queue, _, _) = channel
        .queue_declare(QueueDeclareArguments::durable_client_named(&queue_name))
        .await?
        .unwrap();

    tracing::info!(queue_name, exchange_name, "binding queue to exchange");
    // bind queue to exchange
    channel
        .queue_bind(QueueBindArguments::new(&queue, &exchange_name, ""))
        .await?;

    tracing::info!(queue_name, "consuming messages");
    // consume messages from queue
    let args = BasicConsumeArguments::new(&queue, consumer_tag)
        .manual_ack(false)
        .finish();

    let email_sender = EmailSender::new();

    match consumer {
        UpdatesConsumer::FlightsConsumer => channel
            .basic_consume(FlightsConsumer::new(clients, email_sender), args)
            .await
            .unwrap(),
        UpdatesConsumer::TicketsConsumer => channel
            .basic_consume(TicketsConsumer::new(clients, email_sender), args)
            .await
            .unwrap(),
    };

    Ok(())
}
