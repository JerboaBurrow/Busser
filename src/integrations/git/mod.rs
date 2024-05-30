use core::fmt;
use std::path::Path;

use git2::{Cred, RemoteCallbacks, Repository};

use crate::config::GitConfig;

pub mod refresh;

#[derive(Debug, Clone)]
pub struct GitError
{
    pub why: String
}

impl fmt::Display for GitError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.why)
    }
}

impl From<git2::Error> for GitError
{
    fn from(value: git2::Error) -> Self
    {
        GitError
        {
            why: format!("git2::Error {}", value)
        }
    }
}

/// Attempt to clone a remote repo from a [crate::config::GitConfig]
pub fn from_clone(path: &str, config: &GitConfig) -> Result<Repository, GitError>
{
    if let GitConfig{auth: Some(_), remote: _, checkout_schedule: _, branch: _} = config
    {
        let auth = config.auth.clone().unwrap();
        let result = match &auth.key_path
        {
            Some(_) => 
            {
                let mut callbacks = RemoteCallbacks::new();
                callbacks.credentials(|_url, _username_from_url, _allowed_types|
                {
                    Cred::ssh_key(
                        &auth.user,
                        None,
                        Path::new(&auth.key_path.clone().unwrap()),
                        Some(&auth.passphrase),
                    )
                });
                callbacks
            },
            None =>
            {
                let mut callbacks = RemoteCallbacks::new();
                callbacks.credentials(|_url, _username_from_url, _allowed_types|
                {
                    Cred::userpass_plaintext(
                        &auth.user,
                        &auth.passphrase,
                    )
                });
                callbacks
            }
        };

        let mut fo = git2::FetchOptions::new();
        fo.remote_callbacks(result);
        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(fo);
        builder.branch(&config.branch);
        match builder.clone(&config.remote,Path::new(&path))
        {
            Ok(repo) => Ok(repo),
            Err(e) =>
            {
                crate::debug(format!("Error {} while cloning (authenticated) repo at {}", e, config.remote), None);
                Err(GitError::from(e))
            }
        }
    }
    else
    {
        match Repository::clone(&config.remote, path)
        {
            Ok(repo) => Ok(repo),
            Err(e) => 
            {
                crate::debug(format!("Error {} while cloning (pub) repo at {}", e, config.remote), None);
                Err(GitError::from(e))
            }
        }
    }
}

/// Make a fresh clone if [crate::config::Config::git] is present
///  deleting any file/dir called [crate::config::ContentConfig::path]
pub fn clean_and_clone(dir: &str, config: GitConfig) -> Result<Repository, GitError>
{
    let path = Path::new(dir);
    let result = if path.is_file()
    {
        std::fs::remove_file(path)
    }
    else if path.is_dir()
    {
        std::fs::remove_dir_all(path)
    }
    else
    {
        Ok(())
    };
    match result
    {
        Ok(_) => (),
        Err(e) =>
        {
            return Err(GitError{why: format!("Could not clone, could not remove, {}", e)})
        }
    }
    match from_clone(dir, &config)
    {
        Ok(repo) =>
        {
            Ok(repo)
        },
        Err(e) =>
        {
            Err(GitError{why: format!("Could not clone, {}", e)})
        }
    }
}

/// Fast forward pull from the repository, makes no attempt to resolve
///  if a fast foward is not possible
pub fn fast_forward_pull(repo: Repository, branch: &str) -> Result<(), GitError>
{
    // modified from https://stackoverflow.com/questions/58768910/how-to-perform-git-pull-with-the-rust-git2-crate
    repo.find_remote("origin")?.fetch(&[branch], None, None)?;

    let fetch_head = repo.find_reference("FETCH_HEAD")?;
    let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;
    let (analysis, _pref) = repo.merge_analysis(&[&fetch_commit])?;

    if analysis.is_up_to_date()
    {
        Ok(())
    }
    else if analysis.is_fast_forward()
    {
        let refname = format!("refs/heads/{}", branch);
        let mut reference = repo.find_reference(&refname)?;
        reference.set_target(fetch_commit.id(), "Fast-Forward")?;
        repo.set_head(&refname)?;
        Ok(repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?)
    }
    else
    {
        Err(GitError{why: "Cannot fastforward".to_owned()})
    }
}