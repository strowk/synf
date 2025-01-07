use std::{
    fmt::{Display, Formatter},
    path::Path,
};

use serde::Deserialize;
use toml;

#[derive(Deserialize, Debug, PartialEq, Clone)]
pub(crate) enum Language {
    #[serde(rename = "typescript")]
    Typescript,
    #[serde(rename = "python")]
    Python,
    #[serde(rename = "golang")]
    Golang,
    #[serde(rename = "kotlin")]
    Kotlin,
}

impl Display for Language {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Language::Typescript => write!(f, "typescript"),
            Language::Python => write!(f, "python"),
            Language::Golang => write!(f, "golang"),
            Language::Kotlin => write!(f, "kotlin"),
        }
    }
}

#[derive(Deserialize)]
pub(crate) struct Config {
    pub(crate) language: Language,
    pub(crate) watch: Option<Watch>,
    pub(crate) resend_resource_subscriptions: Option<bool>,
    pub(crate) build: Option<BuildConfig>,
}

#[derive(Deserialize)]

pub(crate) struct BuildConfig {
    pub(crate) command: Option<String>,
    pub(crate) args: Option<Vec<String>>,
}

#[derive(Deserialize)]
pub(crate) struct Watch {
    pub(crate) default_watch_paths: Option<Vec<String>>,
    pub(crate) extra_watch_paths: Option<Vec<String>>,
}

pub(crate) fn read_from_toml(path: &Path) -> Result<Config, eyre::Report> {
    let path = path.join("synf.toml");
    let toml_str = std::fs::read_to_string(path)?;
    let config: Config = toml::from_str(&toml_str)?;
    Ok(config)
}
