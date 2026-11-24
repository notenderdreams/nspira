use anyhow::Result;

/// Configure rayon thread pool based on user config
pub fn configure_thread_pool(parallelism: usize) -> Result<()> {
    rayon::ThreadPoolBuilder::new()
        .num_threads(parallelism)
        .build_global()
        .map_err(|e| anyhow::anyhow!("Failed to configure thread pool: {}", e))?;

    Ok(())
}
