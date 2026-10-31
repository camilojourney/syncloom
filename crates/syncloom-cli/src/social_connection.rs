//! SocialPlatforms connection commands. Secrets are read from stdin, never command arguments or files.
use clap::Subcommand;
use serde_json::Value;
use syncloom_core::{
    Channel, Secret, Studio, StudioError, oauth::Connection, social::SocialPlatforms,
};
#[derive(Subcommand)]
pub(crate) enum Command {
    /// Bluesky: provide --identifier (handle), then app password on stdin. X: browser callback.
    /// Threads: print consent URL, then paste the full HTTPS callback URL on stdin.
    SignIn {
        #[arg(long)]
        identifier: Option<String>,
    },
    /// Read local credential state without a network request.
    Status,
    /// Explicitly renew stored credentials. Never posts.
    Refresh,
    /// Delete local credentials.
    SignOut,
}
fn stdin_secret() -> Result<Secret, StudioError> {
    use std::io::{IsTerminal, Read};
    if std::io::stdin().is_terminal() {
        let input = rpassword::read_password()?;
        if input.is_empty() || input.len() > 16 * 1024 {
            return Err(StudioError::InvalidRequest);
        }
        return Ok(Secret::new(input));
    }
    let mut input = String::new();
    std::io::stdin()
        .lock()
        .take(16 * 1024 + 1)
        .read_to_string(&mut input)?;
    if input.len() > 16 * 1024 || input.trim().is_empty() {
        return Err(StudioError::InvalidRequest);
    }
    Ok(Secret::new(input.trim_end_matches(['\r', '\n'])))
}
pub(crate) async fn run(
    studio: &Studio,
    platform: &str,
    command: Command,
) -> Result<Value, StudioError> {
    let channel = Channel::parse(platform).ok_or(StudioError::UnsupportedChannel)?;
    let q = studio.social()?;
    let connection = match command {
        Command::SignIn { identifier } => sign_in(q, channel, identifier).await?,
        Command::Status => q.connection(channel)?,
        Command::Refresh => q.refresh(channel).await?,
        Command::SignOut => q.sign_out(channel)?,
    };
    Ok(crate::to_json(connection))
}
async fn sign_in(
    q: &SocialPlatforms,
    channel: Channel,
    identifier: Option<String>,
) -> Result<Connection, StudioError> {
    match channel {
        Channel::Bluesky => {
            q.bluesky
                .sign_in(
                    &identifier.ok_or(StudioError::InvalidRequest)?,
                    stdin_secret()?,
                )
                .await
        }
        Channel::X => {
            println!("{}", crate::to_json(q.x.sign_in_begin()?));
            q.x.sign_in_wait().await
        }
        Channel::Threads => {
            println!("{}", crate::to_json(q.threads.sign_in_begin()?));
            q.threads.complete(stdin_secret()?.expose()).await
        }
        _ => Err(StudioError::UnsupportedChannel),
    }
}
