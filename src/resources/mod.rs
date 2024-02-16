pub mod resource;

use regex::Regex;

use crate::{util::{list_dir_by, list_sub_dirs, read_file_bytes}, HTML_REGEX, RESOURCE_REGEX};

use self::resource::{content_type, Resource};

/// Scan the path (if None the current dir) for non .html resources
/// 
/// # Example
/// ```rust
/// // with files resources/index.html, resources/animation.js
/// 
/// use busser::resources::{get_resources, resource::Resource};
/// 
/// pub fn main()
/// {
///     let resources = get_resources(Some("resources"));
/// 
///     // assert_eq!(resources.len(), 1);
///     // assert!(!resources.contains(&Resource::new("resources/index.html", "")));
///     // assert!(resources.contains(&Resource::new("resources/animation.js", "")));
/// }
/// ``` 
pub fn get_resources(path: Option<&str>) -> Vec<Resource>
{
    let scan_path = match path
    {
        Some(s) => s,
        None => ""
    };

    let resource_regex = Regex::new(RESOURCE_REGEX).unwrap();
    let html_regex = Regex::new(HTML_REGEX).unwrap();

    let resource_paths = list_dir_by(resource_regex, scan_path.to_string());
    let mut resources: Vec<Resource> = vec![];

    for resource_path in resource_paths
    {
        match html_regex.find_iter(resource_path.as_str()).count()
        {
            0 => {},
            _ => {continue}
        }

        let data = match read_file_bytes(&resource_path)
        {
            Some(data) => data,
            None => continue
        };

        resources.push(Resource::new(resource_path.as_str(), data, content_type(resource_path.to_string())));
    }

    let dirs = list_sub_dirs(scan_path.to_string());

    if !dirs.is_empty()
    {
        for dir in dirs
        {
            for resource in get_resources(Some(&dir))
            {
                resources.push(resource);
            }
        }
    }

    resources

}