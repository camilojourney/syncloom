//! The command line over the same core (studio-cli), run as the operator would run it.
#[path = "../../syncloom-core/tests/support/fake_claude.rs"]
mod fake_claude;
#[path = "../../syncloom-core/tests/support/scripted.rs"]
mod scripted;

use fake_claude::Mode;
use serde_json::Value;
use std::{process::Command, sync::Arc};
use tempfile::TempDir;

const THOUGHT: &str = "Agents write better when the spec names one owner per rule.";

struct Run {
    status: i32,
    stdout: String,
    stderr: String,
}

impl Run {
    fn json(&self) -> Value {
        serde_json::from_str(&self.stdout).unwrap_or_else(|e| panic!("{e}: stdout {:?} stderr {:?}", self.stdout, self.stderr))
    }
    fn error(&self) -> Value {
        serde_json::from_str(&self.stderr).unwrap_or_else(|e| panic!("{e}: stderr {:?}", self.stderr))
    }
}

fn syncloom(dir: &TempDir, base_url: &str, key: Option<&str>, args: &[&str]) -> Run {
    let mut command = Command::new(env!("CARGO_BIN_EXE_syncloom"));
    command
        .env_clear()
        .env("HOME", dir.path())
        .env("SYNCLOOM_DATA_DIR", dir.path().join("data"))
        .env("SYNCLOOM_CLAUDE_BASE_URL", base_url)
        .args(args);
    if let Some(key) = key {
        command.env("ANTHROPIC_API_KEY", key);
    }
    let output = command.output().unwrap();
    Run {
        status: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8(output.stdout).unwrap(),
        stderr: String::from_utf8(output.stderr).unwrap(),
    }
}

fn fixture_message(code: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../contracts/studio-api/errors")
        .join(format!("{code}.json"));
    let file: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    file["body"]["error"]["message"].as_str().unwrap().to_owned()
}

// studio-cli: The four commands
#[tokio::test]
async fn thought_show_approve_and_publish_work_end_to_end() {
    let fake = fake_claude::start(Mode::Ok("A draft from the fake provider.".into())).await;
    let dir = TempDir::new().unwrap();
    let base = fake.base_url();

    let run = syncloom(&dir, &base, Some("sk-fake"), &["thought", "--text", THOUGHT, "--channel", "linkedin_text"]);
    assert_eq!(run.status, 0, "{}", run.stderr);
    let created = run.json();
    assert_eq!(created["group_id"].as_str().unwrap().len(), 32);
    let piece = &created["pieces"][0];
    assert_eq!(piece["text"], "A draft from the fake provider.");
    assert_eq!(piece["status"], "pending_review");
    let id = piece["piece_id"].as_str().unwrap();
    let revision = piece["revision"].as_str().unwrap();
    assert_eq!(fake.requests().len(), 1);
    assert_eq!(fake.requests()[0].body["messages"][0]["content"], THOUGHT);

    let run = syncloom(&dir, &base, None, &["show", id]);
    assert_eq!(run.status, 0, "{}", run.stderr);
    assert_eq!(run.json(), *piece);

    let run = syncloom(&dir, &base, None, &["approve", id, "--expected-revision", revision]);
    assert_eq!(run.status, 0, "{}", run.stderr);
    assert_eq!(run.json()["status"], "approved");
    assert_eq!(run.json()["review_decision_id"], format!("review-{id}-{}", &revision[..16]));

    let run = syncloom(&dir, &base, None, &["publish", id, "--expected-revision", revision, "--dry-run"]);
    assert_eq!(run.status, 0, "{}", run.stderr);
    assert_eq!(run.json()["status"], "dry_run");
    assert_eq!(run.json()["payload"]["content"], "A draft from the fake provider.");

    let run = syncloom(&dir, &base, None, &["publish", id, "--expected-revision", revision]);
    assert_eq!(run.status, 0, "{}", run.stderr);
    assert_eq!(run.json()["status"], "contained");
    assert_eq!(run.json()["created"], true);
    let request_id = run.json()["request_id"].as_str().unwrap().to_owned();
    assert!(request_id.starts_with("dispatch-"));

    let run = syncloom(&dir, &base, None, &["publish", id, "--expected-revision", revision]);
    assert_eq!(run.json()["request_id"], request_id);
    assert_eq!(run.json()["created"], false);
    assert_eq!(fake.requests().len(), 1, "publishing makes no provider call");
}

