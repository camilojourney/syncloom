#[path = "../../syncloom-core/tests/support/scripted.rs"]
mod scripted;
use std::{process::Command, sync::Arc};
use syncloom_core::{Settings, Store, Studio, SystemClock, ThoughtRequest};

#[tokio::test]
async fn cli_render_view_approve_and_missing_renderer() {
    let dir = tempfile::tempdir().unwrap();
    let studio = Studio::new(
        Store::open(dir.path()).unwrap(),
        scripted::Scripted::new(scripted::text("Build with evidence.")),
        Arc::new(SystemClock),
        Settings::default(),
    );
    let piece = studio
        .create_thought(ThoughtRequest {
            text: "Build with evidence.".into(),
            channels: vec!["linkedin_text".into()],
        })
        .await
        .unwrap()
        .pieces
        .remove(0);
    let run = |args: &[&str], missing: bool| {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_syncloom"));
        cmd.env_clear()
            .env("HOME", std::env::var("HOME").unwrap())
            .env("SYNCLOOM_DATA_DIR", dir.path())
            .args(args);
        if missing {
            cmd.env("SYNCLOOM_RENDERER_PATH", "/nonexistent/browser");
        }
        cmd.output().unwrap()
    };
    let failed = run(
        &[
            "render",
            &piece.piece_id,
            "--expected-revision",
            &piece.revision,
            "--template",
            "midnight",
            "--slide",
            "Build with evidence.",
        ],
        true,
    );
    assert_eq!(failed.status.code(), Some(1));
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&failed.stderr).unwrap()["error"]["code"],
        "renderer_missing"
    );
    let current = studio.show(&piece.piece_id).unwrap();
    let output = run(
        &[
            "render",
            &piece.piece_id,
            "--expected-revision",
            &current.revision,
            "--template",
            "midnight",
            "--slide",
            "Build with evidence.",
            "--slide",
            "Review exact bytes.",
        ],
        false,
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let rendered: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let revision = rendered["revision"].as_str().unwrap();
    let image = run(&["view", &piece.piece_id, "--kind", "image"], false);
    assert!(image.status.success());
    assert!(image.stdout.starts_with(b"\x89PNG\r\n\x1a\n"));
    let pdf = run(&["view", &piece.piece_id, "--kind", "carousel"], false);
    assert!(pdf.stdout.starts_with(b"%PDF-"));
    assert!(
        run(
            &[
                "approve",
                &piece.piece_id,
                "--expected-revision",
                revision,
                "--image-sha256",
                rendered["assets"]["image_sha256"].as_str().unwrap(),
                "--carousel-sha256",
                rendered["assets"]["carousel_sha256"].as_str().unwrap()
            ],
            false
        )
        .status
        .success()
    );
}
