use core::fmt;
use std::{fmt::Write, io::{Read, Write as ioWrite}, time::Instant};
use chrono::{DateTime, Datelike, FixedOffset};
use libflate::deflate::{Encoder, Decoder};
use openssl::sha::Sha256;
use regex::Regex;

use crate::BLAZING;

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

pub fn decompress(bytes: Vec<u8>) -> Result<Vec<u8>, CompressionError>
{
    let mut decoder = Decoder::new(&bytes[..]);
    let mut decoded_data = Vec::new();

    match decoder.read_to_end(&mut decoded_data)
    {
        Ok(_) => Ok(decoded_data),
        Err(e) => 
        {
            Err(CompressionError { why: format!("Error decoding data: {}", e) })
        }
    }
}

pub fn compress_string(s: &String) -> Result<Vec<u8>, CompressionError>
{
    compress(s.as_bytes())
}

pub fn decompress_utf8_string(compressed: Vec<u8>) -> Result<String, CompressionError>
{
    let decoded_data = match decompress(compressed)
    {
        Ok(d) => d,
        Err(e) => return Err(e)
    };

    match std::str::from_utf8(&decoded_data)
    {
        Ok(s) => Ok(s.to_string()),
        Err(e) => 
        {
            Err(CompressionError { why: format!("Decoded data is not utf8: {}", e) })
        }
    }
}

pub fn hash(v: Vec<u8>) -> Vec<u8>
{
    let mut sha = Sha256::new();
    sha.update(&v);
    sha.finish().to_vec()
}

pub fn format_elapsed(tic: Instant) -> String
{
    match tic.elapsed().as_millis()
    {
        0..=999 => 
        {
            format!("{}ms {}",tic.elapsed().as_millis(), String::from_utf8(BLAZING.to_vec()).unwrap())
        },
        _ => 
        {
            format!("{}s",tic.elapsed().as_secs())
        }
    }
}

pub fn date_now() -> String
{
    let date = chrono::offset::Utc::now();
    format!("{:0>4}-{:0>2}-{:0>2}", date.year(), date.month(), date.day())
}

pub fn date_to_rfc3339(date: &str) -> Result<DateTime<FixedOffset>, chrono::ParseError>
{
    DateTime::parse_from_rfc3339(format!("{}T00:00:00+00:00", date).as_str())
}