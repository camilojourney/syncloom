//! `syncloom`: the studio from the command line, over the same core as the HTTP API.
//!
//! Results print as JSON on stdout with exit 0. A failed command prints the core's
//! `{"error":{"code","message"}}` on stderr with exit 1. Configuration errors exit 2.
mod social_connection;
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
    /// Approve the revision and, for visual pieces, the exact byte hashes you reviewed.
    Approve {
        piece_id: String,
        #[arg(long)]
        expected_revision: String,
        /// Account fingerprint from show. Required for connected delivery approval.
        #[arg(long)]
        expected_target: Option<String>,
        #[arg(long, requires = "carousel_sha256")]
        image_sha256: Option<String>,
        #[arg(long, requires = "image_sha256")]
        carousel_sha256: Option<String>,
        /// Repeat in slide order for Meta JPEG assets.
        #[arg(long = "slide-sha256", requires = "image_sha256")]
        slides_sha256: Vec<String>,
    },
    /// Publish an approved piece (dispatched when SYNCLOOM_DELIVERY grants its platform,
    /// else a contained intent), or preview the payload with --dry-run.
    Publish {
        piece_id: String,
        #[arg(long)]
        expected_revision: String,
        #[arg(long)]
        dry_run: bool,
        /// Exact estimate from --dry-run, in integer micro-USD. Required for X delivery.
        #[arg(long)]
        accepted_cost_microusd: Option<u64>,
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
    /// Inspect exact local rendered bytes. This never judges meaning or approves content.
    AssessVisual {
        piece_id: String,
        #[arg(long)]
        expected_revision: String,
        #[arg(long)]
        image_sha256: String,
        #[arg(long)]
        carousel_sha256: String,
        #[arg(long = "slide-sha256")]
        slides_sha256: Vec<String>,
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
        command: ConnectionCommand,
    },
    /// Connect to the explicitly configured Mastodon instance.
    Mastodon {
        #[command(subcommand)]
        command: ConnectionCommand,
    },
    /// Manage Bluesky, X or Threads credentials. No publishing occurs here.
    Connect {
        #[arg(value_parser = ["bluesky", "x", "threads"])]
        platform: String,
        #[command(subcommand)]
        command: social_connection::Command,
    },
    /// Export approved copy and visual files locally. This never posts.
    Export {
        piece_id: String,
        #[arg(long)]
        expected_revision: String,
        #[arg(long, value_parser = ["reddit", "tiktok"])]
        target: String,
    },
}

#[derive(Subcommand)]
enum ConnectionCommand {
    /// Print the authorization URL, wait for the browser, store the token in the Keychain.
    SignIn,
    /// Show the connection state from the Keychain; no LinkedIn call.
    Status,
    /// Delete the token from the Keychain.
    SignOut,
}

#[expect(
    clippy::expect_used,
    reason = "Studio command results contain only supported JSON-serializable values."
)]
fn to_json<T: serde::Serialize>(value: T) -> Value {
    serde_json::to_value(value).expect("studio results serialize")
}

enum StartupError {
    Configuration(String),
    Storage(StudioError),
}
fn initialize() -> Result<Studio, StartupError> {
    let config =
        studio_config(|name| std::env::var(name).ok()).map_err(StartupError::Configuration)?;
    build_studio(config, Settings::default()).map_err(|error| StartupError::Storage(error.into()))
}

#[expect(
    clippy::print_stderr,
    reason = "This binary intentionally writes its documented user-facing output to the terminal."
)]
async fn run(cli: Cli) -> i32 {
    let studio = match initialize() {
        Ok(studio) => studio,
        Err(StartupError::Configuration(error)) => {
            eprintln!("{error}");
            return 2;
        }
        Err(StartupError::Storage(error)) => return failed(error),
    };
    print_result(dispatch(&studio, cli.command).await)
}

enum CommandOutput {
    Json(Value),
    Exit(i32),
}

