pub mod file;
pub mod folder;

/// Set all entries in a dir to permissions().set_readonly(readonly)
pub fn set_dir_readonly(dir: &str, readonly: bool) -> Result<(), std::io::Error>
{
    let paths = std::fs::read_dir(dir.to_owned())?;

    for path in paths
    {
        if path.is_err()
        {
            return Err(path.err().unwrap())
        }

        let path = path.unwrap();
        
        match path.metadata()
        {
            Ok(p) => p.permissions().set_readonly(readonly),
            Err(e) => return Err(e)
        }
    }
    Ok(())
}