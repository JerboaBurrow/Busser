mod common;

#[cfg(test)]
mod git
{
    use std::path::Path;

    use busser::{config::{Config, GitAuthConfig, GitConfig}, integrations::git::{clean_and_clone, fast_forward_pull, from_clone, refresh::GitRefreshTask, remove_repository}};

    #[test]
    pub fn test_clone()
    {
        let config = GitConfig
        {
            remote: "https://github.com/JerboaBurrow/Busser".into(),
            branch: "main".into(),
            checkout_schedule: None,
            auth: None,
            remote_webhook_token: None
        };

        let path = "tests/test_clone";
        let repo = from_clone(path.into(), &config);

        assert!(repo.is_ok());
        assert!(!repo.as_ref().unwrap().is_empty().unwrap());
        assert!(Path::exists(Path::new(path)));

        if Path::exists(Path::new(path))
        {
            let _ = remove_repository(&path);
        }
    }

    #[test]
    pub fn test_clean_and_clone()
    {
        let config = GitConfig
        {
            remote: "https://github.com/JerboaBurrow/Busser".into(),
            branch: "main".into(),
            checkout_schedule: None,
            auth: None,
            remote_webhook_token: None
        };

        let path = "tests/test_clean_and_clone";
        
        {
            let repo = from_clone(path.into(), &config);

            assert!(repo.is_ok());
            assert!(!repo.as_ref().unwrap().is_empty().unwrap());
            assert!(Path::exists(Path::new(path)));
            drop(repo.unwrap());
        }

        let repo = clean_and_clone(path.into(), config);
        println!("{:?}",repo.as_ref().err());
        assert!(repo.is_ok());
        assert!(!repo.as_ref().unwrap().is_empty().unwrap());
        assert!(Path::exists(Path::new(path)));

        if Path::exists(Path::new(path))
        {
            let _ = remove_repository(&path);
        }
    }

    #[test]
    pub fn test_key_authed_clone()
    {
        let config = GitConfig
        {
            remote: "git@github.com:JerboaBurrow/test.git".into(),
            branch: "main".into(),
            checkout_schedule: None,
            remote_webhook_token: None,
            auth: Some(GitAuthConfig
            {
                key_path: Some("not_a_key".into()),
                user: "not_a_user".into(),
                passphrase: "not_a_passphrase".into(),
            }),
        };

        let path = "tests/test_key_authed_clone";
        let repo = from_clone(path.into(), &config);

        assert!(repo.is_err());

        let repo = clean_and_clone(path.into(), config);

        assert!(repo.is_err());

        if Path::exists(Path::new(path))
        {
            let _ = remove_repository(&path);
        }
    }

    #[test]
    pub fn test_pass_authed_clone()
    {
        let config = GitConfig
        {
            remote: "https://github.com/JerboaBurrow/test".into(),
            branch: "main".into(),
            checkout_schedule: None,
            remote_webhook_token: None,
            auth: Some(GitAuthConfig
            {
                key_path: None,
                user: "not_a_user".into(),
                passphrase: "not_a_passphrase".into(),
            }),
        };

        let path = "tests/test_pass_authed_clone";
        let repo = from_clone(path.into(), &config);

        assert!(repo.is_err());

        if Path::exists(Path::new(path))
        {
            let _ = remove_repository(&path);
        }
    }

    #[test]
    pub fn test_fast_forward_pull()
    {
        let config = GitConfig
        {
            remote: "https://github.com/JerboaBurrow/Busser".into(),
            branch: "main".into(),
            checkout_schedule: None,
            remote_webhook_token: None,
            auth: None,
        };

        let path = "tests/test_fast_forward_pull";
        let repo = from_clone(path.into(), &config);

        assert!(repo.is_ok());
        assert!(!repo.as_ref().unwrap().is_empty().unwrap());
        assert!(Path::exists(Path::new(path)));

        assert!(fast_forward_pull(repo.unwrap(), "main").is_ok());

        if Path::exists(Path::new(path))
        {
            let _ = remove_repository(&path);
        }
    }

    #[test]
    pub fn test_pull()
    {
        let git_config = GitConfig
        {
            remote: "https://github.com/JerboaBurrow/Busser".into(),
            branch: "main".into(),
            checkout_schedule: None,
            remote_webhook_token: None,
            auth: None,
        };

        let mut config = Config::default();
        config.git = Some(git_config);

        let path = "tests/test_pull";
        config.content.path = path.to_owned();

        std::fs::create_dir(path).unwrap();
        GitRefreshTask::pull(&config);

        assert!(Path::exists(Path::new(path)));

        if Path::exists(Path::new(path))
        {
            let _ = remove_repository(&path);
        }

    }
}