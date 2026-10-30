//! The binary's startup rules: bind address, base URL, the listening line (studio-api
//! "Loopback bind"; thought-drafting "Base URL").
use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
};
use syncloom_server::{DEFAULT_ADDRESS, local_address};
use tempfile::TempDir;

// studio-api: Loopback bind, Default address and Other addresses refused (the pure rule)
#[test]
fn only_127_0_0_1_addresses_are_accepted() {
    assert_eq!(DEFAULT_ADDRESS, "127.0.0.1:8790");
    assert_eq!(local_address(None).unwrap().to_string(), "127.0.0.1:8790");
    assert_eq!(local_address(Some("127.0.0.1:0")).unwrap().port(), 0);
    for refused in [
        "0.0.0.0:8790",
        "127.0.0.2:8790",
        "192.168.1.1:8790",
        "[::1]:8790",
        "localhost:8790",
        "nonsense",
    ] {
        let error = local_address(Some(refused)).expect_err(refused);
        assert!(error.contains("SYNCLOOM_ADDR"), "{refused}: {error}");
    }
}

fn binary(dir: &TempDir) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_syncloom-server"));
    command
        .env_clear()
        .env("HOME", dir.path())
        .env("SYNCLOOM_DATA_DIR", dir.path().join("data"))
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    command
}

// studio-api: Loopback bind, Other addresses refused (the process)
#[test]
fn the_process_refuses_other_bind_addresses_naming_the_variable() {
    let dir = TempDir::new().unwrap();
    for refused in ["0.0.0.0:8790", "[::1]:8790", "localhost:8790"] {
        let output = binary(&dir)
            .env("SYNCLOOM_ADDR", refused)
            .output()
            .unwrap();
        assert!(!output.status.success(), "{refused} was accepted");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("SYNCLOOM_ADDR"), "{refused}: {stderr}");
    }
}

// thought-drafting: Base URL, refused values
#[test]
fn the_process_refuses_a_bad_provider_base_url_naming_the_variable() {
    let dir = TempDir::new().unwrap();
    for refused in ["http://example.com", "ftp://x", "https://u:p@api.anthropic.com", "https://api.anthropic.com/?q=1"] {
        let output = binary(&dir)
            .env("SYNCLOOM_ADDR", "127.0.0.1:0")
            .env("SYNCLOOM_CLAUDE_BASE_URL", refused)
            .output()
            .unwrap();
        assert!(!output.status.success(), "{refused} was accepted");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(stderr.contains("SYNCLOOM_CLAUDE_BASE_URL"), "{refused}: {stderr}");
    }
}

// studio-api: Loopback bind, Bound address announced
#[test]
fn the_process_announces_the_bound_address_in_one_json_line() {
    let dir = TempDir::new().unwrap();
    let mut child = binary(&dir)
        .env("SYNCLOOM_ADDR", "127.0.0.1:0")
        .env("SYNCLOOM_CLAUDE_BASE_URL", "http://127.0.0.1:9")
        .spawn()
        .unwrap();
    let mut line = String::new();
    BufReader::new(child.stdout.take().unwrap())
        .read_line(&mut line)
        .unwrap();
    child.kill().unwrap();
    child.wait().unwrap();
    let json: serde_json::Value = serde_json::from_str(&line).unwrap_or_else(|_| panic!("not JSON: {line:?}"));
    assert_eq!(json["message"], "listening");
    let address = json["address"].as_str().unwrap();
    assert!(address.starts_with("127.0.0.1:"), "{address}");
    assert_ne!(address, "127.0.0.1:0");
    assert_eq!(json["version"], env!("CARGO_PKG_VERSION"));
}