/// Routing only. Each handler owns request construction for its operation.
async fn dispatch(studio: &Studio, command: Command) -> Result<CommandOutput, StudioError> {
    let result = match command {
        Command::Thought { text, channel } => thought(studio, text, channel).await,
        Command::Show { piece_id } => studio.review_context(&piece_id).map(to_json),
        Command::Approve {
            piece_id,
            expected_revision,
            expected_target,
            image_sha256,
            carousel_sha256,
            slides_sha256,
        } => approve(
            studio,
            &piece_id,
            expected_revision,
            expected_target,
            image_sha256,
            carousel_sha256,
            slides_sha256,
        ),
        Command::Publish {
            piece_id,
            expected_revision,
            dry_run,
            accepted_cost_microusd,
        } => {
            publish(
                studio,
                &piece_id,
                expected_revision,
                dry_run,
                accepted_cost_microusd,
            )
            .await
        }
        Command::Render {
            piece_id,
            expected_revision,
            template,
            slides,
        } => render(studio, &piece_id, expected_revision, template, slides).await,
        Command::AssessVisual {
            piece_id,
            expected_revision,
            image_sha256,
            carousel_sha256,
            slides_sha256,
        } => studio
            .assess_visual(
                &piece_id,
                syncloom_core::visual_quality::AssessmentRequest {
                    expected_revision,
                    expected_assets: syncloom_core::AssetHashes {
                        image_sha256,
                        carousel_sha256,
                        slides_sha256,
                    },
                },
            )
            .map(to_json),
        Command::View { piece_id, kind } => {
            return Ok(CommandOutput::Exit(view(studio, &piece_id, &kind)));
        }
        Command::Linkedin { command } => {
            connection(studio, command, ConnectionPlatform::Linkedin).await
        }
        Command::Mastodon { command } => {
            connection(studio, command, ConnectionPlatform::Mastodon).await
        }
        Command::Connect { platform, command } => {
            social_connection::run(studio, &platform, command).await
        }
        Command::Export {
            piece_id,
            expected_revision,
            target,
        } => export(studio, &piece_id, expected_revision, target),
    };
    result.map(CommandOutput::Json)
}

async fn thought(
    studio: &Studio,
    text: String,
    channels: Vec<String>,
) -> Result<Value, StudioError> {
    studio
        .create_thought(ThoughtRequest { text, channels })
        .await
        .map(to_json)
}
fn approve(
    studio: &Studio,
    piece_id: &str,
    expected_revision: String,
    expected_target: Option<String>,
    image: Option<String>,
    carousel: Option<String>,
    slides_sha256: Vec<String>,
) -> Result<Value, StudioError> {
    let expected_assets =
        image.zip(carousel).map(
            |(image_sha256, carousel_sha256)| syncloom_core::AssetHashes {
                image_sha256,
                carousel_sha256,
                slides_sha256,
            },
        );
    studio
        .review(
            piece_id,
            ReviewRequest {
                status: Some(Status::Approved),
                text: None,
                expected_revision,
                expected_assets,
                expected_target,
            },
        )
        .map(to_json)
}
async fn publish(
    studio: &Studio,
    piece_id: &str,
    expected_revision: String,
    dry_run: bool,
    accepted_cost_microusd: Option<u64>,
) -> Result<Value, StudioError> {
    studio
        .publish(
            piece_id,
            PublishRequest {
                expected_revision,
                dry_run,
                accepted_cost_microusd,
            },
        )
        .await
        .map(to_json)
}
async fn render(
    studio: &Studio,
    piece_id: &str,
    expected_revision: String,
    template: String,
    slides: Vec<String>,
) -> Result<Value, StudioError> {
    studio
        .render(
            piece_id,
            syncloom_core::RenderRequest {
                expected_revision,
                template,
                slides,
            },
        )
        .await
        .map(to_json)
}
fn export(
    studio: &Studio,
    piece_id: &str,
    expected_revision: String,
    target: String,
) -> Result<Value, StudioError> {
    studio
        .export(
            piece_id,
            syncloom_core::manual_export::ExportRequest {
                expected_revision,
                target,
            },
        )
        .map(to_json)
}
#[expect(
    clippy::print_stdout,
    reason = "This binary intentionally writes its documented user-facing output to the terminal."
)]
fn print_result(result: Result<CommandOutput, StudioError>) -> i32 {
    match result {
        Ok(CommandOutput::Json(value)) => {
            println!("{value}");
            0
        }
        Ok(CommandOutput::Exit(code)) => code,
        Err(error) => failed(error),
    }
}

enum ConnectionPlatform {
    Linkedin,
    Mastodon,
}

/// Both platforms share command output and waiting semantics, not their protocol.
#[expect(
    clippy::print_stdout,
    reason = "This binary intentionally writes its documented user-facing output to the terminal."
)]
async fn connection(
    studio: &Studio,
    command: ConnectionCommand,
    platform: ConnectionPlatform,
) -> Result<Value, StudioError> {
    use ConnectionPlatform::*;
    match command {
        ConnectionCommand::SignIn => {
            let start = match platform {
                Linkedin => studio.linkedin_sign_in_begin()?,
                Mastodon => studio.mastodon_sign_in_begin().await?,
            };
            println!("{}", to_json(start));
            match platform {
                Linkedin => studio.linkedin_sign_in_wait().await.map(to_json),
                Mastodon => studio.mastodon_sign_in_wait().await.map(to_json),
            }
        }
        ConnectionCommand::Status => match platform {
            Linkedin => studio.linkedin_connection().map(to_json),
            Mastodon => studio.mastodon_connection().map(to_json),
        },
        ConnectionCommand::SignOut => match platform {
            Linkedin => studio.linkedin_sign_out().map(to_json),
            Mastodon => studio.mastodon_sign_out().map(to_json),
        },
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

#[expect(
    clippy::print_stderr,
    reason = "This binary intentionally writes its documented user-facing output to the terminal."
)]
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
