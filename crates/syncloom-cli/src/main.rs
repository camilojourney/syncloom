//! `syncloom`: the studio from the command line, over the same core as the HTTP API.
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "syncloom", about = "Syncloom: thought to reviewed content", version)]
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
    /// Record a contained publish intent, or preview the payload with --dry-run.
    Publish {
        piece_id: String,
        #[arg(long)]
        expected_revision: String,
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() {
    let _cli = Cli::parse();
    std::process::exit(1);
}
