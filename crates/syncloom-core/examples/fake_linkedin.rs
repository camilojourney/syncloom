//! A fake LinkedIn as a process, for CLI runs and manual end-to-end checks. Never linkedin.com.
//!
//! `cargo run -p syncloom-core --example fake_linkedin -- [--auth consent|deny|wrong-state] [--token ok|400|401] [--post-mode posted|noid|slow|401|403|422|429|500]`
//! prints `listening on http://127.0.0.1:<port>`; point both `SYNCLOOM_LINKEDIN_AUTH_BASE_URL`
//! and `SYNCLOOM_LINKEDIN_API_BASE_URL` at it.
#[path = "../tests/support/fake_linkedin.rs"]
mod fake_linkedin;

use clap::Parser;
use fake_linkedin::{AuthMode, PostMode, TokenMode};

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "consent")]
    auth: String,
    #[arg(long, default_value = "ok")]
    token: String,
    #[arg(long, default_value = "posted")]
    post_mode: String,
}

#[tokio::main]
#[expect(
    clippy::print_stdout,
    reason = "This binary intentionally writes its documented user-facing output to the terminal."
)]
async fn main() -> Result<(), String> {
    let args = Args::parse();
    let auth = match args.auth.as_str() {
        "consent" => AuthMode::Consent,
        "deny" => AuthMode::Deny,
        "wrong-state" => AuthMode::WrongState,
        other => return Err(format!("unknown auth mode {other}")),
    };
    let token = match args.token.as_str() {
        "ok" => TokenMode::ok(),
        status => TokenMode::Status(
            status
                .parse()
                .map_err(|_| format!("unknown token mode {status}"))?,
        ),
    };
    let fake = fake_linkedin::start(auth, token, PostMode::parse(&args.post_mode)?).await;
    println!("listening on {}", fake.base_url());
    std::future::pending::<()>().await;
    Ok(())
}
