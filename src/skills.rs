//! `skills` command group (SPEC-SKILLS-001..006, §3.11): install / upgrade / remove /
//! status of the binary-embedded skill trees. Ports Go's `internal/cli/skills.go`.
//!
//! The embed primitives (hashing, marker injection, status) live in [`crate::embed`]
//! (Issue 4.0); this module is the command behavior + message strings + destination
//! resolution layered on top. `resolve_dest` is shared with `doctor` (Issue 4.3).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::embed;
use crate::error::{AppError, AppResult};
use crate::output;
use crate::version;

/// One of the three write/remove deployment modes driven by [`run`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Install,
    Upgrade,
    Remove,
}

impl Mode {
    /// The action verb for the `--json` envelope (SPEC-JSON-006).
    fn action(self) -> &'static str {
        match self {
            Mode::Install => "install",
            Mode::Upgrade => "upgrade",
            Mode::Remove => "remove",
        }
    }
}

/// Resolved flags a `skills` invocation carries (mirrors Go's package-level `skills*` vars).
#[derive(Debug, Clone)]
pub struct Opts {
    pub scope: String,
    /// One or more resolved harness ids (`--harness`, repeatable; alias-mapped from the
    /// deprecated `--surface`). Epic 1 resolves the first; the multi-target install loop +
    /// receipt upsert lands in Epic 2 (Issue 2.2/2.3).
    pub harnesses: Vec<String>,
    pub target: String,
    /// True when the operator explicitly passed `--harness`/`--surface`/`--target`. A
    /// **false** value on `upgrade` is the *unqualified* case that enumerates the install
    /// receipt instead of the default harness (Issue 2.3).
    pub explicit: bool,
    pub dry_run: bool,
    pub quiet: bool,
    /// Emit the universal `--json` envelope (SPEC-JSON-006) instead of the human lines.
    pub json: bool,
}

/// One `install`/`upgrade`/`remove` outcome row for the `--json` envelope (SPEC-JSON-006).
#[derive(Debug, Clone, serde::Serialize)]
struct SkillActionItem {
    name: String,
    path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    files: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    removed: Option<bool>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pruned: Vec<String>,
}

/// One whole-skill garbage-collection outcome (plan-012). A skill is a GC candidate when it was
/// recorded as deployed to a target but the current binary no longer ships it
/// (`embed::skill_names()`). Because a GC'd skill is absent from the embedded set it can never
/// surface through the [`SkillActionItem`] deploy loop — this collection is how sweeps render.
#[derive(Debug, Clone, serde::Serialize)]
struct GcItem {
    name: String,
    path: String,
    /// `removed` (swept), `skipped_no_marker` (lacked naba's integrity marker — vetoed), or
    /// `would-remove` (under `--dry-run`).
    outcome: &'static str,
}

/// The `install`/`upgrade`/`remove` `--json` payload (SPEC-JSON-006) for a **single** target —
/// the pinned flat shape (`--target` or a single `--harness`).
#[derive(Debug, Clone, serde::Serialize)]
struct SkillsActionReport {
    action: &'static str,
    destination: String,
    dry_run: bool,
    skills: Vec<SkillActionItem>,
    /// Whole-skill GC outcomes for this target (plan-012); omitted when empty.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    gc: Vec<GcItem>,
}

