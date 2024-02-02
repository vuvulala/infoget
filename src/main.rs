use rumqttc::{self, MqttOptions, QoS};

use std::io::Read;
use tokio;
use toml;

mod common;
mod cpu;
mod ram;

static CONFIG_PATH: &'static str = "./config.toml";

#[tokio::main(flavor = "current_thread")]
async fn main() {
    let mut config_file =
        std::fs::File::open(CONFIG_PATH).expect("Could not find config file in specified path");

    let mut config_string = String::new();

    config_file.read_to_string(&mut config_string).unwrap();

    let config =
        toml::from_str::<common::Config>(&config_string).expect("Could not parse config file");
    println!("config: {:?}", config);

    let (mut mqtt_client, mut mqtt_connection) = rumqttc::AsyncClient::new(
        MqttOptions::new(&config.client_name, &config.server_ip, config.server_port),
        64,
    );

    let (sender, mut receiver) = tokio::sync::mpsc::channel::<crate::common::UpdatePacket>(128);

    let conf = config.clone();
    let send = sender.clone();
    tokio::spawn(async move {
        cpu::report_cpu(conf, send).await;
    });

    let conf = config.clone();
    let send = sender.clone();
    tokio::spawn(async move {
        ram::report_ram(conf, send).await;
    });

    loop {
        tokio::select! {
            _ = mqtt_connection.poll() => {
                println!("Notif");
            }
            message = receiver.recv() => {
                println!("message {:?}", message);
                handle_message(&config, message.unwrap(), &mut mqtt_client).await;
            }
        }
    }
}

async fn handle_message(
    config: &common::Config,
    message: common::UpdatePacket,
    client: &mut rumqttc::AsyncClient,
) {
    let path = format!("{}/{}", config.base_path, config.client_name);

    let (topic, payload) = match message {
        common::UpdatePacket::CPU(val) => match val {
            cpu::CPUUpdate::Average(value) => (format!("{path}/cpu/average"), value.to_string()),

            cpu::CPUUpdate::Core(core, value) => (
                format!("{path}/cpu/core/{}", core.to_string()),
                value.to_string(),
            ),
        },

        common::UpdatePacket::RAM(update) => match update {
            ram::RAMUpdate::Total(value) => (format!("{path}/ram/total"), value.to_string()),
            ram::RAMUpdate::Usage(value) => (format!("{path}/ram/usage"), value.to_string()),
        },
    };

    client
        .publish(topic, QoS::AtMostOnce, false, payload)
        .await
        .unwrap();
}
