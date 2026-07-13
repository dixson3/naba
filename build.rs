// Build-time version injection (SPEC-VERSION-BUILD-001, M3).
// Captures Version / Commit / Date into compile-time env vars, replacing Go's ldflags.
use std::process::Command;

fn git(args: &[&str]) -> Option<String> {
    let out = Command::new("git").args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8(out.stdout).ok()?.trim().to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

fn utc_date() -> String {
    // Shell out to `date -u` to avoid pulling a date/time crate (near-zero-dep posture).
    Command::new("date")
        .args(["-u", "+%Y-%m-%dT%H:%M:%SZ"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string())
}

fn main() {
    let version =
        git(&["describe", "--tags", "--always", "--dirty"]).unwrap_or_else(|| "dev".to_string());
    let commit = git(&["rev-parse", "--short", "HEAD"]).unwrap_or_else(|| "none".to_string());
    let date = utc_date();

    // The compile target triple (e.g. aarch64-apple-darwin) — cargo sets $TARGET for build
    // scripts. `naba self update` matches this against the dist-manifest artifact target_triples
    // (SPEC-SELF-004). Fallback `unknown` should never occur under cargo.
    let host_triple = std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());

    println!("cargo:rustc-env=NABA_VERSION={version}");
    println!("cargo:rustc-env=NABA_COMMIT={commit}");
    println!("cargo:rustc-env=NABA_DATE={date}");
    println!("cargo:rustc-env=NABA_HOST_TRIPLE={host_triple}");

    // Re-run when the checked-out commit changes so the injected values stay fresh.
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs");
}
