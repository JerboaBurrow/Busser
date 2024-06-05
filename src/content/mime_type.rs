use std::collections::HashMap;

use regex::Regex;
use serde::{Deserialize, Serialize};

/// Identifies the MIME type by file extension, no attempt is made to verify the file's content
/// 
/// Supported MIME types in Busser, default is ```"application/octet-stream"```
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum MIME 
{
    TextPlain,
    TextHtml,
    TextCSS,
    TextCSV,
    TextJavascript,
    TextXML,
    ImageGIF,
    ImagePNG,
    ImageJPEG,
    ImageWEBP,
    ImageTIFF,
    ImageXICON,
    ImageDJV,
    ImageSVG,
    VideoMPEG,
    VideoMP4,
    VideoQuicktime,
    VideoWMV,
    VideoFLV,
    VideoWEBM,
    Unknown
}

const IS_HTML: [bool; 21] = 
    [
        false, true, false, false, false, false, false, false, false, false, false, 
        false, false, false, false, false, false, false, false, false, false
    ];

const IS_TEXT: [bool; 21] = 
    [
        true, true, true, true, true, true, false, false, false, false, false, 
        false, false, false, false, false, false, false, false, false, false
    ];

const IS_IMAGE: [bool; 21] = 
    [
        false, false, false, false, false, false, true, true, true, true, true, 
        true, true, true, false, false, false, false, false, false, false
    ];

const IS_VIDEO: [bool; 21] = 
    [
        false, false, false, false, false, false, false, false, false, false, 
        false, false, false, false, true, true, true, true, true, true, false
    ];

const STR_FORM: [&'static str; 21] = 
    [
        "text/plain", "text/html", "text/css", "text/csv", "text/javascript", "text/xml", 
        "image/gif", "image/png", "image/jpeg", "image/webp", "image/tiff", "image/x-icon",
        "image/vnd.djvu", "image/svg+xml", "video/mpeg", "video/mp4", "video/quicktime",
        "video/x-ms-wmv", "video/x-flv", "video/webm", "application/octet-stream"
    ];
pub trait Mime 
{
    fn is_html(&self) -> bool {false}
    fn is_text(&self) -> bool {false}
    fn is_image(&self) -> bool {false}
    fn is_video(&self) -> bool {false}
    
    fn in_sitemap(&self) -> bool { self.is_html() || self.is_image() || self.is_video() }

    fn as_str(&self) -> &'static str {"application/octet-stream"}
    fn infer_mime_type(extension: &str) -> MIME;
}

impl Mime for MIME
{
    fn is_html(&self) -> bool {IS_HTML[self.clone() as usize]}
    fn is_text(&self) -> bool {IS_TEXT[self.clone() as usize]}
    fn is_image(&self) -> bool {IS_IMAGE[self.clone() as usize]}
    fn is_video(&self) -> bool {IS_VIDEO[self.clone() as usize]}
    fn as_str(&self) -> &'static str { STR_FORM[self.clone() as usize] }

    fn infer_mime_type(extension: &str) -> MIME
    {
        let content_types = HashMap::from
        ( 
            [
                (r"\.txt$", MIME::TextPlain),
                (r"\.html$", MIME::TextHtml),
                (r"\.css$", MIME::TextCSS),
                (r"\.csv$", MIME::TextCSV),
                (r"\.(javascript|js)$", MIME::TextJavascript),
                (r"\.xml$", MIME::TextXML),
                (r"\.gif$", MIME::ImageGIF), 
                (r"\.webp$", MIME::ImageWEBP),   
                (r"\.(jpg|jpeg)$", MIME::ImageJPEG),   
                (r"\.png$", MIME::ImagePNG),   
                (r"\.tiff$", MIME::ImageTIFF),      
                (r"\.ico$", MIME::ImageXICON),  
                (r"\.(djvu)|(djv)$", MIME::ImageDJV),  
                (r"\.svg$", MIME::ImageSVG),
                (r"\.(mpeg|mpg|mp2|mpe|mpv|m2v)$", MIME::VideoMPEG),    
                (r"\.(mp4|m4v)$", MIME::VideoMP4),    
                (r"\.(qt|mov)$", MIME::VideoQuicktime),    
                (r"\.(wmv)$", MIME::VideoWMV),    
                (r"\.(flv|f4v|f4p|f4a|f4b)$", MIME::VideoFLV),   
                (r"\.webm$", MIME::VideoWEBM)    
            ]
        );

        for (re, content) in content_types
        {
            if Regex::new(re).unwrap().is_match(&extension)
            {
                return content
            }
        }

        MIME::Unknown
    }
}