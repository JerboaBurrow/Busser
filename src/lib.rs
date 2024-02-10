use semver::{BuildMetadata, Prerelease, Version};

pub mod web;
pub mod server;
pub mod util;
pub mod pages;

const MAJOR: &str = env!("CARGO_PKG_VERSION_MAJOR");
const MINOR: &str = env!("CARGO_PKG_VERSION_MINOR");
const PATCH: &str = env!("CARGO_PKG_VERSION_PATCH");

const DEBUG: bool = true;

pub fn debug(msg: String, context: Option<String>)
{
    if DEBUG == false { return }

    match context
    {
        Some(s) => println!("[DEBUG] {msg} in context {s}"),
        None => println!("[DEBUG] {msg}")
    }
    
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