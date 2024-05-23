use serde::Deserialize;

fn default_rabbitmq_port() -> u16 {
    5672
}

#[derive(Deserialize, Debug)]
pub struct Options {
    pub rabbitmq_host: String,
    #[serde(default = "default_rabbitmq_port")]
    pub rabbitmq_port: u16,
    pub rabbitmq_username: String,
    pub rabbitmq_password: String,
}