/// One target's outcome inside a multi-harness action report (plan-008 Issue 2.2/2.3). Under
/// continue-on-error enumeration a failed target carries an `error` instead of `skills`.
#[derive(Debug, Clone, serde::Serialize)]
struct SkillsTargetReport {
    harness: String,
    destination: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    skills: Vec<SkillActionItem>,
    /// Whole-skill GC outcomes for this target (plan-012); omitted when empty.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    gc: Vec<GcItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

/// The multi-target `--json` payload emitted when several harnesses are installed in one
/// invocation (plan-008 Issue 2.2). The SPEC fold for this shape is Issue 5.5.
#[derive(Debug, Clone, serde::Serialize)]
struct SkillsMultiActionReport {
    action: &'static str,
    dry_run: bool,
    targets: Vec<SkillsTargetReport>,
}

/// One `status` row for the `--json` envelope (SPEC-JSON-006).
#[derive(Debug, Clone, serde::Serialize)]
struct SkillStatusItem {
    name: String,
    path: String,
    installed: bool,
    up_to_date: bool,
    complete: bool,
    unmodified: bool,
}

/// The `status` `--json` payload (SPEC-JSON-006).
#[derive(Debug, Clone, serde::Serialize)]
struct SkillsStatusReport {
    destination: String,
    skills: Vec<SkillStatusItem>,
}

/// `resolve_dest` is harness-aware (plan-008, Issue 1.3): an explicit `target` wins; otherwise
/// the anchor is `$HOME` (user scope) or the git root / cwd (project scope), joined with the
/// harness's scope-appropriate subpath from the [`crate::harness`] descriptor table
/// (SPEC-SKILLS harness-layout). A canonical harness (`claude-code`, `opencode`, `pi`, `codex`,
/// `agents`) uses its idiomatic subpath; an unknown/legacy id falls back to the uniform
/// `.<id>/skills` layout so deprecated `--surface` values still resolve to their historical
/// directory. Shared by `naba skills`, `naba doctor`, and `naba skills preflight`.
pub fn resolve_dest(scope: &str, harness: &str, target: &str) -> AppResult<PathBuf> {
    if !target.is_empty() {
        return Ok(PathBuf::from(target));
    }
    let anchor = if scope == "project" {
        git_root_or_cwd()
    } else {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .filter(|p| !p.as_os_str().is_empty())
            .ok_or_else(|| AppError::general("$HOME is not defined"))?
    };
    Ok(anchor.join(crate::harness::resolve_subpath(scope, harness)))
}

/// The harness a single-dest `skills`/`doctor`/`preflight` operation resolves against. Epic 1
/// uses the first requested harness (default `claude-code`); the multi-harness install loop is
/// Issue 2.2.
fn primary_harness(harnesses: &[String]) -> &str {
    harnesses
        .first()
        .map(String::as_str)
        .unwrap_or(crate::harness::DEFAULT_HARNESS)
}

/// Git repository root (`git rev-parse --show-toplevel`), falling back to the current
/// working directory when not in a repo (matches Go's `gitRootOrCwd`).
fn git_root_or_cwd() -> PathBuf {
    if let Ok(out) = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
    {
        if out.status.success() {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !s.is_empty() {
                return PathBuf::from(s);
            }
        }
    }
    std::env::current_dir().unwrap_or_default()
}

/// A resolved deployment target: one on-disk `dest`, the registry `rows` that dedup to it, and
/// the `prev_skills` recorded there before this run (plan-012 GC basis).
///
/// Two registry rows can share a resolved path (codex + portable `agents` both → `.agents/skills`).
/// GC must **read** the union of their recorded skill sets (so a skill recorded under either row
/// is a candidate) and, after a sweep, **write** the new set back to *every* co-resolving row —
/// else a row left behind keeps a phantom name and re-reports it on every future upgrade. `rows`
/// carries the whole `(harness, scope)` fan-out; `harness`/`scope` are the representative first
/// row used for reporting.
#[derive(Debug, Clone)]
struct ResolvedTarget {
    harness: String,
    scope: String,
    dest: PathBuf,
    /// Every `(harness, scope)` registry row that resolves to `dest` — the record fan-out set.
    rows: Vec<(String, String)>,
    /// Union of the skill names recorded at `dest` across `rows` before this run.
    prev_skills: Vec<String>,
}

/// The targets a `skills` invocation operates on, deduped by resolved absolute path (plan-008
/// Issue 2.2/2.3). A `--target` override collapses to a single target under the primary harness;
/// otherwise every requested `--harness` resolves to its idiomatic dest, and paths shared by two
/// harnesses (codex + portable `agents` both → `.agents/skills`) are deployed **once**. These
/// flag-resolved targets carry no `prev_skills` — GC only runs on the receipt-driven upgrade path.
fn install_targets(opts: &Opts) -> AppResult<Vec<ResolvedTarget>> {
    let mut pairs: Vec<(String, PathBuf)> = Vec::new();
    if !opts.target.is_empty() {
        let h = primary_harness(&opts.harnesses).to_string();
        pairs.push((h, PathBuf::from(&opts.target)));
    } else {
        let mut seen = std::collections::BTreeSet::new();
        for h in &opts.harnesses {
            let dest = resolve_dest(&opts.scope, h, "")?;
            if seen.insert(dest.clone()) {
                pairs.push((h.clone(), dest));
            }
        }
    }
    Ok(pairs
        .into_iter()
        .map(|(harness, dest)| ResolvedTarget {
            rows: vec![(harness.clone(), opts.scope.clone())],
            scope: opts.scope.clone(),
            harness,
            dest,
            prev_skills: Vec::new(),
        })
        .collect())
}

/// Record (or, on remove, drop) a target in the skills-install registry (Issue 2.2; plan-012
/// records the deployed skill set for GC). Best-effort: a registry read/write failure is reported
/// to stderr but never fails the deploy that already succeeded — the on-disk skills are the source
/// of truth, the registry is a convenience index.
///
/// The upsert (and, on remove, the drop) fans out over **every** `(harness, scope)` row in
/// `rt.rows` — the write half of the read-union GC contract — so co-resolving rows never diverge.
/// `skills` is the set to record (embedded ∪ any GC name not yet swept); see [`recorded_skill_set`].
fn record_target(mode: Mode, rt: &ResolvedTarget, skills: &[String]) {
    let path = rt.dest.display().to_string();
    let result = (|| -> AppResult<()> {
        let mut reg = crate::skills_install::Registry::load()?;
        let mut changed = false;
        for (harness, scope) in &rt.rows {
            let c = match mode {
                // Install and upgrade both assert "this target now carries naba's skills".
                Mode::Install | Mode::Upgrade => reg.upsert(
                    crate::skills_install::Target::new(harness, scope, &path)
                        .with_skills(skills.to_vec()),
                ),
                Mode::Remove => reg.remove(harness, scope, &path),
            };
            changed |= c;
        }
        // Install/upgrade may only mutate an existing row's skill list (upsert → false), so save
        // whenever we asserted ownership; on remove, save only when a row was actually dropped.
        let should_save = matches!(mode, Mode::Install | Mode::Upgrade) || changed;
        if should_save {
            reg.save()?;
        }
        Ok(())
    })();
    if let Err(e) = result {
        eprintln!("warning: could not update skills-install registry: {e}");
    }
}

/// Resolve the targets a run operates on. An **unqualified `upgrade`** (no `--harness`/
/// `--surface`/`--target`) enumerates the install receipt — every previously-installed target,
/// across every harness — migrating an empty registry from a legacy disk scan first (Issue 2.3,
/// 2.4). Enumerated targets are deduped by resolved absolute path (codex + `agents` share
/// `.agents/skills`); rows that dedup to one path have their `(harness, scope)` rows collected and
/// their recorded skill sets **unioned** into `prev_skills` (plan-012 GC read half). Every other
/// invocation uses the flag-resolved [`install_targets`].
fn targets_for(mode: Mode, opts: &Opts) -> AppResult<Vec<ResolvedTarget>> {
    if mode == Mode::Upgrade && !opts.explicit {
        let reg = crate::skills_install::load_or_migrate()?;
        let mut out: Vec<ResolvedTarget> = Vec::new();
        let mut idx: std::collections::HashMap<PathBuf, usize> = std::collections::HashMap::new();
        for t in reg.targets {
            let p = PathBuf::from(&t.path);
            match idx.get(&p) {
                Some(&i) => {
                    let rt = &mut out[i];
                    rt.rows.push((t.harness.clone(), t.scope.clone()));
                    for s in t.skills {
                        if !rt.prev_skills.contains(&s) {
                            rt.prev_skills.push(s);
                        }
                    }
                }
                None => {
                    idx.insert(p.clone(), out.len());
                    out.push(ResolvedTarget {
                        harness: t.harness.clone(),
                        scope: t.scope.clone(),
                        rows: vec![(t.harness, t.scope)],
                        dest: p,
                        prev_skills: t.skills,
                    });
                }
            }
        }
        return Ok(out);
    }
    install_targets(opts)
}

/// The GC remove-safety gate (plan-012): true when `<skill_dir>/SKILL.md` carries naba's hidden
/// integrity marker ([`embed::MARKER_PREFIX`]), proving naba deployed it. A dir that lost or never
/// had the marker is another tool's (or the operator's) — GC vetoes its removal.
fn has_naba_marker(skill_dir: &Path) -> bool {
    fs::read_to_string(skill_dir.join("SKILL.md"))
        .map(|c| c.contains(embed::MARKER_PREFIX))
        .unwrap_or(false)
}

/// Whole-skill garbage-collection pass over one target (plan-012, step (c)/(d)). The GC set is the
/// recorded `prev_skills` minus the current binary's `embed::skill_names()`; each candidate dir is
/// swept with `remove_dir_all` **only** after the marker-safety gate ([`has_naba_marker`]) passes.
/// A dir lacking the marker is REPORTED (`skipped_no_marker`), never removed — `remove_dir_all` is
/// irreversible, so the marker veto plus the empty-default recorded set are the complete safety
/// story. Under `--dry-run` nothing is deleted; candidates report `would-remove`. An already-absent
/// dir reports `removed` so its phantom name is dropped from the recorded set on the next record.
fn gc_pass(rt: &ResolvedTarget, opts: &Opts) -> AppResult<Vec<GcItem>> {
    let embedded: std::collections::HashSet<String> = embed::skill_names().into_iter().collect();
    let mut out = Vec::new();
    for name in &rt.prev_skills {
        if embedded.contains(name) {
            continue; // still shipped by this binary — not a GC candidate.
        }
        let skill_dir = rt.dest.join(name);
        let path = skill_dir.display().to_string();
        let item = |outcome: &'static str| GcItem {
            name: name.clone(),
            path: path.clone(),
            outcome,
        };
        if !skill_dir.exists() {
            // Already gone — record as removed so the phantom name is dropped.
            out.push(item("removed"));
            continue;
        }
        // Remove-safety gate: a dir that lost/lacks the marker is reported, never removed.
        if !has_naba_marker(&skill_dir) {
            if !opts.json && !opts.quiet {
                println!("  GC skipped (no naba marker): {name} ({path})");
            }
            out.push(item("skipped_no_marker"));
            continue;
        }
        if opts.dry_run {
            if !opts.json && !opts.quiet {
                println!("(dry run) would remove stale skill {name} ({path})");
            }
            out.push(item("would-remove"));
            continue;
        }
        fs::remove_dir_all(&skill_dir).map_err(|e| AppError::file_io(e.to_string()))?;
        if !opts.json && !opts.quiet {
            println!("  GC removed stale skill: {name} ({path})");
        }
        out.push(item("removed"));
    }
    Ok(out)
}

