use std::process::Command;

// `lib.rs` embeds the build's git commit as `GH_COMMIT_SHA` via
// `env!("COMMIT_SHA")`. CI sets `COMMIT_SHA` in the environment (github.sha); in
// that case `env!` reads it directly and this script is a no-op. For local
// builds the var is usually unset, so derive it from `git rev-parse HEAD` here
// rather than hard-failing the compile (or relying on a hand-written .env that
// goes stale). Falls back to "unknown" if git isn't available.
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
