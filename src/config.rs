use serde::Deserialize;

fn default_rabbitmq_port() -> u16 {
    5672
}

fn default_validationsvc_url() -> String {
    "grpc://validationsvc:50051".to_string()
}

fn default_ticketsrvc_url() -> String {
    "grpc://ticketsvc:50051".to_string()
}

#[derive(Deserialize, Debug)]
pub struct Options {
    pub rabbitmq_host: String,
    #[serde(default = "default_rabbitmq_port")]
    pub rabbitmq_port: u16,
    pub rabbitmq_username: String,
    pub rabbitmq_password: String,
    #[serde(default = "default_ticketsrvc_url")]
    pub ticketsrvc_url: String,
    #[serde(default = "default_validationsvc_url")]
    pub validationsvc_url: String,
}

fn default_smtp_host() -> String {
    "mailsvc".to_string()
}

fn default_smtp_port() -> u16 {
    25
}

#[derive(Deserialize, Debug)]
pub struct MailConfig {
    #[serde(default = "default_smtp_host")]
    pub smtp_host: String,
    #[serde(default = "default_smtp_port")]
    pub smtp_port: u16,
}

fn default_sender_name() -> String {
    "Simurgh Airlines".to_string()
}

fn default_sender_address() -> String {
    "update@simurghairlines.com".to_string()
}

fn default_flight_update_subject() -> String {
    "An update on your flight".to_string()
}

fn default_flight_update_body() -> String {
    "Hello, recently your flight has recieved an update. Follow the link down here to retrieve you new ticket!".to_string()
}

fn default_ticket_update_subject() -> String {
    "An update on your ticket".to_string()
}

fn default_ticket_update_body() -> String {
    "Hello, recently your ticket has been updated. Follow the link down here to retrieve you new ticket!\n".to_string()
}

#[derive(Deserialize, Debug)]
pub struct MailContent {
    #[serde(default = "default_sender_name")]
    pub sender_name: String,
    #[serde(default = "default_sender_address")]
    pub sender_address: String,
    #[serde(default = "default_flight_update_subject")]
    pub flight_update_subject: String,
    #[serde(default = "default_flight_update_body")]
    pub flight_update_body: String,
    #[serde(default = "default_ticket_update_subject")]
    pub ticket_update_subject: String,
    #[serde(default = "default_ticket_update_body")]
    pub ticket_update_body: String,
}