/// The skill-name set to record after an install/upgrade: the current embedded set plus any GC
/// candidate **not** successfully swept (`skipped_no_marker`, `would-remove`, or a failed remove).
/// A name is dropped from the registry only once its dir is gone, so a partial GC failure leaves it
/// recorded and the next upgrade retries rather than silently orphaning the on-disk directory.
fn recorded_skill_set(gc: &[GcItem]) -> Vec<String> {
    let mut set = embed::skill_names();
    for g in gc {
        if g.outcome != "removed" && !set.contains(&g.name) {
            set.push(g.name.clone());
        }
    }
    set.sort();
    set.dedup();
    set
}

/// Deploy/remove all embedded skills for a single target, returning the per-skill items or the
/// first error. Extracted so the caller can apply continue-on-error across targets (Issue 2.3).
fn run_one_target(mode: Mode, dest: &Path, opts: &Opts) -> AppResult<Vec<SkillActionItem>> {
    let mut items = Vec::new();
    for name in embed::skill_names() {
        let item = match mode {
            Mode::Remove => remove_skill(&name, dest, opts)?,
            _ => deploy_skill(&name, dest, mode == Mode::Upgrade, opts)?,
        };
        items.push(item);
    }
    Ok(items)
}

/// Run install / upgrade / remove over every embedded skill (Go's `runSkills`), for each
/// resolved (harness, dest) target (Issue 2.2 multi-harness install, Issue 2.3 receipt-driven
/// upgrade). Multi-target runs are **continue-on-error**: a failing target is skipped-and-
/// reported and the command still processes the rest, then exits non-zero. Under `--json`
/// (incl. the piped auto-enable) emits the universal envelope (SPEC-JSON-006): the pinned flat
/// shape for a single target, a `targets` array otherwise.
/// One per-target install/upgrade outcome row:
/// `(harness, dest, skill actions, gc sweeps, error)`.
type PerTargetRow = (
    String,
    PathBuf,
    Vec<SkillActionItem>,
    Vec<GcItem>,
    Option<String>,
);

