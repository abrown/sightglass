use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use std::process::Command;
/// Describe a specific Git commit in a repository; this is important for replicating the creation
/// of artifacts. The fields are optional (for now), since in certain cases they can be assumed:
/// the repository of some engines is well-known (e.g. Wasmtime) and the default branch can be used
/// for the revision.
///
/// TODO eventually this and GitSource should merge together.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct GitLocation {
    /// The URL of the Git repository.
    pub repository: Option<String>,
    /// A revision in the repository. According to [the Git
    /// documentation](https://mirrors.edge.kernel.org/pub/software/scm/git/docs/gitrevisions.html#_specifying_revisions)
    /// this could be, e.g.:
    /// - a branch name
    /// - a commit hash
    /// - a tag
    pub revision: Option<String>,
}

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
