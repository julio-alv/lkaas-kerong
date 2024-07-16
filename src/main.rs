use chrono::Utc;
use dotenv::dotenv;
use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};

mod config;
mod kerong;

use config::Settings;
use kerong::board::CU16;
use kerong::status::Status;
use std::{sync::Arc, time::Duration};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::time;

#[derive(Debug)]
pub enum Msg {
    Status(String),
    Event(String),
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let cfg = Settings::from_env().expect("Failed to load Env variables");

    let mut mqttoptions = MqttOptions::new(&cfg.uid, &cfg.mqtt_url, 8883);
    mqttoptions
        .set_credentials(&cfg.mqtt_user, &cfg.mqtt_pass)
        .set_transport(rumqttc::Transport::tls_with_default_config())
        .set_clean_session(true)
        .set_keep_alive(Duration::from_secs(cfg.keep_alive));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 0);
    client
        .subscribe(format!("{}/cmd", &cfg.uid), QoS::ExactlyOnce)
        .await
        .unwrap();

    // Initialize CU16 Board
    let board = Arc::new(Mutex::new(
        CU16::initialize(&cfg.serial_port)
            .expect("Failed to initialize communication with CU16 Board"),
    ));
    let (tx, mut rx) = mpsc::unbounded_channel();
    let status = Arc::new(RwLock::new(Status::new()));

    // Events Loop
    // Checks every 50ms if the locker wall has changed
    // If it has changed, it will send a message to an unbounded channel
    let writer = Arc::clone(&status);
    let event_board = Arc::clone(&board);
    let txe = tx.clone();
    tokio::spawn(async move {
        event_loop(event_board, writer, txe, 50).await;
    });

    // Status loop
    // Sends the last status written by the event loop every minute
    let reader = Arc::clone(&status);
    tokio::spawn(async move {
        status_loop(reader, tx, cfg.post_seconds).await;
    });

    // CMD loop
    // Receives a message and attempts to unlock a locker
    let board_writer = Arc::clone(&board);
    tokio::spawn(async move {
        while let Ok(msg) = eventloop.poll().await {
            if let Event::Incoming(Packet::Publish(publish)) = msg {
                let payload = String::from_utf8(publish.payload.to_vec()).unwrap();
                if let Ok(n) = payload.parse::<u8>() {
                    let mut board = board_writer.lock().await;
                    board.open(n.saturating_sub(1)).unwrap();
                    println!("CMD: {:?}", n);
                }
            }
        }
    });

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            println!("{:?}", msg);
            match msg {
                Msg::Status(s) => client
                    .publish(format!("{}/status", &cfg.uid), QoS::AtLeastOnce, false, s)
                    .await
                    .unwrap_or_else(|e| eprintln!("Failed to publish to MQTT broker: {}", e)),
                Msg::Event(s) => client
                    .publish(format!("{}/events", &cfg.uid), QoS::AtLeastOnce, false, s)
                    .await
                    .unwrap_or_else(|e| eprintln!("Failed to publish to MQTT broker: {}", e)),
            }
        }
    });

    // Wait for Ctrl + C signal
    tokio::signal::ctrl_c().await.unwrap();
}

async fn event_loop(
    board: Arc<Mutex<CU16>>,
    writer: Arc<RwLock<Status>>,
    channel: UnboundedSender<Msg>,
    interval_ms: u64,
) {
    loop {
        let mut board = board.lock().await;
        let res = board.query_all();

        match res {
            Ok(new) => {
                let mut curr = writer.write().await;
                if new != *curr {
                    let now = Utc::now();
                    let timestamp = now.timestamp();
                    *curr = new.clone();

                    let msg = format!("{},{}", timestamp, new);

                    match channel.send(Msg::Event(msg)) {
                        Ok(_) => {}
                        Err(e) => {
                            eprintln!("Failed to send message to unbounded channel: {}", e);
                        }
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => {}
            Err(_) => break,
        }
        time::sleep(Duration::from_millis(interval_ms)).await;
    }
}

async fn status_loop(
    reader: Arc<RwLock<Status>>,
    channel: UnboundedSender<Msg>,
    interval_secs: u64,
) {
    loop {
        time::sleep(Duration::from_secs(interval_secs)).await;

        let timestamp = Utc::now().timestamp();
        let status = reader.read().await;
        let msg = format!("{},{}", timestamp, status);

        match channel.send(Msg::Status(msg)) {
            Ok(_) => {}
            Err(e) => {
                eprintln!("Failed to send message to unbounded channel: {}", e);
            }
        }
    }
}
