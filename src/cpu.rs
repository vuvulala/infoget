use serde::Deserialize;
use sysinfo::{CpuRefreshKind, RefreshKind};

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct CPUConfig {
    pub update_frequency: Option<i32>,
}

impl Default for CPUConfig {
    fn default() -> Self {
        Self {
            update_frequency: None,
        }
    }
}

impl Into<crate::common::UpdatePacket> for CPUUpdate {
    fn into(self) -> crate::common::UpdatePacket {
        crate::common::UpdatePacket::CPU(self)
    }
}
#[derive(Debug)]
pub enum CPUUpdate {
    Average(f32),
    Core(usize, f32),
}

pub async fn report_cpu(
    config: crate::common::Config,
    sender: tokio::sync::mpsc::Sender<crate::common::UpdatePacket>,
) {
    let mut sys = sysinfo::System::new();

    let timeout = config
        .cpu
        .update_frequency
        .unwrap_or(config.update_frequency);

    let mut int = tokio::time::interval(std::time::Duration::from_secs(timeout as u64));

    loop {
        int.tick().await;

        sys.refresh_specifics(RefreshKind::new().with_cpu(CpuRefreshKind::new().with_cpu_usage()));

        let mut sum = 0f32;
        let mut count = 0;

        for (i, cpu) in sys.cpus().iter().enumerate() {
            count += 1;
            let usage = cpu.cpu_usage();
            sum += usage;
            sender.send(CPUUpdate::Core(i, usage).into()).await.unwrap();
        }

        sender
            .send(CPUUpdate::Average(sum / count as f32).into())
            .await
            .unwrap();
    }
}
