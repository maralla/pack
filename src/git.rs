use crate::Result;

use git2::{self, Repository};
use std::fs;
use std::path::Path;
use std::thread;

const LOCATION: &str = "https://github.com";

pub enum UpdateStatus {
    /// The local checkout already matches the upstream tip: nothing to do.
    UpToDate,
    /// The repository was replaced with a fresh clone of the new tip,
    /// carrying a summary of the commits in between (issue #38).
    Updated(Vec<String>),
}

fn github_url(name: &str) -> String {
    format!("{}/{}", LOCATION, name)
}

pub fn clone<P: AsRef<Path>>(name: &str, target: P) -> Result<()> {
    let url = github_url(name);
    let result = git2::Repository::clone_recurse(&url, &target);
    if result.is_err() {
        fs::remove_dir_all(&target)?;
    }
    Ok(())
}

/// Update a plugin checkout to the upstream tip.
///
/// The checkout is compared against the remote tip first (no fetch): a
/// checkout already at the tip is reported as `UpToDate` so the caller can
/// ignore it. A checkout behind the tip is replaced by a fresh clone:
/// an in-place fetch cannot repair a damaged object database (a layer that
/// lost pack files makes every fetch fail on missing base objects), and a
/// fresh clone is the only update that actually works.
pub fn update<P: AsRef<Path>>(name: &str, path: P) -> Result<UpdateStatus> {
    let path = path.as_ref();
    let url = github_url(name);
    let repo = Repository::open(path)?;
    let local = repo.refname_to_id("HEAD")?;

    // Resolve the current upstream tip without fetching anything.
    let remote_head = {
        let mut remote = repo.remote_anonymous(&url)?;
        let connection = remote.connect_auth(git2::Direction::Fetch, None, None)?;
        let heads = connection.list()?;
        heads
            .iter()
            .find(|h| h.name() == "HEAD")
            .map(|h| h.oid())
            .ok_or_else(|| crate::Error::Git("Fail to resolve remote HEAD".into()))?
    };

    if local == remote_head {
        return Ok(UpdateStatus::UpToDate);
    }

    reclone(&url, path)?;
    // Issue #38: overview of what the update brought in. Only a
    // fast-forward range has a meaningful old..new summary; a diverged
    // `local` is unrelated history and gets no overview.
    let repo = Repository::open(path)?;
    let changelog = changelog(&repo, local, remote_head);
    Ok(UpdateStatus::Updated(changelog))
}

/// One-line summary (`short-oid subject`) of each commit in `old..new`
/// (exclusive of `old`), capped. Empty when the range is not walkable.
fn changelog(repo: &Repository, old: git2::Oid, new: git2::Oid) -> Vec<String> {
    const CAP: usize = 20;
    let mut out = Vec::new();
    if !repo.graph_descendant_of(new, old).unwrap_or(false) {
        return out;
    }
    let mut stack = match repo.find_commit(new) {
        Ok(c) => vec![c],
        Err(_) => return out,
    };
    let mut seen = std::collections::HashSet::new();
    while let Some(commit) = stack.pop() {
        let id = commit.id();
        if id == old || !seen.insert(id) {
            continue;
        }
        if out.len() >= CAP {
            break;
        }
        let hex = id.to_string();
        out.push(format!(
            "{} {}",
            &hex[..hex.len().min(7)],
            commit.summary().unwrap_or("")
        ));
        for parent in commit.parents() {
            stack.push(parent);
        }
    }
    out
}

/// Replace `path` with a fresh clone of `url`.
///
/// The broken checkout is swapped out with directory renames (single
/// directory-entry operations) and the fresh clone is swapped in. The
/// leftover is removed on a detached thread: deleting a plugin tree is
/// slow on overlay filesystems, and update must not wait for it. Leftovers
/// from a previous run that did not finish cleaning up are swept the same
/// way, so trash converges to zero across runs.
fn reclone(url: &str, path: &Path) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| crate::Error::Git("Fail to resolve plugin parent directory".into()))?;
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| crate::Error::Git("Fail to resolve plugin directory name".into()))?;
    let pid = std::process::id();
    let tmp = parent.join(format!(".{}.new.{}", name, pid));
    let old = parent.join(format!(".{}.old.{}", name, pid));

    let repo = Repository::clone(url, &tmp)?;
    update_submodules(&repo)?;

    fs::rename(path, &old)?;
    fs::rename(&tmp, path)?;

    // Owned copies for the detached sweeper thread.
    let parent = parent.to_path_buf();
    let name = name.to_string();
    thread::spawn(move || {
        // This run's swap-out plus leftovers from earlier runs that did
        // not finish cleaning up: old.* and new.* are trash by definition
        // once the swap has completed.
        let _ = fs::remove_dir_all(&old);
        let old_prefix = format!(".{}.old.", name);
        let new_prefix = format!(".{}.new.", name);
        if let Ok(entries) = fs::read_dir(&parent) {
            for entry in entries.flatten() {
                let file_name = entry.file_name();
                let file_name = match file_name.to_str() {
                    Some(n) => n.to_string(),
                    None => continue,
                };
                if file_name.starts_with(&old_prefix) || file_name.starts_with(&new_prefix) {
                    let _ = fs::remove_dir_all(entry.path());
                }
            }
        }
    });
    Ok(())
}

fn update_submodules(repo: &Repository) -> Result<()> {
    fn add_subrepos(repo: &Repository, list: &mut Vec<Repository>) -> Result<()> {
        for mut subm in repo.submodules()? {
            if let Some("docs") = subm.name() {
                continue;
            }
            subm.update(true, None)?;
            list.push(subm.open()?);
        }
        Ok(())
    }

    let mut repos = Vec::new();
    add_subrepos(repo, &mut repos)?;
    while let Some(r) = repos.pop() {
        add_subrepos(&r, &mut repos)?;
    }
    Ok(())
}
