use anyhow::Result;
use git2::Repository;

pub fn open_repo() -> Result<Repository> {
    let repo = Repository::discover(".")?;
    Ok(repo)
}