pub fn run(mode: Mode, opts: &Opts) -> AppResult<()> {
    let targets = targets_for(mode, opts)?;

    // Nothing recorded to upgrade: report cleanly rather than silently no-op.
    if targets.is_empty() {
        if opts.json {
            output::print_ok_json(SkillsMultiActionReport {
                action: mode.action(),
                dry_run: opts.dry_run,
                targets: Vec::new(),
            });
        } else if !opts.quiet {
            println!(
                "No installed skill targets recorded (nothing to {}).",
                mode.action()
            );
        }
        return Ok(());
    }

    // A single flag-resolved target keeps the pinned fail-fast + flat-JSON behavior; several
    // targets (multi-harness install or receipt-driven upgrade) run continue-on-error.
    let continue_on_error = targets.len() > 1;
    let mut per_target: Vec<PerTargetRow> = Vec::new();
    let mut failures = 0usize;
    for rt in &targets {
        // Ordered per plan-012: (a) prev_skills already read in targets_for; (b) deploy the
        // current embedded set; (c)/(d) diff + marker-vetoed sweep; (e) record the new set.
        let outcome = run_one_target(mode, &rt.dest, opts).and_then(|items| {
            let gc = if mode == Mode::Upgrade {
                gc_pass(rt, opts)?
            } else {
                Vec::new()
            };
            Ok((items, gc))
        });
        match outcome {
            Ok((items, gc)) => {
                if !opts.dry_run {
                    record_target(mode, rt, &recorded_skill_set(&gc));
                }
                per_target.push((rt.harness.clone(), rt.dest.clone(), items, gc, None));
            }
            Err(e) if continue_on_error => {
                failures += 1;
                eprintln!(
                    "warning: skipping {} target {} — {e}",
                    rt.harness,
                    rt.dest.display()
                );
                per_target.push((
                    rt.harness.clone(),
                    rt.dest.clone(),
                    Vec::new(),
                    Vec::new(),
                    Some(e.to_string()),
                ));
            }
            Err(e) => return Err(e),
        }
    }

    if opts.json {
        if per_target.len() == 1 && per_target[0].4.is_none() {
            let (_, dest, items, gc, _) = per_target.into_iter().next().unwrap();
            output::print_ok_json(SkillsActionReport {
                action: mode.action(),
                destination: dest.display().to_string(),
                dry_run: opts.dry_run,
                skills: items,
                gc,
            });
        } else {
            let targets = per_target
                .into_iter()
                .map(|(harness, dest, skills, gc, error)| SkillsTargetReport {
                    harness,
                    destination: dest.display().to_string(),
                    skills,
                    gc,
                    error,
                })
                .collect();
            output::print_ok_json(SkillsMultiActionReport {
                action: mode.action(),
                dry_run: opts.dry_run,
                targets,
            });
        }
    } else if !opts.dry_run && !opts.quiet {
        for (_, dest, _, _, error) in &per_target {
            match error {
                None => println!("Destination: {}", dest.display()),
                Some(e) => println!("Skipped: {} — {e}", dest.display()),
            }
        }
    }

    // Continue-on-error still surfaces failure: a non-zero exit if any target failed.
    if failures > 0 {
        return Err(AppError::general(format!(
            "{failures} skill target(s) failed to {}",
            mode.action()
        )));
    }
    Ok(())
}

