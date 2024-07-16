use std::env;

#[derive(Clone)]
pub struct Settings {
    pub uid: String,

    pub mqtt_url: String,
    pub mqtt_user: String,
    pub mqtt_pass: String,
    pub keep_alive: u64,

    pub post_seconds: u64,

    pub serial_port: String,
}

impl Settings {
    pub fn from_env() -> Result<Self, String> {
        let uid =
            env::var("UID").map_err(|e| format!("UID: {}", e))?;

        let mqtt_url = env::var("MQTT_URL").map_err(|e| format!("MQTT_URL: {}", e))?;

        let mqtt_user = env::var("MQTT_USER").map_err(|e| format!("MQTT_USER: {}", e))?;
        let mqtt_pass = env::var("MQTT_PASS").map_err(|e| format!("MQTT_PASS: {}", e))?;

        let post_seconds = env::var("POST_SECONDS")
            .map_err(|e| format!("POST_SECONDS: {}", e))?
            .parse::<u64>()
            .map_err(|e| format!("POST_SECONDS: {}", e))?;

        let keep_alive = env::var("KEEP_ALIVE")
            .map_err(|e| format!("KEEP_ALIVE: {}", e))?
            .parse::<u64>()
            .map_err(|e| format!("KEEP_ALIVE: {}", e))?;

        let serial_port = env::var("SERIAL_PORT").map_err(|e| format!("SERIAL_PORT: {}", e))?;

        Ok(Settings {
            uid,
            mqtt_url,
            mqtt_user,
            mqtt_pass,
            keep_alive,
            post_seconds,
            serial_port,
        })
    }
}
