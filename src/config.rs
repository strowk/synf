use std::path::Path;

use serde::Deserialize;
use toml;

#[derive(Deserialize,Debug,PartialEq,Clone)]
pub (crate) enum Language {
    #[serde(rename = "typescript")]
    Typescript,
    #[serde(rename = "python")]
    Python,
    #[serde(rename = "golang")]
    Golang,
}

#[derive(Deserialize)]
pub(crate) struct Config {
    pub(crate) language: Language,
    pub (crate) watch: Option<Watch>,
}

#[derive(Deserialize)]
pub (crate) struct Watch {
    pub(crate) default_watch_paths: Option<Vec<String>>,
    pub (crate) extra_watch_paths: Option<Vec<String>>,
}

pub(crate) fn read_from_toml(path: &Path) -> Result<Config, eyre::Report> {
    let path = path.join("synf.toml");
    let toml_str = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&toml_str)?;
    Ok(config)
}
