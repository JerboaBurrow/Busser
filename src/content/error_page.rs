use crate::{config::Config, filesystem::file::read_file_utf8};

pub const DEFAULT_BODY: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
	<meta charset="utf-8">
	<meta http-equiv="X-UA-Compatible" content="IE=edge">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
	<title>Error ERROR_CODE</title>
</head>

<style>
    .page {
        display: table;
        position: absolute;
        top: 0;
        left: 0;
        height: 100%;
        width: 100%;
    }

    .content {
        display: table-cell;
        vertical-align: middle;
        margin-left: auto;
        margin-right: auto;
        width: 75vw;
        text-align: center;
    }

    a:link, a:visited {
        transition-duration: 0.4s;
        border: none;
        color: rgb(25, 25, 25);
        border: 2px solid rgb(25, 25, 25);
        padding: 16px 32px;
        text-align: center;
        text-decoration: none;
        display: inline-block;
        font-size: 16px;
        margin: 4px 2px;
        transition-duration: 0.4s;
        cursor: pointer;
    }

    a:hover, a:active {
        background-color: rgb(25, 25, 25);
        color: rgb(230, 230, 230);
    }
</style>

<body>
    <div class="page">
        <div class="content">
            <h1>That's a ERROR_CODE error.</h1>
            <a href="LINK_TO_HOME">Let's go home.</a>
        </div>
    </div>
</body>
</html>
"#;

pub struct ErrorPage
{
    pub body_template: String
}

impl ErrorPage
{
    fn expand_template(template: String, config: &Config) -> String
    {
        let mut domain = config.domain.to_string();
        if !domain.starts_with("https://") {domain = format!("https://{domain}")}
        template.replace("LINK_TO_HOME", &domain)
    }

    pub fn expand_error_code(&self, code: &str) -> String
    {
        self.body_template.replace("ERROR_CODE", code)
    }

    pub fn from(config: &Config) -> ErrorPage
    {
        if let Some(ref path) = config.content.error_template
        {
            if let Some(body) = read_file_utf8(&path)
            {
                return ErrorPage {body_template: Self::expand_template(body, config)}
            }
        }
        ErrorPage {body_template: Self::expand_template(DEFAULT_BODY.to_string(), config)}
    }
}