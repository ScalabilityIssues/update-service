use tonic::transport::Channel;

use crate::proto::{
    salesvc::validation_client::ValidationClient, ticketsrvc::tickets_client::TicketsClient,
};

#[derive(Clone)]
pub struct Dependencies {
    pub tickets: TicketsClient<Channel>,
    pub validation: ValidationClient<Channel>,
}

impl Dependencies {
    pub fn new(
        ticketsrvc_url: &str,
        validationsvc_url: &str,
    ) -> std::result::Result<Self, Box<dyn std::error::Error>> {
        let tickets_channel = Channel::builder(ticketsrvc_url.try_into()?).connect_lazy();
        let validationsvc_channel = Channel::builder(validationsvc_url.try_into()?).connect_lazy();

        Ok(Self {
            tickets: TicketsClient::new(tickets_channel),
            validation: ValidationClient::new(validationsvc_channel),
        })
    }
}
