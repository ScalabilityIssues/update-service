use amqprs::{channel::Channel, consumer::AsyncConsumer, BasicProperties, Deliver};
use tonic::async_trait;

use crate::{dependencies::Dependencies, proto::salesvc::SignTicketRequest, rabbitmq::send_mail};

pub struct TicketsConsumer {
    client: Dependencies,
}

impl TicketsConsumer {
    pub fn new(client: Dependencies) -> Self {
        Self { client }
    }
}

#[async_trait]
impl AsyncConsumer for TicketsConsumer {
    async fn consume(
        &mut self,
        _channel: &Channel,
        _deliver: Deliver,
        _basic_properties: BasicProperties,
        content: Vec<u8>,
    ) {
        let ticket =
            <crate::proto::ticketsrvc::Ticket as prost::Message>::decode(content.as_slice())
                .unwrap();

        tracing::info!("Send update for ticket: {:?}", ticket);
        let qr = self
            .client
            .validation
            .sign_ticket(SignTicketRequest {
                ticket: Some(ticket.clone()),
            })
            .await
            .unwrap()
            .into_inner()
            .qr;

        let passenger = ticket.passenger.unwrap_or_default();
        match send_mail(
            passenger.name.as_str(),
            passenger.email.as_str(),
            &ticket.url,
            qr,
        )
        .await
        {
            Ok(_) => (),
            Err(e) => tracing::error!("Error sending email: {}", e),
        }
        // we assume automatic ack
    }
}