/// `skills status`: print a one-line status per embedded skill (Go's `skillsStatusCmd`), or the
/// universal envelope under `--json` (SPEC-JSON-006).
pub fn status(opts: &Opts) -> AppResult<()> {
    let dest = resolve_dest(&opts.scope, primary_harness(&opts.harnesses), &opts.target)?;
    let mut items = Vec::new();
    for name in embed::skill_names() {
        let st = embed::skill_status(&name, &dest.join(&name));
        if opts.json {
            items.push(SkillStatusItem {
                name: name.clone(),
                path: dest.join(&name).display().to_string(),
                installed: st.installed,
                up_to_date: st.up_to_date,
                complete: st.complete,
                unmodified: st.unmodified,
            });
        } else {
            println!("{}", status_line(&name, &st, &dest));
        }
    }
    if opts.json {
        output::print_ok_json(SkillsStatusReport {
            destination: dest.display().to_string(),
            skills: items,
        });
    }
    Ok(())
}

/// One-line human status (Go's `SkillStatusResult.Line`). Not installed →
/// `<name>: not installed (<path>)`; else `<name>: <flags> (<path>)`.
fn status_line(name: &str, st: &embed::SkillStatus, dest: &Path) -> String {
    let path = dest.join(name);
    if !st.installed {
        return format!("{name}: not installed ({})", path.display());
    }
    let flags = format!(
        "{} {} {}",
        bool_flag("up-to-date", st.up_to_date),
        bool_flag("complete", st.complete),
        bool_flag("unmodified", st.unmodified),
    );
    format!("{name}: {flags} ({})", path.display())
}

fn bool_flag(label: &str, ok: bool) -> String {
    if ok {
        format!("\u{2713}{label}") // ✓
    } else {
        format!("\u{2717}{label}") // ✗
    }
}

