mod common;

#[cfg(test)]
mod hmac
{
    use axum::{body::Bytes, http::{HeaderMap, HeaderValue}};
    use busser::{integrations::is_authentic, util::dump_bytes};
    use openssl::{hash::MessageDigest, pkey::PKey, sha::sha256, sign::Signer};
    use reqwest::StatusCode;

    const TOKEN: &str = "A_SECRET_TOKEN";
    const KEY: &str = "HMAC_TOKEN_HEADER_KEY";

    #[test]
    fn test_hmac()
    {
        let mut headers = HeaderMap::new();
        let body = Bytes::from_static("this_is_a_body".as_bytes());

        assert_eq!(is_authentic(&headers, KEY, TOKEN.to_string(), &body), StatusCode::UNAUTHORIZED);

        let not_an_hmac = "a";
        headers.append(KEY, HeaderValue::from_str(&not_an_hmac).unwrap());
        assert_eq!(is_authentic(&headers, KEY, TOKEN.to_string(), &body), StatusCode::BAD_REQUEST);

        let invalid_hmac = dump_bytes(&sha256("NOT VALID".as_bytes()));
        headers = HeaderMap::new();
        headers.append(KEY, HeaderValue::from_str(&invalid_hmac).unwrap());
        assert_eq!(is_authentic(&headers, KEY, TOKEN.to_string(), &body), StatusCode::UNAUTHORIZED);

        let key = PKey::hmac(TOKEN.as_bytes()).unwrap();
        let mut signer = Signer::new(MessageDigest::sha256(), &key).unwrap();
        signer.update(&body).unwrap();
        let hmac = dump_bytes(&signer.sign_to_vec().unwrap());

        headers = HeaderMap::new();
        headers.append(KEY, HeaderValue::from_str(&hmac).unwrap());
        assert_eq!(is_authentic(&headers, KEY, TOKEN.to_string(), &body), StatusCode::ACCEPTED);
    }
}