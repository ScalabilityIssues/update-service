use tonic::{transport::Channel, Status};

use crate::proto::salesvc::validation_client::ValidationClient;
use crate::proto::salesvc::SignTicketRequest;
use crate::proto::ticketsrvc::{tickets_client::TicketsClient, ListTicketsRequest, Ticket};

#[derive(Clone)]
pub struct Dependencies {
    ticketsvc: TicketsClient<Channel>,
    validationsvc: ValidationClient<Channel>,
}

impl Dependencies {
    pub fn new(
        ticketsrvc_url: &str,
        validationsvc_url: &str,
    ) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let tickets_channel = Channel::builder(ticketsrvc_url.try_into()?).connect_lazy();
        let validationsvc_channel = Channel::builder(validationsvc_url.try_into()?).connect_lazy();

        Ok(Self {
            ticketsvc: TicketsClient::new(tickets_channel),
            validationsvc: ValidationClient::new(validationsvc_channel),
        })
    }

    pub async fn list_tickets_for_flight(
        &mut self,
        flight_id: String,
    ) -> Result<Vec<Ticket>, Status> {
        self.ticketsvc
            .list_tickets(ListTicketsRequest {
                include_nonvalid: false,
                flight_id: Some(flight_id),
            })
            .await
            .map(|r| r.into_inner().tickets)
    }

    pub async fn get_qr_code(&mut self, ticket: Ticket) -> Result<Vec<u8>, Status> {
        self.validationsvc
            .sign_ticket(SignTicketRequest {
                ticket: Some(ticket),
            })
            .await
            .map(|r| r.into_inner().qr)
    }
}
