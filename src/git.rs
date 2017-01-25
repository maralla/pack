use std::path::Path;

use Result;
use git2::Repository;

const LOCATION: &'static str = "https://github.com/";

pub fn clone<P: AsRef<Path>>(user: &str, repo: &str, target: P) -> Result<()> {
    let url = format!("{}{}/{}", LOCATION, user, repo);
    let _ = Repository::clone(&url, target.as_ref())?;
    Ok(())
}
