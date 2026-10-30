//! The command line over the same core (studio-cli), run as the operator would run it.
#[path = "../../syncloom-core/tests/support/fake_claude.rs"]
mod fake_claude;
#[path = "../../syncloom-core/tests/support/fake_linkedin.rs"]
mod fake_linkedin;
#[path = "../../syncloom-core/tests/support/scripted.rs"]
mod scripted;

use fake_claude::Mode;
use fake_linkedin::{AuthMode, PostMode, TokenMode};
use serde_json::Value;
use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
    sync::Arc,
};
use tempfile::TempDir;

const THOUGHT: &str = "Agents write better when the spec names one owner per rule.";

struct Run {
    status: i32,
    stdout: String,
    stderr: String,
}

impl Run {
    fn json(&self) -> Value {
        serde_json::from_str(&self.stdout)
            .unwrap_or_else(|e| panic!("{e}: stdout {:?} stderr {:?}", self.stdout, self.stderr))
    }
    fn error(&self) -> Value {
        serde_json::from_str(&self.stderr)
            .unwrap_or_else(|e| panic!("{e}: stderr {:?}", self.stderr))
    }
}

fn command(dir: &TempDir, base_url: &str, key: Option<&str>, args: &[&str]) -> Command {
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
    command
}

fn finish(output: std::process::Output) -> Run {
    Run {
        status: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8(output.stdout).unwrap(),
        stderr: String::from_utf8(output.stderr).unwrap(),
    }
}

fn syncloom(dir: &TempDir, base_url: &str, key: Option<&str>, args: &[&str]) -> Run {
    finish(command(dir, base_url, key, args).output().unwrap())
}

/// The LinkedIn environment of one test: both bases at the fake, a fixed free redirect port,
/// a test-only Keychain service that is deleted when the value drops, app credentials when
/// `configured`, and delivery granted when `granted`.
struct LinkedinEnv {
    vars: Vec<(String, String)>,
    service: String,
}

impl LinkedinEnv {
    fn new(fake: &fake_linkedin::FakeLinkedin, configured: bool, granted: bool) -> Self {
        let port = std::net::TcpListener::bind("127.0.0.1:0")
            .unwrap()
            .local_addr()
            .unwrap()
            .port();
        let service = format!("syncloom-test-cli-{}-{}", std::process::id(), uuid_like());
        // The Keychain lives under the real HOME; the data directory stays the temp one.
        let mut vars = vec![
            ("HOME".to_owned(), std::env::var("HOME").unwrap()),
            (
                "SYNCLOOM_LINKEDIN_AUTH_BASE_URL".to_owned(),
                fake.base_url(),
            ),
            ("SYNCLOOM_LINKEDIN_API_BASE_URL".to_owned(), fake.base_url()),
            (
                "SYNCLOOM_LINKEDIN_REDIRECT_URL".to_owned(),
                format!("http://127.0.0.1:{port}/linkedin/callback"),
            ),
            ("SYNCLOOM_KEYCHAIN_SERVICE".to_owned(), service.clone()),
        ];
        if configured {
            vars.push((
                "SYNCLOOM_LINKEDIN_CLIENT_ID".to_owned(),
                "fake-client-id".to_owned(),
            ));
            vars.push((
                "SYNCLOOM_LINKEDIN_CLIENT_SECRET".to_owned(),
                "client-secret-axolotl".to_owned(),
            ));
        }
        if granted {
            vars.push(("SYNCLOOM_DELIVERY".to_owned(), "linkedin".to_owned()));
        }
        Self { vars, service }
    }
    fn apply(&self, command: &mut Command) {
        for (name, value) in &self.vars {
            command.env(name, value);
        }
    }
    /// Whether the Keychain holds the item, by its attributes only. The secret is readable only
    /// by the binary that stored it (the Keychain binds the item to that code hash), so this
    /// test process asks Apple's own tool for presence and lets `syncloom` read the value.
    fn item_exists(&self) -> bool {
        Command::new("/usr/bin/security")
            .args([
                "find-generic-password",
                "-s",
                &self.service,
                "-a",
                "linkedin",
            ])
            .output()
            .unwrap()
            .status
            .success()
    }
}

