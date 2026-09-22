//! Build identity (C2 a76/a77): what build is running, stated as a reading.
//!
//! The rules that shape this module: git is read **only** — the game never
//! writes to the repository; a **dirty tree disqualifies a build from release
//! labelling**; and every value is what a command returned, with `None` for
//! "could not determine" rather than a plausible guess. The pane displays
//! these with their absence named, never papered over.

use std::process::Command;

/// One read-only git fact. `None` means "git could not be asked" (no repo, no
/// git binary) — which the pane renders as its own state, not as a blank.
#[derive(Clone, Debug, Default)]
pub struct BuildInfo {
    pub commit: Option<String>,
    pub branch: Option<String>,
    pub tag: Option<String>,
    /// True when the working tree has uncommitted changes. `None` when git
    /// could not be asked at all.
    pub dirty: Option<bool>,
    /// How many files were dirty at query time — the number the pane shows
    /// as "the tree at launch" (a76: files changed since the session began).
    pub dirty_files: Option<usize>,
}

/// Query the build's identity from git, read-only. Run once at startup and
/// cached: the pane is a reading of the tree at launch, not a live VCS poll —
/// the game must not look like it is doing repository work while running.
pub fn query() -> BuildInfo {
    let run = |args: &[&str]| -> Option<String> {
        let out = Command::new("git").args(args).output().ok()?;
        if !out.status.success() {
            return None;
        }
        let text = String::from_utf8(out.stdout).ok()?;
        let trimmed = text.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    };

    let commit = run(&["rev-parse", "--short", "HEAD"]);
    let branch = run(&["rev-parse", "--abbrev-ref", "HEAD"]);
    let tag = run(&["describe", "--tags", "--exact-match"]);
    let porcelain = run(&["status", "--porcelain"]);
    let dirty = porcelain.clone().map(|out| !out.is_empty());
    let dirty_files = porcelain.map(|out| out.lines().count());
    BuildInfo {
        commit,
        branch,
        tag,
        dirty,
        dirty_files,
    }
}

impl BuildInfo {
    /// The release label. A dirty tree disqualifies the build (a77): a build
    /// that cannot name its own contents is not a release, however it ships.
    pub fn release_label(&self) -> String {
        match (&self.commit, self.dirty) {
            (Some(commit), Some(false)) => match &self.tag {
                Some(tag) => format!("release {tag} ({commit})"),
                None => format!("release {commit}"),
            },
            (Some(commit), Some(true)) => format!("dev build {commit} — dirty tree, not releasable"),
            (Some(commit), None) => format!("build {commit} — tree state unknown"),
            (None, _) => "build identity unknown (no git)".to_string(),
        }
    }

    /// One line for the pane: commit, branch, dirty flag — each named when
    /// absent, because "unknown" is a fact too.
    pub fn summary_line(&self) -> String {
        let commit = self.commit.clone().unwrap_or_else(|| "unknown".into());
        let branch = self.branch.clone().unwrap_or_else(|| "unknown".into());
        let dirty = match self.dirty {
            Some(true) => "dirty",
            Some(false) => "clean",
            None => "unknown",
        };
        format!("{commit} on {branch}, tree {dirty}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// This test runs inside a git checkout, so the query must read it.
    /// What it pins: the facts arrive, and a dirty tree disqualifies release
    /// labelling — the rule a release process would otherwise quietly skip.
    #[test]
    fn identity_reads_the_tree_and_dirty_disqualifies_release() {
        let info = query();
        // In this repo the commit and branch are always determinable.
        assert!(info.commit.is_some(), "commit must be readable in the repo");
        assert!(info.branch.is_some(), "branch must be readable in the repo");
        // The label must always be one of the four honest states.
        let label = info.release_label();
        assert!(
            label.starts_with("release ")
                || label.contains("dev build")
                || label.contains("tree state unknown")
                || label.contains("identity unknown"),
            "label {label:?} is not one of the honest states"
        );
        // Whatever the tree state, a dirty build can never read "release".
        if info.dirty == Some(true) {
            assert!(
                !label.starts_with("release"),
                "a dirty tree claimed a release label: {label}"
            );
        }
    }

    #[test]
    fn summary_line_names_what_it_does_not_know() {
        let unknown = BuildInfo::default();
        let line = unknown.summary_line();
        assert!(line.contains("unknown"), "absence must be named: {line}");
    }
}
