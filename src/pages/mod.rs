use std::path::Path;

use crate::{server::model::{Config, CONFIG_PATH}, util::{list_dir, read_file_utf8}};

use self::page::Page;

pub mod page;

pub fn read_pages(path: Option<&str>) -> Vec<Page>
{

    let config_path: &str = match path
    {
        Some(s) => s,
        None => CONFIG_PATH
    };

    let config = if Path::new(config_path).exists()
    {
        let data = match read_file_utf8(config_path)
        {
            Some(d) => d,
            None =>
            {
                println!("Error reading configuration file {} no data", CONFIG_PATH);
                std::process::exit(1);
            }
        };

        let config: Config = match serde_json::from_str(&data)
        {
            Ok(data) => {data},
            Err(why) => 
            {
                println!("Error reading configuration file {}\n{}", CONFIG_PATH, why);
                std::process::exit(1);
            }
        };

        config
    }
    else 
    {
        println!("Error configuration file {} does not exist", CONFIG_PATH);
        std::process::exit(1);
    };

    let path = config.get_path();

    match list_dir(path.clone())
    {
        Ok(files) =>
        {
            let mut p: Vec<Page> = Vec::new();
            for file in files
            {

                let file_os_string = match file
                {
                    Ok(d) => d.file_name(),
                    Err(e) =>
                    {
                        crate::debug(format!("could not load file name: {}", e), None);
                        continue
                    }
                };

                let file_name = match file_os_string.to_str()
                {
                    Some(name) => {name},
                    None =>
                    {
                        crate::debug(format!("could not load file name: {:?}", file_os_string), None);
                        continue
                    }
                };

                let file_path = path.clone()+"/"+file_name;

                let file_string = match std::fs::metadata(file_path.clone())
                {
                    Ok(md) =>
                    {
                        match md.is_file()
                        {
                            true => file_path,
                            false => 
                            {
                                crate::debug(format!("not a file: {}", file_path), None);
                                continue
                            }
                        }
                    },
                    Err(e) =>
                    {
                        crate::debug(format!("error getting file: {}", e), None);
                        continue
                    }
                };

                let data = match read_file_utf8(&file_string)
                {
                    Some(s) => s,
                    None =>
                    {
                        crate::debug(format!("got no data from file: {}", file_string), None);
                        continue
                    }
                };

                let page: Page = match serde_json::from_str(&data)
                {
                    Ok(d) => d,
                    Err(e) =>
                    {
                        crate::debug(format!("could not pass as a json page:\n {}\n {}", data, e), None);
                        continue
                    }
                };

                p.push(page);

            }

            if p.is_empty()
            {
                vec![Page::error("No pages to serve")]
            }
            else
            {
                p
            }
        },
        Err(e) =>
        {
            crate::debug(format!("Error reading pages {}", e), None);
            vec![Page::error("No pages to serve")]
        }
    }
}