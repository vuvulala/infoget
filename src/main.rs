use rumqttc::{self, MqttOptions, QoS};
use serde::Deserialize;
use std::{fmt::format, io::Read, net::IpAddr, thread, time};
use sysinfo::{self, CpuRefreshKind, MemoryRefreshKind, RefreshKind};
use tokio;
use toml;

static CONFIG_PATH: &'static str = "./config.toml";

#[derive(Debug, Deserialize, Clone)]
struct Config {
    update_frequency: i32,
    server_ip: String,
    server_port: u16,
    client_name: String,
    base_path: String,
}

fn main() {
    let mut config_file =
        std::fs::File::open(CONFIG_PATH).expect("Could not find config file in specified path");
    let mut config_string = String::new();
    config_file.read_to_string(&mut config_string).unwrap();

    let config = toml::from_str::<Config>(&config_string).expect("Could not parse config file");
    println!("config: {:?}", config);

    let (mut mqtt_client, mut mqtt_connection) = rumqttc::Client::new(
        MqttOptions::new(&config.client_name, &config.server_ip, config.server_port),
        64,
    );

    thread::spawn(move || report_data(config.clone(), &mut mqtt_client));

    for notification in mqtt_connection.iter() {
        println!("notif = {:?}", notification.unwrap());
    }
}

fn report_data(config: Config, mqtt_client: &mut rumqttc::Client) {

    let mut sys = sysinfo::System::new();

    loop {
        sys.refresh_specifics(
            RefreshKind::new()
                .with_cpu(CpuRefreshKind::new().with_cpu_usage())
                .with_memory(MemoryRefreshKind::new().with_ram()),
        );

        let base_topic = format!("{}/{}", config.base_path, config.client_name);

        let mut sum = 0f32;
        let mut count = 0;
        for (i, cpu) in sys.cpus().iter().enumerate() {
            count += 1;
            sum += cpu.cpu_usage();
            mqtt_client
            .publish(format!("{base_topic}/cpu_usage/cores/{i}"), QoS::AtMostOnce, false, cpu.cpu_usage().to_string())
            .unwrap();
        }


        mqtt_client
        .publish(format!("{base_topic}/cpu_usage/average"), QoS::AtMostOnce, false, (sum / count as f32).to_string())
        .unwrap();
        mqtt_client
            .publish(format!("{base_topic}/ram_usage"), QoS::AtMostOnce, false, sys.used_memory().to_string())
            .unwrap();

        mqtt_client
            .publish(format!("{base_topic}/ram_total"), QoS::AtMostOnce, false, sys.total_memory().to_string())
            .unwrap();

        thread::sleep(std::time::Duration::from_secs(
            config.update_frequency as u64,
        ));
    }
}
