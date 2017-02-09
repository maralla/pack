use std::path::Path;

use Result;
use git2::{self, Repository};

const LOCATION: &'static str = "https://github.com";

fn fetch(repo: &Repository, name: &str) -> Result<()> {
    let url = format!("{}/{}", LOCATION, name);

    let mut opts = git2::FetchOptions::new();
    opts.download_tags(git2::AutotagOption::All);

    let refspec = "refs/heads/*:refs/heads/*";
    let mut remote = repo.remote_anonymous(&url)?;
    remote.fetch(&[refspec], Some(&mut opts), None)?;
    Ok(())
}

pub fn clone<P: AsRef<Path>>(name: &str, target: P) -> Result<()> {
    let repo = git2::Repository::init(&target)?;
    fetch(&repo, name)?;
    let reference = "HEAD";
    let oid = repo.refname_to_id(reference)?;
    let object = repo.find_object(oid, None)?;
    repo.reset(&object, git2::ResetType::Hard, None)?;
    Ok(())
}

pub fn update<P: AsRef<Path>>(name: &str, path: P) -> Result<()> {
    let repo = Repository::open(&path)?;
    fetch(&repo, name)
}
