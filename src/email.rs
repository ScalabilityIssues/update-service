use base64::Engine;
use lettre::{
    message::{header, Mailbox},
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};
use thiserror::Error;

use crate::config;

pub struct EmailSender {
    mailer: AsyncSmtpTransport<Tokio1Executor>,
    sender_mailbox: Mailbox,
    content: config::MailContent,
}

impl EmailSender {
    pub fn new() -> Self {
        let config = envy::from_env::<config::MailConfig>().unwrap();
        let content = envy::from_env::<config::MailContent>().unwrap();

        let mailer = AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(config.smtp_host)
            .port(config.smtp_port)
            .build();

        let sender_mailbox = Mailbox::new(
            Some(config.sender_name),
            config.sender_address.parse().unwrap(),
        );

        Self {
            mailer,
            sender_mailbox,
            content,
        }
    }

    pub async fn send_email(
        &self,
        recipient_name: &str,
        recipient_address: &str,
        ticket_url: &str,
        qr: Vec<u8>,
    ) -> Result<(), SendEmailError> {
        let recipient_mailbox = format!("{} <{}>", recipient_name, recipient_address).parse()?;
        let qr = base64::engine::general_purpose::STANDARD.encode(qr);

        let email_message = Message::builder()
            .from(self.sender_mailbox.clone())
            .to(recipient_mailbox)
            .subject(&self.content.flight_update_subject)
            .header(header::ContentType::TEXT_HTML)
            .body(self.render_body(ticket_url, &qr))?;

        // Send the email
        self.mailer.send(email_message).await?;

        Ok(())
    }

    fn render_body(&self, url: &str, qr: &str) -> String {
        format!(
            include_str!("email_template.html"),
            body = self.content.flight_update_body.replace("\n", "<br/>"),
            url = url,
            qr = qr
        )
    }
}

#[derive(Debug, Error)]
pub enum SendEmailError {
    #[error("Error sending email: {0}")]
    SmtpError(#[from] lettre::transport::smtp::Error),
    #[error("Error parsing email address: {0}")]
    ParseError(#[from] lettre::address::AddressError),
    #[error("Error building email: {0}")]
    BuildError(#[from] lettre::error::Error),
}
