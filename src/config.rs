use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
    pub uid: String,
    pub serial_port: String,
    pub post_seconds: u64,
    pub mqtt: Mqtt,
}

#[derive(Deserialize)]
pub struct Mqtt {
    pub url: String,
    pub user: String,
    pub pass: String,
    pub keep_alive: u64,
}
