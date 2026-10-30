//! A fake Claude Messages endpoint as a process, for CLI runs and manual end-to-end checks.
//!
//! `cargo run -p syncloom-core --example fake_claude -- --mode ok|empty|notext|slow|401|429|500 [--text "..."]`
//! prints `listening on http://127.0.0.1:<port>`; point `SYNCLOOM_CLAUDE_BASE_URL` at it.
#[path = "../tests/support/fake_claude.rs"]
mod fake_claude;

use clap::Parser;

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "ok")]
    mode: String,
    #[arg(long, default_value = "A draft from the fake provider about the thought.")]
    text: String,
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let args = Args::parse();
    let fake = fake_claude::start(fake_claude::Mode::parse(&args.mode, &args.text)?).await;
    println!("listening on {}", fake.base_url());
    std::future::pending::<()>().await;
    Ok(())
}
