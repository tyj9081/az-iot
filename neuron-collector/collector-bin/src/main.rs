use collector_scheduler::*;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("AZ-IOT Collector v1.0.0 starting...");

    let registry = Arc::new(RwLock::new(DeviceRegistry::new()));
    let collector = Collector::new(registry.clone());

    // Run scheduler
    collector.run().await?;

    tracing::info!("AZ-IOT Collector shutdown.");
    Ok(())
}