impl Drop for LinkedinEnv {
    fn drop(&mut self) {
        let _ = Command::new("/usr/bin/security")
            .args([
                "delete-generic-password",
                "-s",
                &self.service,
                "-a",
                "linkedin",
            ])
            .output();
    }
}

fn uuid_like() -> String {
    format!(
        "{:x}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    )
}

fn syncloom_linkedin(dir: &TempDir, base: &str, env: &LinkedinEnv, args: &[&str]) -> Run {
    let mut command = command(dir, base, Some("sk-fake"), args);
    env.apply(&mut command);
    finish(command.output().unwrap())
}

/// Run `linkedin sign-in`: read the URL line, play the browser, collect the rest.
async fn sign_in(dir: &TempDir, base: &str, env: &LinkedinEnv) -> (Run, Value) {
    let mut command = command(dir, base, Some("sk-fake"), &["linkedin", "sign-in"]);
    env.apply(&mut command);
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let mut stdout = BufReader::new(child.stdout.take().unwrap());
    let mut first = String::new();
    stdout.read_line(&mut first).unwrap();
    let start: Value = serde_json::from_str(&first).unwrap_or_else(|e| panic!("{e}: {first:?}"));
    if let Some(url) = start["authorization_url"].as_str() {
        reqwest::Client::builder()
            .no_proxy()
            .build()
            .unwrap()
            .get(url)
            .send()
            .await
            .unwrap();
    }
    let mut rest = String::new();
    std::io::Read::read_to_string(&mut stdout, &mut rest).unwrap();
    let output = child.wait_with_output().unwrap();
    (
        Run {
            status: output.status.code().unwrap_or(-1),
            stdout: rest,
            stderr: String::from_utf8(output.stderr).unwrap(),
        },
        start,
    )
}

fn fixture_message(code: &str) -> String {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../contracts/studio-api/errors")
        .join(format!("{code}.json"));
    let file: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    file["body"]["error"]["message"]
        .as_str()
        .unwrap()
        .to_owned()
}

// The fake endpoint lives in this runtime while the CLI process runs; the runtime must keep
// serving while `Command::output` blocks, so these tests use worker threads.

// studio-cli: The four commands
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn thought_show_approve_and_publish_work_end_to_end() {
    let fake = fake_claude::start(Mode::Ok("A draft from the fake provider.".into())).await;
    let dir = TempDir::new().unwrap();
    let base = fake.base_url();

    let run = syncloom(
        &dir,
        &base,
        Some("sk-fake"),
        &["thought", "--text", THOUGHT, "--channel", "linkedin_text"],
    );
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

    let run = syncloom(
        &dir,
        &base,
        None,
        &["approve", id, "--expected-revision", revision],
    );
    assert_eq!(run.status, 0, "{}", run.stderr);
    assert_eq!(run.json()["status"], "approved");
    assert_eq!(
        run.json()["review_decision_id"],
        format!("review-{id}-{}", &revision[..16])
    );

    let run = syncloom(
        &dir,
        &base,
        None,
        &["publish", id, "--expected-revision", revision, "--dry-run"],
    );
    assert_eq!(run.status, 0, "{}", run.stderr);
    assert_eq!(run.json()["status"], "dry_run");
    assert_eq!(
        run.json()["payload"]["content"],
        "A draft from the fake provider."
    );

    let run = syncloom(
        &dir,
        &base,
        None,
        &["publish", id, "--expected-revision", revision],
    );
    assert_eq!(run.status, 0, "{}", run.stderr);
    assert_eq!(run.json()["status"], "contained");
    assert_eq!(run.json()["created"], true);
    let request_id = run.json()["request_id"].as_str().unwrap().to_owned();
    assert!(request_id.starts_with("dispatch-"));

    let run = syncloom(
        &dir,
        &base,
        None,
        &["publish", id, "--expected-revision", revision],
    );
    assert_eq!(run.json()["request_id"], request_id);
    assert_eq!(run.json()["created"], false);
    assert_eq!(
        fake.requests().len(),
        1,
        "publishing makes no provider call"
    );
}

// studio-cli: Same errors as the API
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn failures_print_the_api_code_and_message_on_stderr_and_exit_1() {
    let fake = fake_claude::start(Mode::Ok("A draft from the fake provider.".into())).await;
    let dir = TempDir::new().unwrap();
    let base = fake.base_url();
    let created = syncloom(
        &dir,
        &base,
        Some("sk-fake"),
        &["thought", "--text", THOUGHT],
    )
    .json();
    let id = created["pieces"][0]["piece_id"].as_str().unwrap();
    let revision = created["pieces"][0]["revision"].as_str().unwrap();
    let stale = "0".repeat(64);

    let cases: Vec<(Vec<&str>, &str)> = vec![
        (
            vec!["thought", "--text", "a b c d e f g"],
            "thought_too_short",
        ),
        (
            vec!["show", "thought-000000000000-linkedin_text"],
            "piece_not_found",
        ),
        (
            vec!["approve", id, "--expected-revision", &stale],
            "revision_conflict",
        ),
        (
            vec!["publish", id, "--expected-revision", revision],
            "approval_required",
        ),
        (
            vec!["thought", "--text", THOUGHT, "--channel", "tiktok_video"],
            "unsupported_channel",
        ),
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
    let run = syncloom(
        &dir,
        &base,
        Some("sk-fake"),
        &["thought", "--text", THOUGHT],
    );
    assert_eq!(run.error()["error"]["code"], "provider_error");
    assert!(!run.stderr.contains("sk-fake"));
}

// studio-cli: Data directory, Shared store with the server
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn a_piece_created_by_the_cli_is_served_by_the_server_on_the_same_data_dir() {
    let fake = fake_claude::start(Mode::Ok("A draft from the fake provider.".into())).await;
    let dir = TempDir::new().unwrap();
    let created = syncloom(
        &dir,
        &fake.base_url(),
        Some("sk-fake"),
        &["thought", "--text", THOUGHT],
    )
    .json();
    let piece = created["pieces"][0].clone();

    let studio = syncloom_core::Studio::new(
        syncloom_core::Store::open(dir.path().join("data")).unwrap(),
        scripted::Scripted::new(scripted::Reply::Fail),
        Arc::new(syncloom_core::SystemClock),
        syncloom_core::Settings::default(),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        syncloom_server::serve(listener, Arc::new(studio))
            .await
            .unwrap()
    });
    let response = reqwest::Client::builder()
        .no_proxy()
        .build()
        .unwrap()
        .get(format!(
            "http://{addr}/v1/pieces/{}",
            piece["piece_id"].as_str().unwrap()
        ))
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
    assert!(
        dir.path().join(".syncloom").join("pieces").is_dir(),
        "the default store was opened under HOME"
    );
}

// studio-cli: Sign in, publish, repeat, sign out (through the real Keychain under a test service)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn linkedin_sign_in_publish_repeat_and_sign_out_work_end_to_end() {
    let claude = fake_claude::start(Mode::Ok("A draft from the fake provider.".into())).await;
    let linkedin = fake_linkedin::start(
        AuthMode::Consent,
        TokenMode::ok(),
        PostMode::Posted("urn:li:share:7000000000000000001".to_owned()),
    )
    .await;
    let dir = TempDir::new().unwrap();
    let base = claude.base_url();
    let env = LinkedinEnv::new(&linkedin, true, true);

    let status = syncloom_linkedin(&dir, &base, &env, &["linkedin", "status"]);
    assert_eq!(status.status, 0, "{}", status.stderr);
    assert_eq!(status.json()["status"], "signed_out");

    let (run, start) = sign_in(&dir, &base, &env).await;
    assert_eq!(run.status, 0, "{} {}", run.stdout, run.stderr);
    assert_eq!(start["status"], "awaiting_consent");
    assert!(
        start["authorization_url"]
            .as_str()
            .unwrap()
            .starts_with(&linkedin.base_url())
    );
    let last = run.stdout.lines().last().unwrap();
    let connection: Value = serde_json::from_str(last).unwrap();
    assert_eq!(connection["status"], "signed_in");
    assert_eq!(connection["expires_at"].as_str().unwrap().len(), 20);
    assert!(!run.stdout.contains("fake-access-token"), "{}", run.stdout);
    assert!(
        env.item_exists(),
        "the token is in the Keychain under the test service"
    );
    let all_files: String = walk(dir.path());
    assert!(
        !all_files.contains("fake-access-token"),
        "the token reached a file"
    );

    let status = syncloom_linkedin(&dir, &base, &env, &["linkedin", "status"]);
    assert_eq!(status.json()["status"], "signed_in");

    let created = syncloom_linkedin(&dir, &base, &env, &["thought", "--text", THOUGHT]).json();
    let piece = &created["pieces"][0];
    let id = piece["piece_id"].as_str().unwrap();
    let revision = piece["revision"].as_str().unwrap();
    syncloom_linkedin(
        &dir,
        &base,
        &env,
        &["approve", id, "--expected-revision", revision],
    );
    let published = syncloom_linkedin(
        &dir,
        &base,
        &env,
        &["publish", id, "--expected-revision", revision],
    );
    assert_eq!(published.status, 0, "{}", published.stderr);
    assert_eq!(published.json()["status"], "posted");
    assert_eq!(
        published.json()["post_id"],
        "urn:li:share:7000000000000000001"
    );
    assert_eq!(
        published.json()["post_url"],
        "https://www.linkedin.com/feed/update/urn:li:share:7000000000000000001/"
    );
    assert_eq!(published.json()["created"], true);
    let again = syncloom_linkedin(
        &dir,
        &base,
        &env,
        &["publish", id, "--expected-revision", revision],
    );
    assert_eq!(again.json()["post_id"], "urn:li:share:7000000000000000001");
    assert_eq!(again.json()["created"], false);
    assert_eq!(linkedin.posts().len(), 1, "one post for two publishes");
    assert_eq!(
        linkedin.posts()[0].body["commentary"],
        "A draft from the fake provider."
    );

    let out = syncloom_linkedin(&dir, &base, &env, &["linkedin", "sign-out"]);
    assert_eq!(out.status, 0, "{}", out.stderr);
    assert_eq!(out.json()["status"], "signed_out");
    assert!(!env.item_exists(), "sign-out deleted the Keychain item");
}

