//! Process-boundary fixtures shared by core and API rendering tests.
use std::path::{Path, PathBuf};
pub fn config() -> syncloom_core::RendererConfig {
    syncloom_core::renderer_config(|name| std::env::var(name).ok())
}
#[expect(
    clippy::unwrap_used,
    reason = "Test fixture setup and assertions must fail immediately on unexpected state rather than continue with invalid evidence."
)]
pub fn executable(dir: &Path, body: &str) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;
    let path = dir.join("fake-renderer");
    std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
    path
}
