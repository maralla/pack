use crate::Result;

use git2::{self, Repository};
use std::fs;
use std::path::Path;
use std::thread;
use std::time::Duration;

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

    // The quit check: the reclone is the long network operation, and it is
    // the one thing that can actually be cancelled — by never starting
    // it. The in-flight ls-remote above already ran (seconds); nothing
    // after this line begins once quitting is set.
    if crate::task::quitting() {
        return Err(crate::Error::Aborted);
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

    // The sweeper runs FIRST, not after the swap: an abandoned transfer
    // never reaches the swap, so a sweeper that only runs there leaves the
    // partial clone on disk forever (one directory per aborted run, each
    // under a fresh pid). Sweeping at the top makes cleanup convergent
    // across runs: this run's sweeper clears every earlier run's garbage
    // (previous partials and previous swap-outs), excludes this run's
    // in-flight directory, and an abandoned transfer is cleared by the
    // next run that gets here.
    {
        let parent = parent.to_path_buf();
        let name = name.to_string();
        let keep = tmp.clone();
        thread::spawn(move || {
            let old_prefix = format!(".{}.old.", name);
            let new_prefix = format!(".{}.new.", name);
            if let Ok(entries) = fs::read_dir(&parent) {
                for entry in entries.flatten() {
                    let file_name = entry.file_name();
                    let file_name = match file_name.to_str() {
                        Some(n) => n.to_string(),
                        None => continue,
                    };
                    if (file_name.starts_with(&old_prefix)
                        || file_name.starts_with(&new_prefix))
                        && entry.path() != keep
                    {
                        let _ = fs::remove_dir_all(entry.path());
                    }
                }
            }
        });
    }

    // The long part (the network transfer) runs in a thread this caller
    // polls instead of blocking on: libgit2 has no cancellation, so
    // abandonment is the only one — on quit the caller walks away, the
    // handle is dropped, the thread stays detached, and the process exit
    // kills the in-flight transfer. The partial stays under this run's
    // .new.<pid> name; the next run's sweeper clears it.
    let clone_url = url.to_string();
    let clone_tmp = tmp.clone();
    let handle = thread::spawn(move || -> Result<()> {
        let repo = Repository::clone(&clone_url, &clone_tmp)?;
        update_submodules(&repo)
    });
    loop {
        if handle.is_finished() {
            break;
        }
        if crate::task::quitting() {
            // The quit sequence, in order:
            // 1. Cancel the current work: abandon the transfer. The handle
            //    is dropped and the thread stays detached; the process exit
            //    at the end of the quit sequence kills it.
            drop(handle);
            // 2. Clear the unfinished things: remove this run's partial
            //    clone now, blocking — the quit does not happen before
            //    the cleanup. Whatever the abandoned thread still manages
            //    to write is residual, cleared by the top-of-reclone
            //    sweeper of the next run that gets here.
            let _ = fs::remove_dir_all(&tmp);
            // 3. Then quit.
            return Err(crate::Error::Aborted);
        }
        thread::sleep(Duration::from_millis(50));
    }
    handle
        .join()
        .map_err(|_| crate::Error::Git("the clone thread panicked".into()))??;

    fs::rename(path, &old)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use git2::{IndexAddOption, Signature};

    /// Create a real commit in `repo` (a real git repository in a temp
    /// directory): one file change per commit, `msg` as the subject.
    fn commit(repo: &Repository, msg: &str) -> git2::Oid {
        let workdir = repo.workdir().unwrap();
        std::fs::write(workdir.join("f.txt"), msg).unwrap();
        let mut index = repo.index().unwrap();
        index
            .add_all(["*"].iter(), IndexAddOption::DEFAULT, None)
            .unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let sig = Signature::now("t", "t@example.com").unwrap();
        let parent = repo.head().ok().and_then(|h| h.peel_to_commit().ok());
        let parents: Vec<&git2::Commit> = match parent {
            Some(ref c) => vec![c],
            None => vec![],
        };
        repo.commit(Some("HEAD"), &sig, &sig, msg, &tree, &parents)
            .unwrap()
    }

    /// Issue #38: the changelog walker must summarize exactly the
    /// old..new range (exclusive of old), newest first, and nothing else.
    #[test]
    fn changelog_walks_the_update_range() {
        let dir = std::env::temp_dir().join(format!(
            "pack-changelog-test-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let repo = Repository::init(&dir).unwrap();

        let old = commit(&repo, "the old tip");
        let _mid = commit(&repo, "the middle commit");
        let new = commit(&repo, "the new tip");

        let log = changelog(&repo, old, new);
        assert_eq!(log.len(), 2, "exactly the old..new range: {log:?}");
        // Newest first (the update's own commits), never the old tip.
        assert!(log[0].ends_with("the new tip"), "{log:?}");
        assert!(log[1].ends_with("the middle commit"), "{log:?}");
        assert!(!log.iter().any(|e| e.contains("the old tip")), "{log:?}");
        // short-oid prefix + subject, the generated line format.
        assert_eq!(log[0].split(' ').next().unwrap().len(), 7);

        // A diverged range (old not an ancestor of new) gets no overview.
        let dir2 = std::env::temp_dir().join(format!(
            "pack-changelog-test2-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir2);
        let repo2 = Repository::init(&dir2).unwrap();
        let unrelated = commit(&repo2, "unrelated tip");
        assert!(changelog(&repo, unrelated, new).is_empty());

        let _ = std::fs::remove_dir_all(&dir);
        let _ = std::fs::remove_dir_all(&dir2);
    }
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
