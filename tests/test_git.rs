mod common;

#[cfg(test)]
mod git
{
    use std::{fs::remove_dir_all, path::Path};

    use busser::{config::{GitAuthConfig, GitConfig}, integrations::git::{clean_and_clone, fast_forward_pull, from_clone}};

    #[test]
    pub fn test_clone()
    {
        let config = GitConfig
        {
            remote: "https://github.com/JerboaBurrow/Busser".into(),
            branch: "main".into(),
            checkout_schedule: None,
            auth: None,
        };

        let path = "tests/test_clone";
        let repo = from_clone(path.into(), &config);

        assert!(repo.is_ok());
        assert!(!repo.as_ref().unwrap().is_empty().unwrap());
        assert!(Path::exists(Path::new(path)));

        if Path::exists(Path::new(path))
        {
            let _ = remove_dir_all(Path::new(path));
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
        };

        let path = "tests/test_clean_and_clone";
        let repo = from_clone(path.into(), &config);

        assert!(repo.is_ok());
        assert!(!repo.as_ref().unwrap().is_empty().unwrap());
        assert!(Path::exists(Path::new(path)));

        let repo = clean_and_clone(path.into(), config);
        println!("{:?}",repo.as_ref().err());
        assert!(repo.is_ok());
        assert!(!repo.as_ref().unwrap().is_empty().unwrap());
        assert!(Path::exists(Path::new(path)));

        if Path::exists(Path::new(path))
        {
            let _ = remove_dir_all(Path::new(path));
        }
    }

    #[test]
    pub fn test_key_authed_clone()
    {
        let config = GitConfig
        {
            remote: "https://github.com/JerboaBurrow/test".into(),
            branch: "main".into(),
            checkout_schedule: None,
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
            let _ = remove_dir_all(Path::new(path));
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
            let _ = remove_dir_all(Path::new(path));
        }
    }

    #[test]
    pub fn test_pull()
    {
        let config = GitConfig
        {
            remote: "https://github.com/JerboaBurrow/Busser".into(),
            branch: "main".into(),
            checkout_schedule: None,
            auth: None,
        };

        let path = "tests/test_pull";
        let repo = from_clone(path.into(), &config);

        assert!(repo.is_ok());
        assert!(!repo.as_ref().unwrap().is_empty().unwrap());
        assert!(Path::exists(Path::new(path)));

        assert!(fast_forward_pull(repo.unwrap(), "main").is_ok());

        if Path::exists(Path::new(path))
        {
            let _ = remove_dir_all(Path::new(path));
        }
    }
}