use core::fmt;
use std::{fmt::Write, io::{Read, Write as ioWrite}};
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
