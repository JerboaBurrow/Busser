use std::{fmt, fs, io::{Read, Write}};

#[derive(Debug, Clone)]
pub struct FileNotReadError
{
    pub why: String
}

impl fmt::Display for FileNotReadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.why)
    }
}

/// A trait for writeable and loadable data from disk
pub trait File
{
    fn write_bytes(&self);
    fn read_bytes(&self) -> Option<Vec<u8>>;
    fn read_utf8(&self) -> Option<String>;
}

pub fn write_file_bytes(path: &str, data: &[u8])
{
    let mut file = fs::File::create(path).unwrap();
    file.write_all(data).unwrap();
}

pub fn read_file_utf8(path: &str) -> Option<String>
{
    let mut file = match fs::File::open(path) {
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
    let mut file = match fs::File::open(path) {
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