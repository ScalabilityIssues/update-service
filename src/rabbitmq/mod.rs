use std::error::Error;

use amqprs::{
    callbacks::{DefaultChannelCallback, DefaultConnectionCallback},
    channel::{BasicConsumeArguments, Channel, QueueBindArguments, QueueDeclareArguments},
    connection::{Connection, OpenConnectionArguments},
    consumer::DefaultConsumer,
};

pub struct Rabbit {
    connection: Connection,
    channel: Channel,
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
            "consumer-flights",
        )
        .await?;

        // declare, bind and consume from tickets queue
        declare_bind_consume(
            &rabbitmq_channel,
            &tickets_exchange_name,
            &tickets_queue_name,
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
    channel
        .basic_consume(DefaultConsumer::new(args.no_ack), args) // TODO: use custom consumer that sends emails, like mailslurp or mailhog
        .await
        .unwrap();

    Ok(())
}