// studio-cli: Same errors as the API
#[tokio::test]
async fn failures_print_the_api_code_and_message_on_stderr_and_exit_1() {
    let fake = fake_claude::start(Mode::Ok("A draft from the fake provider.".into())).await;
    let dir = TempDir::new().unwrap();
    let base = fake.base_url();
    let created = syncloom(&dir, &base, Some("sk-fake"), &["thought", "--text", THOUGHT]).json();
    let id = created["pieces"][0]["piece_id"].as_str().unwrap();
    let revision = created["pieces"][0]["revision"].as_str().unwrap();
    let stale = "0".repeat(64);

    let cases: Vec<(Vec<&str>, &str)> = vec![
        (vec!["thought", "--text", "a b c d e f g"], "thought_too_short"),
        (vec!["show", "thought-000000000000-linkedin_text"], "piece_not_found"),
        (vec!["approve", id, "--expected-revision", &stale], "revision_conflict"),
        (vec!["publish", id, "--expected-revision", revision], "approval_required"),
        (vec!["thought", "--text", THOUGHT, "--channel", "tiktok_video"], "unsupported_channel"),
    ];
    for (args, code) in cases {
        let run = syncloom(&dir, &base, Some("sk-fake"), &args);
        assert_eq!(run.status, 1, "{args:?}: {} {}", run.stdout, run.stderr);
        assert_eq!(run.stdout, "", "{args:?} wrote to stdout");
        let error = run.error();
        assert_eq!(error["error"]["code"], code, "{args:?}");
        assert_eq!(error["error"]["message"], fixture_message(code));
        assert_eq!(error["error"].as_object().unwrap().len(), 2);
    }
    let run = syncloom(&dir, &base, None, &["thought", "--text", THOUGHT]);
    assert_eq!(run.status, 1);
    assert_eq!(run.error()["error"]["code"], "provider_not_configured");
    fake.set(Mode::Status(500));
    let run = syncloom(&dir, &base, Some("sk-fake"), &["thought", "--text", THOUGHT]);
    assert_eq!(run.error()["error"]["code"], "provider_error");
    assert!(!run.stderr.contains("sk-fake"));
}

// studio-cli: Data directory, Shared store with the server
#[tokio::test]
async fn a_piece_created_by_the_cli_is_served_by_the_server_on_the_same_data_dir() {
    let fake = fake_claude::start(Mode::Ok("A draft from the fake provider.".into())).await;
    let dir = TempDir::new().unwrap();
    let created = syncloom(&dir, &fake.base_url(), Some("sk-fake"), &["thought", "--text", THOUGHT]).json();
    let piece = created["pieces"][0].clone();

    let studio = syncloom_core::Studio::new(
        syncloom_core::Store::open(dir.path().join("data")).unwrap(),
        scripted::Scripted::new(scripted::Reply::Fail),
        Arc::new(syncloom_core::SystemClock),
        syncloom_core::Settings::default(),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { syncloom_server::serve(listener, Arc::new(studio)).await.unwrap() });
    let response = reqwest::Client::builder()
        .no_proxy()
        .build()
        .unwrap()
        .get(format!("http://{addr}/v1/pieces/{}", piece["piece_id"].as_str().unwrap()))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert_eq!(response.json::<Value>().await.unwrap(), piece);
}

// studio-cli: Data directory default
#[test]
fn the_data_dir_defaults_to_home_dot_syncloom() {
    let dir = TempDir::new().unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_syncloom"))
        .env_clear()
        .env("HOME", dir.path())
        .args(["show", "thought-000000000000-linkedin_text"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let error: Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(error["error"]["code"], "piece_not_found");
    assert!(dir.path().join(".syncloom").join("pieces").is_dir(), "the default store was opened under HOME");
}
