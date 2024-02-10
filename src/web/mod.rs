use axum::
{
    http::StatusCode,
    body::Bytes
};

use openssl::{pkey::PKey, sign::Signer, hash::MessageDigest, memcmp};
use regex::Regex;

use crate::util::{read_bytes, dump_bytes};

pub mod throttle;
pub mod response;
pub mod discord;

pub fn is_authentic
(
    hmac_token: String, 
    sender_hmac: String, 
    body: Bytes
) -> StatusCode
{
    let post_digest = Regex::new(r"sha256=").unwrap().replace_all(&sender_hmac, "").into_owned().to_uppercase();

    let key = match PKey::hmac(hmac_token.as_bytes())
    {
        Ok(k) => k,
        Err(_) => 
        {
            crate::debug("key creation failure".to_string(), None);
            return StatusCode::INTERNAL_SERVER_ERROR
        }
    };

    let mut signer = match Signer::new(MessageDigest::sha256(), &key)
    {
        Ok(k) => k,
        Err(_) => 
        {
            crate::debug("signer creation failure".to_string(), None);
            return StatusCode::INTERNAL_SERVER_ERROR
        }
    };
    
    match signer.update(&body)
    {
        Ok(k) => k,
        Err(_) => 
        {
            crate::debug("signing update failure".to_string(), None);
            return StatusCode::INTERNAL_SERVER_ERROR
        }
    };

    let hmac = match signer.sign_to_vec()
    {
        Ok(k) => k,
        Err(_) => 
        {
            crate::debug("sign failure".to_string(), None);
            return StatusCode::INTERNAL_SERVER_ERROR
        }
    };

    crate::debug(format!("post_digtest: {}, len: {}\nlocal hmac: {}, len: {}", post_digest, post_digest.len(), dump_bytes(&hmac), dump_bytes(&hmac).len()), None);

    match memcmp::eq(&hmac, &read_bytes(post_digest.clone()))
    {
        true => {},
        false => 
        {
            crate::debug(format!("bad signature: local/post\n{}\n{}", post_digest, dump_bytes(&hmac)), None);
            return StatusCode::UNAUTHORIZED
        }
    }

    // it is now safe to process the POST request

    StatusCode::ACCEPTED
}