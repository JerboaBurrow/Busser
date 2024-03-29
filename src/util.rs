use core::fmt;
use std::{fmt::Write, fs::{DirEntry, File}, io::{Read, Write as ioWrite}};
use libflate::deflate::{Encoder, Decoder};
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

pub fn matches_one(uri: &str, patterns: &Vec<String>) -> bool
{
    let mut ignore = false;  
    for re_string in patterns.into_iter()
    {
        let re = match Regex::new(re_string.as_str())
        {
            Ok(r) => r,
            Err(e) => 
            {crate::debug(format!("Could not parse content ingnore regex\n{e}\n Got {re_string}"), None); continue;}
        };

        if re.is_match(uri)
        {
            crate::debug(format!("Ignoring {} due to pattern {re_string}", uri), None);
            ignore = true;
            break;
        }
    }
    ignore
}

#[derive(Debug, Clone)]
pub struct CompressionError
{
    pub why: String
}

impl fmt::Display for CompressionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.why)
    }
}

pub fn compress(bytes: &[u8]) -> Result<Vec<u8>, CompressionError>
{
    let mut encoder = Encoder::new(Vec::new());
    
    match encoder.write_all(&bytes)
    {
        Ok(_) => (),
        Err(e) => 
        {
            return Err(CompressionError { why: format!("Error writing to compressor: {}", e) })
        }
    };

    match encoder.finish().into_result()
    {
        Ok(data) => Ok(data), 
        Err(e) => 
        {
            Err(CompressionError { why: format!("Error finalising compressor: {}", e) })
        }
    }
}

pub fn decompress(bytes: Vec<u8>) -> Result<String, CompressionError>
{
    let mut decoder = Decoder::new(&bytes[..]);
    let mut decoded_data = Vec::new();

    match decoder.read_to_end(&mut decoded_data)
    {
        Ok(_) => (),
        Err(e) => 
        {
            return Err(CompressionError { why: format!("Error decoding data: {}", e) })
        }
    }
    
    match std::str::from_utf8(&decoded_data)
    {
        Ok(s) => Ok(s.to_string()),
        Err(e) => 
        {
            Err(CompressionError { why: format!("Decoded data is not utf8: {}", e) })
        }
    }
}
