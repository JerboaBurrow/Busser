use std::{fs::File, io::{Read, Write}};

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