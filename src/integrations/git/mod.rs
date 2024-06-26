use core::fmt;
use std::{cmp::min, path::Path};

use chrono::{DateTime, Utc};
use git2::{ Cred, FetchOptions, Oid, RemoteCallbacks, Repository};

use crate::{config::{GitAuthConfig, GitConfig}, filesystem::{folder::list_sub_dirs, set_dir_readonly}};

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

impl From<std::io::Error> for GitError
{
    fn from(value: std::io::Error) -> Self
    {
        GitError
        {
            why: format!("std::io::Error {}", value)
        }
    }
}

fn build_fetch_option(auth: &GitAuthConfig) -> FetchOptions
{
    let mut fo = git2::FetchOptions::new();
    let callbacks = match &auth.key_path
    {
        Some(_) =>
        {
            let mut callbacks = RemoteCallbacks::new();
            callbacks.credentials(move |_url, username_from_url, _allowed_types|
            {
                crate::debug(format!("Using ssh authentication"), Some("GIT"));
                Cred::ssh_key(
                    username_from_url.unwrap(),
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
            callbacks.credentials(move |_url, _username_from_url, _allowed_types|
            {
                crate::debug(format!("Using pass authentication"), Some("GIT"));
                Cred::userpass_plaintext(
                    &auth.user,
                    &auth.passphrase,
                )
            });
            callbacks
        }
    };
    fo.remote_callbacks(callbacks);
    fo
}

/// Attempt to clone a remote repo from a [crate::config::GitConfig]
pub fn from_clone(path: &str, config: &GitConfig) -> Result<Repository, GitError>
{
    if let GitConfig{auth: Some(_), remote: _, checkout_schedule: _, branch: _, remote_webhook_token: _} = config
    {
        crate::debug(format!("Attempting authenticated clone of {}", config.remote), Some("GIT"));
        let auth = config.auth.clone().unwrap();
        let fo = build_fetch_option(&auth);
        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(fo);
        builder.branch(&config.branch);
        match builder.clone(&config.remote,Path::new(&path))
        {
            Ok(repo) =>
            {
                crate::debug(format!("Cloned {}", config.remote), Some("GIT"));
                Ok(repo)
            },
            Err(e) =>
            {
                crate::debug(format!("Error {} while cloning (authenticated) repo at {}", e, config.remote), Some("GIT"));
                Err(GitError::from(e))
            }
        }
    }
    else
    {
        crate::debug(format!("Attempting un-authenticated clone of {}", config.remote), Some("GIT"));
        match Repository::clone(&config.remote, path)
        {
            Ok(repo) =>
            {
                crate::debug(format!("Cloned {}", config.remote), Some("GIT"));
                Ok(repo)
            },
            Err(e) => 
            {
                crate::debug(format!("Error {} while cloning (pub) repo at {}", e, config.remote), Some("GIT"));
                Err(GitError::from(e))
            }
        }
    }
}

pub fn remove_repository(dir: &str) -> Result<(), std::io::Error>
{
    for dir in list_sub_dirs(dir.to_owned())
    {
        set_dir_readonly(&dir, false)?;
    }
    set_dir_readonly(dir, false)?;

    std::fs::remove_dir_all(dir)?;

    Ok(())
}

/// Make a fresh clone if [crate::config::Config::git] is present
///  deleting any file/dir called [crate::config::ContentConfig::path]
pub fn clean_and_clone(dir: &str, config: GitConfig) -> Result<Repository, GitError>
{
    let path = Path::new(dir);
    if path.is_dir()
    {
        remove_repository(dir)?;
    }
    else if path.is_file()
    {
        std::fs::remove_file(path)?;
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
pub fn fast_forward_pull(repo: Repository, git: GitConfig) -> Result<Option<HeadInfo>, GitError>
{
    let branch = git.branch.clone();
    if git.auth.is_some()
    {
        crate::debug(format!("Attempting authenticated pull of {}", git.remote), Some("GIT"));
        let auth = git.auth.unwrap();
        // modified from https://stackoverflow.com/questions/58768910/how-to-perform-git-pull-with-the-rust-git2-crate
        repo.find_remote("origin")?.fetch(&[branch], Some(&mut build_fetch_option(&auth)), None)?;
    }
    else
    {
        crate::debug(format!("Attempting pull of {}", git.remote), Some("GIT"));
        repo.find_remote("origin")?.fetch(&[branch], None, None)?;
    };

    let fetch_head = repo.find_reference("FETCH_HEAD")?;
    let fetch_commit = repo.reference_to_annotated_commit(&fetch_head)?;
    let (analysis, _pref) = repo.merge_analysis(&[&fetch_commit])?;

    if analysis.is_up_to_date()
    {
        Ok(None)
    }
    else if analysis.is_fast_forward()
    {
        let refname = format!("refs/heads/{}", git.branch);
        let mut reference = repo.find_reference(&refname)?;
        reference.set_target(fetch_commit.id(), "Fast-Forward")?;
        repo.set_head(&refname)?;
        repo.checkout_head(Some(git2::build::CheckoutBuilder::default().force()))?;
        Ok(head_info(&repo))
    }
    else
    {
        Err(GitError{why: "Cannot fastforward".to_owned()})
    }
}

/// Commit hash, author and timestamp for head commit
pub struct HeadInfo
{
    pub hash: String,
    pub author_name: String,
    pub author_email: String,
    pub datetime: String 
}

/// Get the [HeadInfo] if it exists
pub fn head_info(repo: &Repository) -> Option<HeadInfo>
{
    let head = match repo.head()
    {
        Ok(h) => match h.target()
        {
            Some(h) => h,
            None => return None
        },
        Err(_) => return None
    };

    match repo.find_commit(head)
    {
        Ok(c) =>
        {
            let dt: DateTime<Utc> = DateTime::from_timestamp(c.time().seconds(), 0).unwrap();
            let name = match c.author().email()
            {
                Some(s) => s.to_string(),
                None => "Unknown".to_string()
            };
            let email = match c.author().name()
            {
                Some(s) => s.to_string(),
                None => "Unknown".to_string()
            };
            Some
            (
                HeadInfo
                {
                    hash: short_oid(c.id()),
                    author_email: email,
                    author_name: name,
                    datetime: format!("{}", dt)
                }
            )
        },
        Err(_) => None
    }
}

/// Get first 7 digits of git hash
pub fn short_oid(id: Oid) -> String
{
    let sid = id.to_string();
    sid[0..min(sid.len(), 7)].to_string()
}