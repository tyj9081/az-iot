use collector_scheduler::Collector;

/// 后台配置同步任务 — Phase 3 实现
pub async fn run(_collector: Collector) {
    tracing::info!("Config sync running (Phase 3 stub)");
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
    }
}
