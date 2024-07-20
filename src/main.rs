use chrono::Utc;
use rumqttc::v5::mqttbytes::v5::Packet;
use rumqttc::v5::mqttbytes::QoS;
use rumqttc::v5::{AsyncClient, Event, EventLoop, MqttOptions};

use rand::{distributions::Alphanumeric, Rng};

mod config;
mod kerong;

use config::Config;
use kerong::board::CU16;
use kerong::status::Status;
use std::env;
use std::path::Path;
use std::{sync::Arc, time::Duration};
use tokio::sync::mpsc::UnboundedSender;
use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::{fs, time};

#[derive(Debug)]
pub enum Msg {
    Status(String),
    Event(String),
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let conf_path = Path::new(&args[1]);
    let content = fs::read_to_string(conf_path).await.expect("Failed to read file");
    let config: Config = toml::from_str(&content).expect("Failed to load Config.toml");

    // Initialize MQTT Client
    let client: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(8)
        .map(char::from)
        .collect();
    let mut opts = MqttOptions::new(format!("lkaas_{client}"), &config.mqtt.url, config.mqtt.port);
    opts.set_clean_start(true)
        .set_credentials(&config.mqtt.user, &config.mqtt.pass)
        .set_keep_alive(Duration::from_secs(config.mqtt.keep_alive));

    if config.mqtt.tls {
        opts.set_transport(rumqttc::Transport::tls_with_default_config());
    }

    let (cli, eventloop) = AsyncClient::new(opts, 16);
    cli.subscribe(format!("{}/cmd", &config.uid), QoS::ExactlyOnce)
        .await
        .unwrap();

    // Initialize CU16 Board
    let board = Arc::new(Mutex::new(
        CU16::initialize(&config.serial_port)
            .expect("Failed to initialize communication with CU16 Board"),
    ));
    let (tx, mut rx) = mpsc::unbounded_channel();
    let status = Arc::new(RwLock::new(Status::new()));

    // CMD loop
    // Receives a message and attempts to unlock a locker
    let board_writer: Arc<Mutex<CU16>> = Arc::clone(&board);
    tokio::spawn(async move {
        cmd_loop(board_writer, eventloop).await;
    });

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
        status_loop(reader, tx, config.post_seconds).await;
    });

    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            match msg {
                Msg::Status(s) => cli
                    .publish(
                        format!("{}/status", &config.uid),
                        QoS::ExactlyOnce,
                        false,
                        s,
                    )
                    .await
                    .unwrap_or_else(|e| eprintln!("Failed to publish to sub-topic /status: {e}")),

                Msg::Event(s) => cli
                    .publish(
                        format!("{}/events", &config.uid),
                        QoS::ExactlyOnce,
                        false,
                        s,
                    )
                    .await
                    .unwrap_or_else(|e| eprintln!("Failed to publish to sub-topic /events: {e}")),
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
                            eprintln!("Failed to send message to unbounded channel: {e}");
                        }
                    }
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::TimedOut => (),
            Err(e) => {
                eprintln!("Serial Error: {e}");
                time::sleep(Duration::from_millis(interval_ms * 2)).await;
            }
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

async fn cmd_loop(board: Arc<Mutex<CU16>>, mut eventloop: EventLoop) {
    loop {
        let event = eventloop.poll().await;
        match &event {
            Ok(v) => {
                match v {
                    Event::Incoming(Packet::Publish(publish)) => {
                        let payload = String::from_utf8(publish.payload.to_vec()).unwrap();
                        if let Ok(n) = payload.parse::<u8>() {
                            let mut board = board.lock().await;
                            board.open(n.saturating_sub(1)).unwrap();
                        }
                    }
                    Event::Incoming(_) => (),
                    Event::Outgoing(_) => (),
                }
            }
            Err(e) => {
                eprintln!("Error = {e:?}");
            }
        }
    }
}
