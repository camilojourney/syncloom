//! `syncloom`: the studio from the command line, over the same core as the HTTP API.
//!
//! Results print as JSON on stdout with exit 0. A failed command prints the core's
//! `{"error":{"code","message"}}` on stderr with exit 1. Configuration errors exit 2.
use clap::{Parser, Subcommand};
use serde_json::{Value, json};
use syncloom_core::{
    PublishRequest, ReviewRequest, Settings, Status, Studio, StudioError, ThoughtRequest,
    build_studio, studio_config,
};

#[derive(Parser)]
#[command(
    name = "syncloom",
    about = "Syncloom: thought to reviewed content",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Draft pieces from one thought.
    Thought {
        #[arg(long)]
        text: String,
        /// A channel from the platform-rules table; repeat for several.
        #[arg(long, default_value = "linkedin_text")]
        channel: Vec<String>,
    },
    /// Show one piece.
    Show { piece_id: String },
    /// Approve one piece at the revision you reviewed.
    Approve {
        piece_id: String,
        #[arg(long)]
        expected_revision: String,
    },
    /// Publish an approved piece (dispatched when SYNCLOOM_DELIVERY grants its platform,
    /// else a contained intent), or preview the payload with --dry-run.
    Publish {
        piece_id: String,
        #[arg(long)]
        expected_revision: String,
        #[arg(long)]
        dry_run: bool,
    },
    /// Render revision-bound PNG and PDF assets using the pinned local browser.
    Render {
        piece_id: String,
        #[arg(long)]
        expected_revision: String,
        #[arg(long, default_value = "midnight")]
        template: String,
        #[arg(long = "slide", required = true)]
        slides: Vec<String>,
    },
    /// Write verified PNG or PDF bytes to stdout (redirect to a local file).
    View {
        piece_id: String,
        #[arg(long, value_parser = ["image", "carousel"])]
        kind: String,
    },
    /// The LinkedIn connection: sign in through the browser, show it, or sign out.
    Linkedin {
        #[command(subcommand)]
        command: LinkedinCommand,
    },
}

#[derive(Subcommand)]
enum LinkedinCommand {
    /// Print the authorization URL, wait for the browser, store the token in the Keychain.
    SignIn,
    /// Show the connection state from the Keychain; no LinkedIn call.
    Status,
    /// Delete the token from the Keychain.
    SignOut,
}

fn to_json<T: serde::Serialize>(value: T) -> Value {
    serde_json::to_value(value).expect("studio results serialize")
}

async fn run(cli: Cli) -> i32 {
    let config = match studio_config(|name| std::env::var(name).ok()) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("{error}");
            return 2;
        }
    };
    let studio = match build_studio(config, Settings::default()) {
        Ok(studio) => studio,
        Err(error) => return failed(StudioError::from(error)),
    };
    let result = match cli.command {
        Command::Thought { text, channel } => studio
            .create_thought(ThoughtRequest {
                text,
                channels: channel,
            })
            .await
            .map(to_json),
        Command::Show { piece_id } => studio.show(&piece_id).map(to_json),
        Command::Approve {
            piece_id,
            expected_revision,
        } => studio
            .review(
                &piece_id,
                ReviewRequest {
                    status: Some(Status::Approved),
                    text: None,
                    expected_revision,
                },
            )
            .map(to_json),
        Command::Publish {
            piece_id,
            expected_revision,
            dry_run,
        } => studio
            .publish(
                &piece_id,
                PublishRequest {
                    expected_revision,
                    dry_run,
                },
            )
            .await
            .map(to_json),
        Command::Render {
            piece_id,
            expected_revision,
            template,
            slides,
        } => studio
            .render(
                &piece_id,
                syncloom_core::RenderRequest {
                    expected_revision,
                    template,
                    slides,
                },
            )
            .await
            .map(to_json),
        Command::View { piece_id, kind } => return view(&studio, &piece_id, &kind),
        Command::Linkedin { command } => linkedin(&studio, command).await,
    };
    match result {
        Ok(value) => {
            println!("{value}");
            0
        }
        Err(error) => failed(error),
    }
}

/// The LinkedIn commands. Sign-in prints the authorization URL first, then waits for the
/// browser and prints the connection.
async fn linkedin(studio: &Studio, command: LinkedinCommand) -> Result<Value, StudioError> {
    match command {
        LinkedinCommand::SignIn => {
            let start = studio.linkedin_sign_in_begin()?;
            println!("{}", to_json(start));
            studio.linkedin_sign_in_wait().await.map(to_json)
        }
        LinkedinCommand::Status => studio.linkedin_connection().map(to_json),
        LinkedinCommand::SignOut => studio.linkedin_sign_out().map(to_json),
    }
}

/// Binary output bypasses the JSON result printer.
fn view(studio: &Studio, piece_id: &str, kind: &str) -> i32 {
    use std::io::Write;
    let kind = if kind == "image" {
        syncloom_core::AssetKind::Image
    } else {
        syncloom_core::AssetKind::Carousel
    };
    match studio.view_asset(piece_id, kind) {
        Ok((bytes, _)) => match std::io::stdout().lock().write_all(&bytes) {
            Ok(()) => 0,
            Err(error) => failed(error.into()),
        },
        Err(error) => failed(error),
    }
}

fn failed(error: StudioError) -> i32 {
    eprintln!(
        "{}",
        json!({"error": {"code": error.code(), "message": error.message()}})
    );
    1
}

#[tokio::main]
async fn main() {
    let code = run(Cli::parse()).await;
    std::process::exit(code);
}
