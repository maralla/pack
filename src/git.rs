use std::path::Path;
use std::fs;

use Result;
use git2::{self, Repository};

const LOCATION: &str = "https://github.com";

fn fetch(repo: &Repository, name: &str) -> Result<()> {
    let url = format!("{}/{}", LOCATION, name);

    let mut opts = git2::FetchOptions::new();
    opts.download_tags(git2::AutotagOption::All);

    let refspec = "refs/heads/*:refs/heads/*";
    let mut remote = repo.remote_anonymous(&url)?;
    remote.fetch(&[refspec], Some(&mut opts), None)?;
    Ok(())
}

fn clone_real<P: AsRef<Path>>(name: &str, target: P) -> Result<()> {
    let repo = git2::Repository::init(&target)?;
    fetch(&repo, name)?;
    let reference = "HEAD";
    let oid = repo.refname_to_id(reference)?;
    let object = repo.find_object(oid, None)?;
    repo.reset(&object, git2::ResetType::Hard, None)?;
    update_submodules(&repo)?;
    Ok(())
}

pub fn clone<P: AsRef<Path>>(name: &str, target: P) -> Result<()> {
    let result = clone_real(name, &target);
    if result.is_err() {
        fs::remove_dir_all(&target)?;
    }
    result
}

pub fn update<P: AsRef<Path>>(name: &str, path: P) -> Result<()> {
    let repo = Repository::open(&path)?;
    fetch(&repo, name)
}
fn update_submodules(repo: &Repository) -> Result<()> {

    fn add_subrepos(repo: &Repository, list: &mut Vec<Repository>)
        -> Result<()> {
            for mut subm in repo.submodules()? {
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
