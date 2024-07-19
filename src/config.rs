use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub uid: String,
    pub serial_port: String,
    pub pid_file: String,
    pub post_seconds: u64,
    pub mqtt: Mqtt,
}

#[derive(Deserialize, Debug)]
pub struct Mqtt {
    pub url: String,
    pub user: String,
    pub pass: String,
    pub keep_alive: u64,
    pub port: u16,
    pub tls: bool
}
