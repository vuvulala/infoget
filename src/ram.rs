use serde::Deserialize;
use sysinfo::{MemoryRefreshKind, RefreshKind};

use crate::common::UpdatePacket;

#[derive(Debug, Deserialize, Clone)]
pub struct RAMConfig {
    pub update_frequency: Option<i32>,
}

impl Default for RAMConfig {
    fn default() -> Self {
        Self {
            update_frequency: None,
        }
    }
}

#[derive(Debug)]
pub enum RAMUpdate {
    Usage(u64),
    Total(u64),
}

impl Into<UpdatePacket> for RAMUpdate {
    fn into(self) -> UpdatePacket {
        UpdatePacket::RAM(self)
    }
}

pub async fn report_ram(
    config: crate::common::Config,
    sender: tokio::sync::mpsc::Sender<crate::common::UpdatePacket>,
) {
    let mut sys = sysinfo::System::new();

    let timeout = config
        .ram
        .update_frequency
        .unwrap_or(config.update_frequency);

    let mut int = tokio::time::interval(std::time::Duration::from_secs(timeout as u64));

    loop {
        int.tick().await;

        sys.refresh_specifics(RefreshKind::new().with_memory(MemoryRefreshKind::new().with_ram()));

        sender
            .send(RAMUpdate::Usage(sys.used_memory()).into())
            .await
            .unwrap();

        sender
            .send(RAMUpdate::Total(sys.total_memory()).into())
            .await
            .unwrap();
    }
}
