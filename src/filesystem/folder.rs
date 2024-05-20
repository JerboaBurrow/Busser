use core::fmt;
use std::fs::DirEntry;

use regex::Regex;

#[derive(Debug, Clone)]
pub struct ListDirError
{
    pub why: String
}

impl fmt::Display for ListDirError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.why)
    }
}

/// List all files in path
pub fn list_dir(path: String) -> Result<std::fs::ReadDir, ListDirError>
{
    match std::fs::read_dir(path)
    {
        Ok(files) => 
        {
            Ok(files)
        },
        Err(why) => 
        {
            Err(ListDirError { why: format!("{}", why)})
        }
    }
}

/// Return parsed [std::ffi::OsString] from [DirEntry]
pub fn dir_entry_to_path(d: DirEntry) -> Option<String>
{
    let file_os_string = d.file_name();

    match file_os_string.to_str()
    {
        Some(name) => Some(name.to_string()),
        None =>
        {
            crate::debug(format!("could not load file name: {:?}", file_os_string), None);
            None
        }
    }
}

/// List all subdirectories of a path
pub fn list_sub_dirs(path: String) -> Vec<String>
{
    let mut found_dirs: Vec<String> = vec![];
    match std::fs::read_dir(path.clone())
    {
        Ok(files) => 
        {
            
            for file in files
            {
                let name = match file
                {
                    Ok(d) => dir_entry_to_path(d),
                    Err(e) =>
                    {
                        crate::debug(format!("could not load file name: {}", e), None);
                        continue
                    }
                };

                match name 
                {
                    Some(n) =>
                    {
                        let p = path.clone()+"/"+&n;
                        match std::fs::metadata(p.clone())
                        {
                            Ok(md) =>
                            {
                                match md.is_dir()
                                {
                                    true => {found_dirs.push(p.clone()); crate::debug(format!("found folder: {}", p), None)},
                                    false => {continue}
                                }
                            },
                            Err(e) =>
                            {
                                crate::debug(format!("error getting file: {}", e), None);
                                continue
                            }
                        }
                    },
                    None => continue
                }
            } 
        },
        Err(why) => 
        {
            crate::debug(format!("Error reading dir {}\n {}", path, why), None); 
        }
    }

    found_dirs
}

/// List all files conforming to an [Option] [Regex]
pub fn list_dir_by(pattern: Option<Regex>, path: String) -> Vec<String>
{
    match std::fs::read_dir(path.clone())
    {
        Ok(files) => 
        {
            let mut found_files: Vec<String> = vec![];
            for file in files 
            {
                
                let file_name = match file
                {
                    Ok(d) => dir_entry_to_path(d),
                    Err(e) =>
                    {
                        crate::debug(format!("could not load file name: {}", e), None);
                        continue
                    }
                };

                let file_path = match file_name
                {
                    Some(name) => path.clone() + "/" + &name,
                    None => continue
                };
            
                if pattern.clone().is_some()
                {
                    match pattern.clone().unwrap().captures(&file_path)
                    {
                        Some(_caps) => {found_files.push(file_path.to_string())},
                        None => {continue}
                    }
                }
                else
                {
                    found_files.push(file_path.to_string())
                }
            }

            return found_files
        },
        Err(why) => 
        {
            crate::debug(format!("Error reading dir {}\n {}", path, why), None); 
        }
    }
    vec![]
}