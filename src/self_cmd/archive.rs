//! Pure-Rust `.tar.gz` extraction + sha256 helpers for `naba self update` (SPEC-SELF-004).
//!
//! cargo-dist ships `unix-archive = ".tar.gz"` (not `.tar.xz`) precisely so the updater extracts
//! with pure-Rust `flate2` + `tar` — no C `xz` codec. The archive holds the binary under an
//! enclosing `naba-<triple>/naba` directory; [`extract_binary`] tolerates that (and a bare
//! top-level `naba`).

use std::io::Read;
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;
use sha2::{Digest, Sha256};

use crate::error::{AppError, AppResult};

/// Lowercase-hex sha256 of `data` (SPEC-SELF-004).
pub fn sha256_hex(data: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(data);
    hex::encode(h.finalize())
}

/// Parse a `.sha256` sidecar. Accepts either `"<64-hex>  <name>"` (the `shasum -a 256` /
/// `sha256sum` format, two spaces) or a bare 64-hex line. Returns the lowercase hex digest, or
/// `None` when the first whitespace-delimited token is not exactly 64 hex chars.
pub fn parse_sha256_file(content: &str) -> Option<String> {
    let token = content.split_whitespace().next()?;
    if token.len() == 64 && token.bytes().all(|b| b.is_ascii_hexdigit()) {
        Some(token.to_ascii_lowercase())
    } else {
        None
    }
}

/// Extract the `bin_name` executable from a `.tar.gz` archive into `scratch`, returning the path
/// to the written file (SPEC-SELF-004).
///
/// The matching entry is the one whose **final path component** equals `bin_name` — this
/// tolerates the cargo-dist enclosing `naba-<triple>/naba` layout as well as a bare `naba` at the
/// archive root. On unix the extracted file is marked executable (`0o755`).
pub fn extract_binary(archive_bytes: &[u8], bin_name: &str, scratch: &Path) -> AppResult<PathBuf> {
    let gz = GzDecoder::new(archive_bytes);
    let mut tar = tar::Archive::new(gz);
    let entries = tar
        .entries()
        .map_err(|e| AppError::general(format!("open archive: {e}")))?;

    for entry in entries {
        let mut entry = entry.map_err(|e| AppError::general(format!("read archive entry: {e}")))?;
        let path = entry
            .path()
            .map_err(|e| AppError::general(format!("bad archive path: {e}")))?
            .into_owned();
        let is_match = path
            .file_name()
            .map(|n| n.to_string_lossy() == bin_name)
            .unwrap_or(false);
        if !is_match {
            continue;
        }

        std::fs::create_dir_all(scratch)
            .map_err(|e| AppError::file_io(format!("mkdir scratch: {e}")))?;
        let out = scratch.join(bin_name);
        let mut bytes = Vec::new();
        entry
            .read_to_end(&mut bytes)
            .map_err(|e| AppError::general(format!("read archive member: {e}")))?;
        std::fs::write(&out, &bytes)
            .map_err(|e| AppError::file_io(format!("write extracted binary: {e}")))?;
        set_exec(&out);
        return Ok(out);
    }

    Err(AppError::general(format!(
        "binary {bin_name:?} not found in archive"
    )))
}

/// Mark a file executable (`0o755`) on unix; no-op elsewhere.
#[cfg(unix)]
fn set_exec(path: &Path) {
    use std::os::unix::fs::PermissionsExt;
    let _ = std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755));
}

#[cfg(not(unix))]
fn set_exec(_path: &Path) {}

#[cfg(test)]
mod tests {
    use super::*;
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write;

    /// Build an in-memory `.tar.gz` with a single file at `inner_path` holding `contents`.
    fn make_targz(inner_path: &str, contents: &[u8]) -> Vec<u8> {
        let mut tar_builder = tar::Builder::new(Vec::new());
        let mut header = tar::Header::new_gnu();
        header.set_size(contents.len() as u64);
        header.set_mode(0o755);
        header.set_cksum();
        tar_builder
            .append_data(&mut header, inner_path, contents)
            .unwrap();
        let tar_bytes = tar_builder.into_inner().unwrap();

        let mut enc = GzEncoder::new(Vec::new(), Compression::default());
        enc.write_all(&tar_bytes).unwrap();
        enc.finish().unwrap()
    }

    #[test]
    fn sha256_hex_known_vectors() {
        // sha256("") and sha256("abc").
        assert_eq!(
            sha256_hex(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn parse_sha256_accepts_shasum_and_bare() {
        let hex = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
        // shasum -a 256 format: "<hex>  <name>".
        assert_eq!(
            parse_sha256_file(&format!("{hex}  naba-x86_64-apple-darwin.tar.gz\n")),
            Some(hex.to_string())
        );
        // bare hex.
        assert_eq!(parse_sha256_file(&format!("{hex}\n")), Some(hex.to_string()));
        // uppercase normalized to lowercase.
        assert_eq!(
            parse_sha256_file(&hex.to_uppercase()),
            Some(hex.to_string())
        );
    }

    #[test]
    fn parse_sha256_rejects_bad() {
        assert_eq!(parse_sha256_file(""), None);
        assert_eq!(parse_sha256_file("not-a-hash  file"), None);
        assert_eq!(parse_sha256_file("abcd"), None); // too short
        // 64 chars but not all hex.
        assert_eq!(parse_sha256_file(&"z".repeat(64)), None);
    }

    #[test]
    fn extract_tolerates_enclosing_triple_dir() {
        let d = std::env::temp_dir().join(format!("naba-archive-test-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        let payload = b"#!/bin/sh\necho naba\n";
        let targz = make_targz("naba-aarch64-apple-darwin/naba", payload);

        let out = extract_binary(&targz, "naba", &d).unwrap();
        assert_eq!(out, d.join("naba"));
        assert_eq!(std::fs::read(&out).unwrap(), payload);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(&out).unwrap().permissions().mode() & 0o111;
            assert_ne!(mode, 0, "extracted binary should be executable");
        }
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn extract_bare_top_level_binary() {
        let d = std::env::temp_dir().join(format!("naba-archive-bare-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        let targz = make_targz("naba", b"payload");
        let out = extract_binary(&targz, "naba", &d).unwrap();
        assert_eq!(std::fs::read(&out).unwrap(), b"payload");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn extract_missing_binary_errors() {
        let d = std::env::temp_dir().join(format!("naba-archive-miss-{}", std::process::id()));
        let targz = make_targz("naba-aarch64-apple-darwin/other-tool", b"x");
        let err = extract_binary(&targz, "naba", &d).unwrap_err();
        assert!(err.message.contains("not found in archive"));
        let _ = std::fs::remove_dir_all(&d);
    }
}
