use amqprs::{channel::Channel, consumer::AsyncConsumer, BasicProperties, Deliver};
use prost::Message;
use tonic::async_trait;
use tracing::instrument;

use crate::dependencies::Dependencies;
use crate::email::EmailSender;
use crate::proto::ticketsrvc::Ticket;

pub struct TicketsConsumer {
    client: Dependencies,
    email_sender: EmailSender,
}

impl TicketsConsumer {
    pub fn new(client: Dependencies, email_sender: EmailSender) -> Self {
        Self {
            client,
            email_sender,
        }
    }
}

#[async_trait]
impl AsyncConsumer for TicketsConsumer {
    #[instrument(skip(self, _channel, _basic_properties, content))]
    async fn consume(
        &mut self,
        _channel: &Channel,
        _deliver: Deliver,
        _basic_properties: BasicProperties,
        content: Vec<u8>,
    ) {
        let Ok(ticket) = Ticket::decode(content.as_slice())
            .inspect_err(|error| tracing::error!(?error, "could not decode ticket"))
        else {
            return;
        };

        tracing::info!(?ticket, "sending update");

        let Ok(qr) = (self.client.get_qr_code(ticket.clone()).await)
            .inspect_err(|error| tracing::error!(?error, ticket.id, "failed to get qr code"))
        else {
            return;
        };

        let passenger = ticket.passenger.unwrap_or_default();

        let _ = self
            .email_sender
            .send_email(
                passenger.name.as_str(),
                passenger.email.as_str(),
                &ticket.url,
                qr,
            )
            .await
            .inspect_err(|error| tracing::error!(%error, ticket.id, "error sending email"));
    }
}
