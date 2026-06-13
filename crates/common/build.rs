use std::process::Command;

// `lib.rs` embeds the build's git commit as `GH_COMMIT_SHA` via
// `env!("COMMIT_SHA")`, and that read is gated behind `#[cfg(test)]` — so only
// test builds need `COMMIT_SHA` to resolve at compile time. This script provides
// the value for those test builds: CI sets `COMMIT_SHA` in the environment
// (github.sha), in which case `env!` reads it directly and this script is a
// no-op; otherwise (local test runs) it derives the SHA from `git rev-parse
// HEAD` rather than hard-failing the compile (or relying on a hand-written .env
// that goes stale). Falls back to "unknown" if git isn't available. A build
// script can't tell the test profile from a prod build, so the derivation runs
// unconditionally — it's harmless for prod, which no longer reads the var.
fn main() {
    println!("cargo:rerun-if-env-changed=COMMIT_SHA");

    if std::env::var_os("COMMIT_SHA").is_none() {
        let sha = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .ok()
            .filter(|out| out.status.success())
            .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
            .filter(|sha| !sha.is_empty())
            .unwrap_or_else(|| "unknown".to_string());
        println!("cargo:rustc-env=COMMIT_SHA={sha}");
    }
}
