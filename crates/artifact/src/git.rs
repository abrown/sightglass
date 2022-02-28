use anyhow::{bail, Result};
use std::process::Command;

/// Use the local system's Git installation to resolve a revision in a repository into a commit SHA.
///
/// ```
/// # use sightglass_artifact::resolve_to_commit;
/// let repo = "https://github.com/bytecodealliance/wasmtime";
/// let revision = "v0.33.1"; // A tag (but a branch or commit would work as well).
/// let commit = resolve_to_commit(repo, revision).unwrap();
/// assert_eq!(&commit[0..7], "5215c78");
/// ```
pub fn resolve_to_commit(repository: &str, revision: &str) -> Result<String> {
    let output = Command::new("git")
        .args(["ls-remote", repository, revision])
        .output()?;
    if output.status.success() {
        let out = std::str::from_utf8(&output.stdout)?;
        println!("{}", out);
        let (commit, _) = out
            .split_once("\t")
            .expect("a string like: <hash> \\t refs/...");
        Ok(commit.trim().to_string())
    } else {
        bail!("unable to run 'git ls-remote {} {}'", repository, revision);
    }
}
