use semver::{BuildMetadata, Prerelease, Version};

pub mod web;
pub mod server;
pub mod util;
pub mod pages;
pub mod resources;
pub mod config;

const MAJOR: &str = env!("CARGO_PKG_VERSION_MAJOR");
const MINOR: &str = env!("CARGO_PKG_VERSION_MINOR");
const PATCH: &str = env!("CARGO_PKG_VERSION_PATCH");

const RESOURCE_REGEX: &str = r"(\.\S+)";
const HTML_REGEX: &str = r"(\.html)$";
const NO_EXTENSION_REGEX: &str = r"^(?!.*\.).*";

const DEBUG: bool = true;

pub fn debug(msg: String, context: Option<String>)
{
    if DEBUG == false { return }

    let mut message = String::new();

    let time = chrono::offset::Utc::now().to_rfc3339();

    let tag = match context
    {
        Some(s) => format!("{time} [{s}] "),
        None => format!("{time} [DEBUG] ")
    };

    for line in msg.split("\n")
    {
        message.push_str(&tag);
        message.push_str(line);
        message.push_str("\n");
    }

    print!("{message}");

}

pub fn program_version() -> Version 
{
    Version
    {
        major: MAJOR.parse().unwrap(),
        minor: MINOR.parse().unwrap(),
        patch: PATCH.parse().unwrap(),
        pre: Prerelease::EMPTY,
        build: BuildMetadata::EMPTY
    }
}