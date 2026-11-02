#[path = "../../syncloom-core/tests/support/scripted.rs"]
mod scripted;
use serde_json::Value;
use std::{process::Command, sync::Arc};
use syncloom_core::{FixedClock, Settings, Store, Studio, ThoughtRequest};

#[test]
fn social_connection_help_and_publish_confirmation_are_discoverable() {
    for args in [
        vec!["connect", "--help"],
        vec!["connect", "bluesky", "sign-in", "--help"],
        vec!["publish", "--help"],
        vec!["approve", "--help"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_syncloom"))
            .args(&args)
            .output()
            .unwrap();
        assert!(output.status.success());
        let help = String::from_utf8(output.stdout).unwrap();
        if args[0] == "approve" {
            assert!(help.contains("--expected-target"));
        } else if args[0] == "publish" {
            assert!(help.contains("--accepted-cost-microusd"));
        } else if args.len() == 2 {
            assert!(
                help.contains("bluesky") && help.contains("threads") && help.contains("refresh")
            );
        } else {
            assert!(help.contains("--identifier"));
            assert!(help.contains("stdin"));
        }
    }
}

#[tokio::test]
async fn x_cli_dry_run_shows_separate_link_price_and_writes_no_intent() {
    let dir = tempfile::TempDir::new().unwrap();
    let studio = Studio::new(
        Store::open(dir.path()).unwrap(),
        scripted::Scripted::new(scripted::text(
            "Read https://example.invalid for the evidence.",
        )),
        Arc::new(FixedClock(time::macros::datetime!(2026-10-31 02:00 UTC))),
        Settings::default(),
    );
    let p = studio
        .create_thought(ThoughtRequest {
            text: "Build with evidence.".into(),
            channels: vec!["x".into()],
        })
        .await
        .unwrap()
        .pieces
        .remove(0);
    let output = Command::new(env!("CARGO_BIN_EXE_syncloom"))
        .env_clear()
        .env("SYNCLOOM_DATA_DIR", dir.path())
        .args([
            "publish",
            &p.piece_id,
            "--expected-revision",
            &p.revision,
            "--dry-run",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["status"], "dry_run");
    assert_eq!(result["cost_estimate"]["microusd"], 200000);
    assert_eq!(
        result["cost_estimate"]["kind"],
        "estimate_not_actual_charge"
    );
    assert_eq!(
        std::fs::read_dir(dir.path().join("outbox"))
            .unwrap()
            .count(),
        0
    );
}
