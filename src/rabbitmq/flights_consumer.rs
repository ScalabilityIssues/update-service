use amqprs::{channel::Channel, consumer::AsyncConsumer, BasicProperties, Deliver};
use tonic::async_trait;

use crate::{
    dependencies::Dependencies,
    proto::{flightmngr::Flight, salesvc::SignTicketRequest, ticketsrvc::ListTicketsRequest},
    rabbitmq::send_mail,
};

pub struct FlightsConsumer {
    client: Dependencies,
}

impl FlightsConsumer {
    pub fn new(client: Dependencies) -> Self {
        Self { client }
    }
}

#[async_trait]
impl AsyncConsumer for FlightsConsumer {
    async fn consume(
        &mut self,
        _channel: &Channel,
        _deliver: Deliver,
        _basic_properties: BasicProperties,
        content: Vec<u8>,
    ) {
        let flight: Flight = <Flight as prost::Message>::decode(content.as_slice()).unwrap();
        let tickets = self
            .client
            .tickets
            .list_tickets(ListTicketsRequest {
                include_nonvalid: false,
                flight_id: Some(flight.id.clone()),
            })
            .await;
        let tickets = match tickets {
            Ok(tickets) => tickets.into_inner().tickets,
            Err(e) => {
                tracing::error!("Error listing tickets: {:?}", e);
                Default::default()
            }
        };

        tracing::info!(
            "Received flight update for flight_id={:?}, with {:?} associated tickets",
            flight.id,
            tickets.len()
        );

        for t in tickets {
            tracing::info!("Send update for ticket: {:?}", t);

            let qr = self
                .client
                .validation
                .sign_ticket(SignTicketRequest {
                    ticket: Some(t.clone()),
                })
                .await
                .unwrap()
                .into_inner()
                .qr;

            let passenger = t.passenger.unwrap_or_default();
            tracing::info!("Try to send email");
            match send_mail(
                passenger.name.as_str(),
                passenger.email.as_str(),
                &t.url,
                qr,
            )
            .await
            {
                Ok(_) => (),
                Err(e) => tracing::error!("Error sending email: {}", e),
            }
        }
        // we assume automatic ack
    }
}
