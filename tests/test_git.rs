mod common;

#[cfg(test)]
mod git
{
    use std::{fs::remove_dir_all, path::Path};

    use busser::{config::GitConfig, integrations::git::from_clone};

    #[test]
    pub fn test_clone()
    {
        let config = GitConfig
        {
            remote: "https://github.com/JerboaBurrow/Busser".into(),
            branch: "main".into(),
            auth: None,
        };

        let repo = from_clone("tests/test-repo".into(), &config);

        assert!(!repo.as_ref().unwrap().is_empty().unwrap());
        assert!(repo.is_ok());
        assert!(Path::exists(Path::new("tests/test-repo")));

        if Path::exists(Path::new("tests/test-repo"))
        {
            let _ = remove_dir_all(Path::new("tests/test-repo"));
        }
    }
}