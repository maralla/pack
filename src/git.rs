use crate::Result;

use git2::{self, Repository};
use std::fs;
use std::path::Path;

const LOCATION: &str = "https://github.com";

fn github_url(name: &str) -> String {
    format!("{}/{}", LOCATION, name)
}

fn fetch(repo: &Repository, name: &str) -> Result<()> {
    let url = github_url(&name);

    let mut opts = git2::FetchOptions::new();
    opts.download_tags(git2::AutotagOption::All)
        .update_fetchhead(true);

    let refspec = "refs/heads/*:refs/heads/*";
    let mut remote = repo.remote_anonymous(&url)?;
    remote.fetch(&[refspec], Some(&mut opts), None)?;
    Ok(())
}

fn sync_repo(repo: &Repository, name: &str) -> Result<()> {
    fetch(&repo, name)?;
    let reference = "HEAD";
    let oid = repo.refname_to_id(reference)?;
    let object = repo.find_object(oid, None)?;
    repo.reset(&object, git2::ResetType::Hard, None)?;
    update_submodules(&repo)?;
    Ok(())
}

pub fn clone<P: AsRef<Path>>(name: &str, target: P) -> Result<()> {
    let url = github_url(name);
    let result = git2::Repository::clone_recurse(&url, &target);
    if result.is_err() {
        fs::remove_dir_all(&target)?;
    }
    Ok(())
}

pub fn update<P: AsRef<Path>>(name: &str, path: P) -> Result<()> {
    let repo = Repository::open(&path)?;
    sync_repo(&repo, name)
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