/// Write an embedded skill's tree to `<dest>/<name>/`, injecting a fresh integrity marker
/// into SKILL.md. With `prune=true` (upgrade) it also removes dest files absent from the
/// embed (rsync --delete parity). Ports Go's `deploySkill`. Returns the outcome row for the
/// `--json` envelope; human lines print only when not `--json`.
fn deploy_skill(name: &str, dest: &Path, prune: bool, opts: &Opts) -> AppResult<SkillActionItem> {
    let dest_dir = dest.join(name);
    let rels = embed::skill_files(name);
    let hash = embed::embedded_tree_hash(name);
    let marker = embed::format_marker(version::VERSION, &hash);

    if opts.dry_run {
        if !opts.json {
            println!(
                "(dry run) would write {} file(s) of {:?} -> {}",
                rels.len(),
                name,
                dest_dir.display()
            );
            if prune {
                println!("(dry run) would prune dest files absent from the embed");
            }
        }
        return Ok(SkillActionItem {
            name: name.to_string(),
            path: dest_dir.display().to_string(),
            files: Some(rels.len()),
            removed: None,
            pruned: Vec::new(),
        });
    }

    for rel in &rels {
        let bytes = embed::read_skill_file(name, rel)
            .ok_or_else(|| AppError::file_io(format!("embedded file missing: {name}/{rel}")))?;
        let data = if rel == "SKILL.md" {
            embed::inject_marker_bytes(bytes, &marker)
        } else {
            bytes.to_vec()
        };
        let path = dest_dir.join(rel.replace('/', std::path::MAIN_SEPARATOR_STR));
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| AppError::file_io(e.to_string()))?;
            set_mode(parent, 0o755);
        }
        fs::write(&path, &data).map_err(|e| AppError::file_io(e.to_string()))?;
        set_mode(&path, 0o644);
    }

    let pruned = if prune {
        prune_stale(name, &dest_dir, opts)?
    } else {
        Vec::new()
    };
    if !opts.json && !opts.quiet {
        println!(
            "OK: {name} -> {} ({} files)",
            dest_dir.display(),
            rels.len()
        );
    }
    Ok(SkillActionItem {
        name: name.to_string(),
        path: dest_dir.display().to_string(),
        files: Some(rels.len()),
        removed: None,
        pruned,
    })
}

/// Remove files under `dest_dir` that are not part of the embedded skill tree (Go's
/// `pruneStale`). Returns the skill-relative paths pruned.
fn prune_stale(name: &str, dest_dir: &Path, opts: &Opts) -> AppResult<Vec<String>> {
    let want: std::collections::HashSet<String> = embed::skill_files(name).into_iter().collect();
    let mut on_disk = Vec::new();
    walk_files(dest_dir, dest_dir, &mut on_disk)?;
    let mut pruned = Vec::new();
    for (path, rel) in on_disk {
        if !want.contains(&rel) {
            fs::remove_file(&path).map_err(|e| AppError::file_io(e.to_string()))?;
            if !opts.json && !opts.quiet {
                println!("  pruned stale: {rel}");
            }
            pruned.push(rel);
        }
    }
    Ok(pruned)
}

/// Recursively collect `(absolute path, skill-relative slash path)` of files under `dir`.
fn walk_files(root: &Path, dir: &Path, out: &mut Vec<(PathBuf, String)>) -> AppResult<()> {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(e) => return Err(AppError::file_io(e.to_string())),
    };
    for entry in entries {
        let entry = entry.map_err(|e| AppError::file_io(e.to_string()))?;
        let path = entry.path();
        let ft = entry
            .file_type()
            .map_err(|e| AppError::file_io(e.to_string()))?;
        if ft.is_dir() {
            walk_files(root, &path, out)?;
        } else {
            let rel = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .replace('\\', "/");
            out.push((path, rel));
        }
    }
    Ok(())
}

