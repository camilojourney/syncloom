use std::sync::Arc;
use syncloom_core::{Settings, build_studio, studio_config};
use syncloom_server::{local_address, log_subscriber, serve};

/// An environment variable that may be unset; set but not valid Unicode is an error.
fn env(name: &str) -> Result<Option<String>, String> {
    match std::env::var(name) {
        Ok(value) => Ok(Some(value)),
        Err(std::env::VarError::NotPresent) => Ok(None),
        Err(std::env::VarError::NotUnicode(_)) => Err(format!("{name} is not valid Unicode")),
    }
}

/// Logging must never stall a request: if nobody drains stdout, lines are dropped, not queued
/// behind a full pipe. The returned guard flushes what is buffered when dropped.
fn init_logging() -> Result<tracing_appender::non_blocking::WorkerGuard, Box<dyn std::error::Error>>
{
    let (stdout, guard) = tracing_appender::non_blocking::NonBlockingBuilder::default()
        .lossy(true)
        .finish(std::io::stdout());
    tracing::subscriber::set_global_default(log_subscriber(stdout))?;
    Ok(guard)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Configuration is checked before anything is bound or logged.
    let address = local_address(env("SYNCLOOM_ADDR")?.as_deref())?;
    let config = studio_config(|name| std::env::var(name).ok())?;
    let _flush_logs = init_logging()?;
    let studio = Arc::new(build_studio(config, Settings::default())?);
    let listener = tokio::net::TcpListener::bind(address).await?;
    tracing::info!(address = %listener.local_addr()?, version = env!("CARGO_PKG_VERSION"), "listening");
    tokio::select! {
        result = serve(listener, studio) => result?,
        result = tokio::signal::ctrl_c() => result?,
    }
    Ok(())
}
