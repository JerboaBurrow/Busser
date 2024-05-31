mod common;

#[cfg(test)]
mod mime
{
    use busser::content::mime_type::{Mime, MIME};


    #[test]
    fn test_mime()
    {
        assert!(MIME::TextPlain.is_text());
        assert!(MIME::TextHtml.is_text());
        assert!(MIME::TextCSS.is_text());
        assert!(MIME::TextCSV.is_text());
        assert!(MIME::TextJavascript.is_text());
        assert!(MIME::TextXML.is_text());
        assert!(!MIME::ImageDJV.is_text());
        assert!(!MIME::ImageGIF.is_text());
        assert!(!MIME::ImageJPEG.is_text());
        assert!(!MIME::ImagePNG.is_text());
        assert!(!MIME::ImageSVG.is_text());
        assert!(!MIME::ImageTIFF.is_text());
        assert!(!MIME::ImageWEBP.is_text());
        assert!(!MIME::ImageXICON.is_text());
        assert!(!MIME::VideoFLV.is_text());
        assert!(!MIME::VideoMP4.is_text());
        assert!(!MIME::VideoMPEG.is_text());
        assert!(!MIME::VideoQuicktime.is_text());
        assert!(!MIME::VideoWEBM.is_text());
        assert!(!MIME::VideoWMV.is_text());
        assert!(!MIME::Unknown.is_text());

        assert!(!MIME::TextPlain.is_html());
        assert!(MIME::TextHtml.is_html());
        assert!(!MIME::TextCSS.is_html());
        assert!(!MIME::TextCSV.is_html());
        assert!(!MIME::TextJavascript.is_html());
        assert!(!MIME::TextXML.is_html());
        assert!(!MIME::ImageDJV.is_html());
        assert!(!MIME::ImageGIF.is_html());
        assert!(!MIME::ImageJPEG.is_html());
        assert!(!MIME::ImagePNG.is_html());
        assert!(!MIME::ImageSVG.is_html());
        assert!(!MIME::ImageTIFF.is_html());
        assert!(!MIME::ImageWEBP.is_html());
        assert!(!MIME::ImageXICON.is_html());
        assert!(!MIME::VideoFLV.is_html());
        assert!(!MIME::VideoMP4.is_html());
        assert!(!MIME::VideoMPEG.is_html());
        assert!(!MIME::VideoQuicktime.is_html());
        assert!(!MIME::VideoWEBM.is_html());
        assert!(!MIME::VideoWMV.is_html());
        assert!(!MIME::Unknown.is_html());

        assert!(!MIME::TextPlain.is_image());
        assert!(!MIME::TextHtml.is_image());
        assert!(!MIME::TextCSS.is_image());
        assert!(!MIME::TextCSV.is_image());
        assert!(!MIME::TextJavascript.is_image());
        assert!(!MIME::TextXML.is_image());
        assert!(MIME::ImageDJV.is_image());
        assert!(MIME::ImageGIF.is_image());
        assert!(MIME::ImageJPEG.is_image());
        assert!(MIME::ImagePNG.is_image());
        assert!(MIME::ImageSVG.is_image());
        assert!(MIME::ImageTIFF.is_image());
        assert!(MIME::ImageWEBP.is_image());
        assert!(MIME::ImageXICON.is_image());
        assert!(!MIME::VideoFLV.is_image());
        assert!(!MIME::VideoMP4.is_image());
        assert!(!MIME::VideoMPEG.is_image());
        assert!(!MIME::VideoQuicktime.is_image());
        assert!(!MIME::VideoWEBM.is_image());
        assert!(!MIME::VideoWMV.is_image());
        assert!(!MIME::Unknown.is_image());

        assert!(!MIME::TextPlain.is_video());
        assert!(!MIME::TextHtml.is_video());
        assert!(!MIME::TextCSS.is_video());
        assert!(!MIME::TextCSV.is_video());
        assert!(!MIME::TextJavascript.is_video());
        assert!(!MIME::TextXML.is_video());
        assert!(!MIME::ImageDJV.is_video());
        assert!(!MIME::ImageGIF.is_video());
        assert!(!MIME::ImageJPEG.is_video());
        assert!(!MIME::ImagePNG.is_video());
        assert!(!MIME::ImageSVG.is_video());
        assert!(!MIME::ImageTIFF.is_video());
        assert!(!MIME::ImageWEBP.is_video());
        assert!(!MIME::ImageXICON.is_video());
        assert!(MIME::VideoFLV.is_video());
        assert!(MIME::VideoMP4.is_video());
        assert!(MIME::VideoMPEG.is_video());
        assert!(MIME::VideoQuicktime.is_video());
        assert!(MIME::VideoWEBM.is_video());
        assert!(MIME::VideoWMV.is_video());
        assert!(!MIME::Unknown.is_video());

        assert_eq!(MIME::TextPlain.as_str(), "text/plain");
        assert_eq!(MIME::TextHtml.as_str(), "text/html");
        assert_eq!(MIME::TextCSS.as_str(), "text/css");
        assert_eq!(MIME::TextCSV.as_str(), "text/csv");
        assert_eq!(MIME::TextJavascript.as_str(), "text/javascript");
        assert_eq!(MIME::TextXML.as_str(), "text/xml");
        assert_eq!(MIME::ImageDJV.as_str(), "image/vnd.djvu");
        assert_eq!(MIME::ImageGIF.as_str(), "image/gif");
        assert_eq!(MIME::ImageJPEG.as_str(), "image/jpeg");
        assert_eq!(MIME::ImagePNG.as_str(), "image/png");
        assert_eq!(MIME::ImageSVG.as_str(), "image/svg+xml");
        assert_eq!(MIME::ImageTIFF.as_str(), "image/tiff");
        assert_eq!(MIME::ImageWEBP.as_str(), "image/webp");
        assert_eq!(MIME::ImageXICON.as_str(), "image/x-icon");
        assert_eq!(MIME::VideoFLV.as_str(), "video/x-flv");
        assert_eq!(MIME::VideoMP4.as_str(), "video/mp4");
        assert_eq!(MIME::VideoMPEG.as_str(), "video/mpeg");
        assert_eq!(MIME::VideoQuicktime.as_str(), "video/quicktime");
        assert_eq!(MIME::VideoWEBM.as_str(), "video/webm");
        assert_eq!(MIME::VideoWMV.as_str(), "video/x-ms-wmv");
        assert_eq!(MIME::Unknown.as_str(), "application/octet-stream");

        assert!(!MIME::TextPlain.in_sitemap());
        assert!(MIME::TextHtml.in_sitemap());
        assert!(!MIME::TextCSS.in_sitemap());
        assert!(!MIME::TextCSV.in_sitemap());
        assert!(!MIME::TextJavascript.in_sitemap());
        assert!(!MIME::TextXML.in_sitemap());
        assert!(MIME::ImageDJV.in_sitemap());
        assert!(MIME::ImageGIF.in_sitemap());
        assert!(MIME::ImageJPEG.in_sitemap());
        assert!(MIME::ImagePNG.in_sitemap());
        assert!(MIME::ImageSVG.in_sitemap());
        assert!(MIME::ImageTIFF.in_sitemap());
        assert!(MIME::ImageWEBP.in_sitemap());
        assert!(MIME::ImageXICON.in_sitemap());
        assert!(MIME::VideoFLV.in_sitemap());
        assert!(MIME::VideoMP4.in_sitemap());
        assert!(MIME::VideoMPEG.in_sitemap());
        assert!(MIME::VideoQuicktime.in_sitemap());
        assert!(MIME::VideoWEBM.in_sitemap());
        assert!(MIME::VideoWMV.in_sitemap());
        assert!(!MIME::Unknown.in_sitemap());
    }

}