/// Remove an installed skill directory (Go's `removeSkill`). Absent → `absent: <dir>`;
/// dry-run → `(dry run) would remove <dir>`; else recursive delete + `removed: <dir>`. Returns
/// the outcome row for the `--json` envelope (`removed` = whether the tree was actually deleted).
fn remove_skill(name: &str, dest: &Path, opts: &Opts) -> AppResult<SkillActionItem> {
    let dest_dir = dest.join(name);
    let item = |removed: bool| SkillActionItem {
        name: name.to_string(),
        path: dest_dir.display().to_string(),
        files: None,
        removed: Some(removed),
        pruned: Vec::new(),
    };
    if !dest_dir.exists() {
        if !opts.json && !opts.quiet {
            println!("absent: {}", dest_dir.display());
        }
        return Ok(item(false));
    }
    if opts.dry_run {
        if !opts.json {
            println!("(dry run) would remove {}", dest_dir.display());
        }
        return Ok(item(false));
    }
    fs::remove_dir_all(&dest_dir).map_err(|e| AppError::file_io(e.to_string()))?;
    if !opts.json && !opts.quiet {
        println!("removed: {}", dest_dir.display());
    }
    Ok(item(true))
}

/// Set unix file mode (no-op semantics elsewhere). Mirrors Go's explicit `0o644`/`0o755`.
#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) {
    use std::os::unix::fs::PermissionsExt;
    let _ = fs::set_permissions(path, fs::Permissions::from_mode(mode));
}

#[cfg(not(unix))]
fn set_mode(_path: &Path, _mode: u32) {}

#[cfg(test)]
mod tests {
    use super::*;

    fn opts(target: &Path) -> Opts {
        Opts {
            scope: "user".into(),
            harnesses: vec!["claude-code".into()],
            target: target.to_string_lossy().into_owned(),
            explicit: true,
            dry_run: false,
            quiet: false,
            json: false,
        }
    }

    #[test]
    fn resolve_dest_target_wins() {
        let d = resolve_dest("user", "claude-code", "/tmp/explicit").unwrap();
        assert_eq!(d, PathBuf::from("/tmp/explicit"));
    }

    /// The harness-layout gate test (SKILL.md Capability Gate: Harness path validation, the
    /// CI baseline sourced from Issue 1.3): `resolve_dest` produces the correct idiomatic path
    /// for every harness × scope, matching the descriptor table.
    #[test]
    fn resolve_dest_harness_paths() {
        let prev_home = std::env::var_os("HOME");
        std::env::set_var("HOME", "/home/tester");

        // User scope: anchored at $HOME, per-harness idiomatic subpath.
        let user_cases = [
            ("claude-code", "/home/tester/.claude/skills"),
            ("opencode", "/home/tester/.config/opencode/skills"),
            ("pi", "/home/tester/.pi/agent/skills"),
            ("codex", "/home/tester/.agents/skills"),
            ("agents", "/home/tester/.agents/skills"),
            // legacy --surface value maps through the alias before resolution
            ("claude", "/home/tester/.claude/skills"),
        ];
        for (harness, expect) in user_cases {
            let h = crate::harness::surface_alias(harness);
            assert_eq!(
                resolve_dest("user", &h, "").unwrap(),
                PathBuf::from(expect),
                "user harness {harness}"
            );
        }

        std::env::set_var("HOME", "/home/tester2");
        // Project scope is anchored at the git root/cwd; assert the trailing subpath per harness.
        let proj_cases = [
            ("claude-code", ".claude/skills"),
            ("opencode", ".opencode/skills"),
            ("pi", ".pi/skills"),
            ("codex", ".agents/skills"),
            ("agents", ".agents/skills"),
        ];
        for (harness, sub) in proj_cases {
            let d = resolve_dest("project", harness, "").unwrap();
            assert!(
                d.ends_with(sub),
                "project harness {harness}: {} should end with {sub}",
                d.display()
            );
        }

        match prev_home {
            Some(v) => std::env::set_var("HOME", v),
            None => std::env::remove_var("HOME"),
        }
    }

    #[test]
    fn install_targets_dedupe_shared_path() {
        // codex and the portable `agents` harness both resolve to `.agents/skills`; the shared
        // path must be deployed once (Issue 2.2/2.3 dedupe). Order-preserving: first wins. Uses
        // project scope (git-root/cwd anchor) to avoid racing the shared $HOME with other tests.
        let o = Opts {
            scope: "project".into(),
            harnesses: vec!["codex".into(), "agents".into(), "claude-code".into()],
            target: String::new(),
            explicit: true,
            dry_run: true,
            quiet: true,
            json: false,
        };
        let targets = install_targets(&o).unwrap();
        let ids: Vec<&str> = targets.iter().map(|rt| rt.harness.as_str()).collect();
        assert_eq!(
            ids,
            ["codex", "claude-code"],
            "agents deduped against codex path"
        );
    }