// studio-cli: Core errors for LinkedIn (linkedin_not_configured, sign_in_denied)
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn linkedin_failures_print_the_api_code_and_message() {
    let claude = fake_claude::start(Mode::Ok("x".into())).await;
    let linkedin = fake_linkedin::start(AuthMode::Deny, TokenMode::ok(), PostMode::NoId).await;
    let dir = TempDir::new().unwrap();
    let base = claude.base_url();

    let unconfigured = LinkedinEnv::new(&linkedin, false, false);
    let run = syncloom_linkedin(&dir, &base, &unconfigured, &["linkedin", "sign-in"]);
    assert_eq!(run.status, 1, "{} {}", run.stdout, run.stderr);
    assert_eq!(run.stdout, "");
    assert_eq!(run.error()["error"]["code"], "linkedin_not_configured");
    assert_eq!(
        run.error()["error"]["message"],
        fixture_message("linkedin_not_configured")
    );

    let env = LinkedinEnv::new(&linkedin, true, false);
    let (run, start) = sign_in(&dir, &base, &env).await;
    assert_eq!(start["status"], "awaiting_consent");
    assert_eq!(run.status, 1, "{} {}", run.stdout, run.stderr);
    assert_eq!(run.error()["error"]["code"], "sign_in_denied");
    assert_eq!(
        run.error()["error"]["message"],
        fixture_message("sign_in_denied")
    );
    assert!(!run.stderr.contains("client-secret-axolotl"));
    assert!(!env.item_exists());
    assert!(linkedin.token_requests().is_empty());
}

/// Every file under the data directory as one string.
fn walk(root: &std::path::Path) -> String {
    let mut out = String::new();
    if let Ok(entries) = std::fs::read_dir(root) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                out.push_str(&walk(&path));
            } else if let Ok(text) = std::fs::read_to_string(&path) {
                out.push_str(&text);
            }
        }
    }
    out
}
