use serde::Deserialize;

use crate::{cpu::CPUUpdate, ram::RAMUpdate};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    pub update_frequency: i32,
    pub server_ip: String,
    pub server_port: u16,
    pub client_name: String,
    pub base_path: String,
    #[serde(default)]
    pub cpu: crate::cpu::CPUConfig,
    #[serde(default)]
    pub ram: crate::ram::RAMConfig,
}

#[derive(Debug)]
pub enum UpdatePacket {
    CPU(CPUUpdate),
    RAM(RAMUpdate),
}