    #[test]
    fn install_then_status_round_trip() {
        let dir = std::env::temp_dir().join(format!("naba-skills-test-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let o = opts(&dir);

        run(Mode::Install, &o).unwrap();

        // SKILL.md exists and carries an injected marker.
        let skill_md = dir.join("naba").join("SKILL.md");
        assert!(skill_md.is_file());
        let content = fs::read_to_string(&skill_md).unwrap();
        assert!(content.contains(embed::MARKER_PREFIX));

        // status: fresh install is up-to-date/complete/unmodified.
        let st = embed::skill_status("naba", &dir.join("naba"));
        assert!(st.installed && st.up_to_date && st.complete && st.unmodified);
        let line = status_line("naba", &st, &dir);
        assert!(line.contains("\u{2713}up-to-date"));
        assert!(line.contains("\u{2713}unmodified"));

        // remove clears the tree.
        run(Mode::Remove, &o).unwrap();
        assert!(!dir.join("naba").exists());

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn upgrade_prunes_stale_files() {
        let dir = std::env::temp_dir().join(format!("naba-skills-prune-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        let o = opts(&dir);

        run(Mode::Install, &o).unwrap();
        // Plant a stale file not present in the embed.
        let stale = dir.join("naba").join("stale.txt");
        fs::write(&stale, b"junk").unwrap();
        assert!(stale.is_file());

        run(Mode::Upgrade, &o).unwrap();
        assert!(!stale.exists(), "upgrade should prune stale files");

        let _ = fs::remove_dir_all(&dir);
    }

    /// Whole-skill GC (plan-012 Issue 3.5), paralleling `upgrade_prunes_stale_files`: a skill "A"
    /// a newer binary no longer ships is swept from disk, while a co-located NON-naba skill "B"
    /// (no integrity marker) is left untouched by the marker-safety veto. Drives [`gc_pass`]
    /// directly with a synthetic `prev_skills` set so it needs no global skills-install registry
    /// state (which lives at a shared config path); [`recorded_skill_set`] verifies the (e)
    /// drop-only-after-removal rule.
    #[test]
    fn upgrade_gc_sweeps_dropped_skill_but_spares_unmarked() {
        let dir = std::env::temp_dir().join(format!("naba-skills-gc-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);

        // "A": a former naba skill (carries the integrity marker) the current binary no longer
        // ships. `naba` (still embedded) plays the "kept" skill.
        let a = dir.join("A");
        fs::create_dir_all(&a).unwrap();
        let marker = embed::format_marker("9.9.9", "deadbeefcafe");
        fs::write(
            a.join("SKILL.md"),
            embed::inject_marker("---\nname: A\n---\n", &marker),
        )
        .unwrap();

        // "B": co-located, NOT naba's (no marker) — the veto must spare it.
        let b = dir.join("B");
        fs::create_dir_all(&b).unwrap();
        fs::write(b.join("SKILL.md"), "---\nname: B\n---\n").unwrap();

        let o = opts(&dir);
        let rt = ResolvedTarget {
            harness: "claude-code".into(),
            scope: "user".into(),
            dest: dir.clone(),
            rows: vec![("claude-code".into(), "user".into())],
            // "naba" is still embedded ⇒ never a GC candidate; "A"/"B" are not embedded.
            prev_skills: vec!["A".into(), "B".into(), "naba".into()],
        };

        let gc = gc_pass(&rt, &o).unwrap();

        // (d) A swept (marker present); B spared (no marker); naba never a candidate.
        assert!(
            !a.exists(),
            "GC should remove the dropped, naba-marked skill A"
        );
        assert!(
            b.join("SKILL.md").is_file(),
            "marker veto must spare the unmarked co-located skill B"
        );
        let by_name = |n: &str| gc.iter().find(|g| g.name == n);
        assert_eq!(by_name("A").unwrap().outcome, "removed");
        assert_eq!(by_name("B").unwrap().outcome, "skipped_no_marker");
        assert!(
            by_name("naba").is_none(),
            "a still-embedded skill is not a GC candidate"
        );

        // (e) drop-only-after-removal: A gone from the recorded set, B retained for retry.
        let recorded = recorded_skill_set(&gc);
        assert!(
            !recorded.contains(&"A".to_string()),
            "swept skill dropped from the recorded set"
        );
        assert!(
            recorded.contains(&"B".to_string()),
            "vetoed skill stays recorded so the next upgrade retries"
        );
        assert!(recorded.contains(&"naba".to_string()));

        let _ = fs::remove_dir_all(&dir);
    }
}
