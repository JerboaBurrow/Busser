use std::collections::HashMap;

use regex::Regex;

/// Identifies the MIME type by file extension, no attempt is made to verify the file's content
/// 
/// Supported MIME types in Busser, default is ```"application/octet-stream"```
/// ```rust
/// use std::collections::HashMap;
/// let content_types = HashMap::from
/// ( 
///     [
///         (r"\.txt$", "text/plain"),
///         (r"\.html$", "text/html"),
///         (r"\.css$", "text/css"),
///         (r"\.csv$", "text/csv"),
///         (r"\.(javascript|js)$", "text/javascript"),
///         (r"\.xml$", "text/xml"),
///         (r"\.gif$", "image/gif"), 
///         (r"\.webp$", "image/webp"),   
///         (r"\.(jpg|jpeg)$", "image/jpeg"),   
///         (r"\.png$", "image/png"),   
///         (r"\.tiff$", "image/tiff"),      
///         (r"\.ico$", "image/x-icon"),  
///         (r"\.(djvu)|(djv)$", "image/vnd.djvu"),  
///         (r"\.svg$", "image/svg+xml"),
///         (r"\.(mpeg|mpg|mp2|mpe|mpv|m2v)$", "video/mpeg"),    
///         (r"\.(mp4|m4v)$", "video/mp4"),    
///         (r"\.(qt|mov)$", "video/quicktime"),    
///         (r"\.(wmv)$", "video/x-ms-wmv"),    
///         (r"\.(flv|f4v|f4p|f4a|f4b)$", "video/x-flv"),   
///         (r"\.webm$", "video/webm")    
///     ]
/// );
/// ```
pub fn infer_mime_type(extension: &str) -> &'static str
{
    let content_types = HashMap::from
    ( 
        [
            (r"\.txt$", "text/plain"),
            (r"\.html$", "text/html"),
            (r"\.css$", "text/css"),
            (r"\.csv$", "text/csv"),
            (r"\.(javascript|js)$", "text/javascript"),
            (r"\.xml$", "text/xml"),
            (r"\.gif$", "image/gif"), 
            (r"\.webp$", "image/webp"),   
            (r"\.(jpg|jpeg)$", "image/jpeg"),   
            (r"\.png$", "image/png"),   
            (r"\.tiff$", "image/tiff"),      
            (r"\.ico$", "image/x-icon"),  
            (r"\.(djvu)|(djv)$", "image/vnd.djvu"),  
            (r"\.svg$", "image/svg+xml"),
            (r"\.(mpeg|mpg|mp2|mpe|mpv|m2v)$", "video/mpeg"),    
            (r"\.(mp4|m4v)$", "video/mp4"),    
            (r"\.(qt|mov)$", "video/quicktime"),    
            (r"\.(wmv)$", "video/x-ms-wmv"),    
            (r"\.(flv|f4v|f4p|f4a|f4b)$", "video/x-flv"),   
            (r"\.webm$", "video/webm")    
        ]
    );

    for (re, content) in content_types
    {
        if Regex::new(re).unwrap().is_match(&extension)
        {
            return content
        }
    }

    "application/octet-stream"
}

pub fn is_html(mime: String) -> bool
{
    mime == "text/html"
}

pub fn is_image(mime: String) -> bool
{
    mime.contains("image")
}

pub fn is_video(mime: String) -> bool
{
    mime.contains("video")
}