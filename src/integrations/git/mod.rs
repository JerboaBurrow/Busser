use std::path::Path;

use git2::{Cred, RemoteCallbacks, Repository};

use crate::config::GitConfig;

/// Attempt to clone a remote repo from a [crate::config::GitConfig]
pub fn from_clone(path: String, config: &GitConfig) -> Result<Repository, git2::Error>
{
    if let GitConfig{auth: Some(_), remote: _, branch: _} = config
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
                Err(e)
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
                Err(e)
            }
        }
    }
}