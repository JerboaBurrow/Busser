mod common;

#[cfg(test)]
mod util
{
    use busser::util::{date_now, date_to_rfc3339, hash, matches_one, read_bytes, strip_control_characters};

    use busser::util::{compress, compress_string, decompress, decompress_utf8_string};
    use chrono::{DateTime, Datelike};

    #[test]
    fn test_compress_decompress()
    {
        let data = "this is some data".as_bytes();
        let compressed = compress(data);
        assert!(compressed.is_ok());
        let decompressed = decompress(compressed.unwrap());
        assert!(decompressed.is_ok());
        assert_eq!(data, decompressed.unwrap());
    }

    #[test]
    fn test_compress_decompress_strings()
    {
        let data = "this is some data".to_string();
        let compressed = compress_string(&data);
        assert!(compressed.is_ok());
        let decompressed = decompress_utf8_string(compressed.unwrap());
        assert!(decompressed.is_ok());
        assert_eq!(data, decompressed.unwrap());
    }

    #[test]
    fn test_hash()
    {
        let hashed = hash("00".as_bytes().to_vec());
        let expected: Vec<u8> = vec![241, 83, 67, 146, 39, 155, 221, 191, 157, 67, 221, 232, 112, 28, 181, 190, 20, 184, 47, 118, 236, 102, 7, 191, 141, 106, 213, 87, 246, 15, 48, 78];
        assert_eq!(hashed, expected);

        let hashed = hash("from openssl command line".as_bytes().to_vec());
        let expected: Vec<u8> = vec![36, 48, 61, 185, 111, 196, 129, 155, 155, 187, 39, 255, 34, 84, 74, 189, 132, 168, 13, 60, 207, 212, 76, 98, 219, 209, 139, 83, 132, 78, 50, 115];
        assert_eq!(hashed, expected);
    }

    #[test]
    fn test_matches_one()
    {
        let uri = "this/is/some/uri.txt";

        assert!(matches_one(uri, &vec!["this".to_string()]));
        assert!(matches_one(uri, &vec![r"\.txt$".to_string()]));
        assert!(!matches_one(uri, &vec!["rnaomd".to_string()]));
        assert!(matches_one(uri, &vec!["rnaomd".to_string(), r"\.txt$".to_string()]));
        assert!(matches_one(uri, &vec!["this".to_string(), r"\.txt$".to_string()]));
        assert!(matches_one(uri, &vec!["rnaomd".to_string(),"this".to_string(), r"\.txt$".to_string()]));
        assert!(!matches_one(uri, &vec!["rnaomd".to_string(), "adsklfaldk".to_string(), "adskgkfld".to_string()]));
    }

    #[test]
    fn test_read_bytes()
    {
        let expected = vec![36, 48, 61, 185, 111, 196, 129, 155, 155, 187, 39, 255, 34, 84, 74, 189, 132, 168, 13, 60, 207, 212, 76, 98, 219, 209, 139, 83, 132, 78, 50, 115];
        let actual = read_bytes("24303db96fc4819b9bbb27ff22544abd84a80d3ccfd44c62dbd18b53844e3273".to_string());
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_date_utils()
    {
        let date = date_now();
        println!("{}", date);
        let date_rfc3339 = date_to_rfc3339(&date).unwrap();
        let now = chrono::offset::Utc::now();

        assert_eq!(date, format!("{:0>4}-{:0>2}-{:0>2}", now.year(), now.month(), now.day()));
        assert_eq!(date_rfc3339, DateTime::parse_from_rfc3339(&format!("{}T00:00:00.000000000+00:00", date)).unwrap());
    }

    #[test]
    fn test_strip_control()
    {
        for character in 0..0x1F
        {
            let ch = char::from_u32(character).unwrap();
            let test_string = format!("{}a_test_string", ch);
            assert_eq!(strip_control_characters(test_string), "a_test_string");
        }
    }
}