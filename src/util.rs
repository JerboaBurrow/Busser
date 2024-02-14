use core::fmt;
use std::{fmt::{format, Write}, fs::{DirEntry, File}, io::{Read, Write as ioWrite}};
use regex::Regex;

pub fn dump_bytes(v: &[u8]) -> String 
{
    let mut byte_string = String::new();
    for &byte in v
    {
        write!(&mut byte_string, "{:0>2X}", byte).expect("byte dump error");
    };
    byte_string
}

pub fn read_bytes(v: String) -> Vec<u8>
{
    (0..v.len()).step_by(2)
    .map
    (
        |index| u8::from_str_radix(&v[index..index+2], 16).unwrap()
    )
    .collect()
}

pub fn strip_control_characters(s: String) -> String
{
    let re = Regex::new(r"[\u0000-\u001F]").unwrap().replace_all(&s, "");
    return re.to_string()
}

pub fn write_file(path: &str, data: &[u8])
{
    let mut file = File::create(path).unwrap();
    file.write_all(data).unwrap();
}

pub fn read_file_utf8(path: &str) -> Option<String>
{
    let mut file = match File::open(path) {
        Err(why) => 
        {
            crate::debug(format!("error reading file to utf8, {}", why), None);
            return None
        },
        Ok(file) => file,
    };

    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Err(why) => 
        {
            crate::debug(format!("error reading file to utf8, {}", why), None);
            None
        },
        Ok(_) => Some(s)
    }
}

pub fn read_file_bytes(path: &str) -> Option<Vec<u8>>
{
    let mut file = match File::open(path) {
        Err(why) => 
        {
            crate::debug(format!("error reading file to utf8, {}", why), None);
            return None
        },
        Ok(file) => file,
    };

    let mut s: Vec<u8> = vec![];
    match file.read_to_end(&mut s) {
        Err(why) => 
        {
            crate::debug(format!("error reading file to utf8, {}", why), None);
            None
        },
        Ok(_) => Some(s)
    }
}

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
                                    true => found_dirs.push(p),
                                    false => 
                                    {
                                        crate::debug(format!("not a folder: {}", n), None);
                                        continue
                                    }
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

pub fn list_dir_by(pattern: Regex, path: String) -> Vec<String>
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
            
                match pattern.captures(&file_path)
                {
                    Some(_caps) => {found_files.push(file_path.to_string())},
                    None => {continue}
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