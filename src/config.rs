use std::{
    fs, io,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use getset::Getters;
use serde::{Deserialize, Serialize};

#[derive(Debug, Getters, Deserialize)]
pub struct Config {
    /// all lists live in the main dir
    #[getset(get)]
    main_dir: PathBuf,
    /// general list - random items with no list specified will be in this list
    #[serde(default = "Config::default_general_list_name")]
    #[getset(get = "pub")]
    general_list: String,
}

#[derive(Serialize)]
struct OptionalConfig {
    main_dir: PathBuf,
    #[serde(skip_serializing_if = "Option::is_none")]
    general_list: Option<String>,
}

impl Config {
    pub fn read_from_default() -> Result<Self> {
        let config_file = Config::default_config_path();

        Config::read_from(&config_file)
    }

    pub fn default_config_path() -> PathBuf {
        let config_dir = Config::default_config_dir_path();

        config_dir.join("config.json")
    }

    fn default_config_dir_path() -> PathBuf {
        const APP_NAME: &str = env!("CARGO_PKG_NAME");
        dirs::config_dir()
            .expect("OS config directory not found")
            .join(APP_NAME)
    }

    pub fn read_from(path: &Path) -> Result<Self> {
        let config_file = fs::read_to_string(path)
            .with_context(|| format!("Couldn't read the config at '{}'", &path.display()))?;

        serde_json::from_str(&config_file).context("Invalid config file")
    }

    fn default_general_list_name() -> String {
        "general".to_string()
    }

    /// Prompts the user for the config
    pub fn read_interactive() -> Result<Self> {
        let main_dir = Config::prompt("Where should the todo lists be located?", None)?;
        let general_list = Config::prompt(
            "What should the general list be called?",
            Some(Config::default_general_list_name().as_str()),
        )?;

        let optconfig = OptionalConfig {
            main_dir: PathBuf::from(main_dir),
            general_list: (!general_list.is_empty()).then_some(general_list),
        };

        // write to the default config path
        let config_dir = Config::default_config_dir_path();
        fs::create_dir_all(config_dir).context("Creating config directory")?;
        let config_path = Config::default_config_path();
        fs::write(config_path, serde_json::to_string_pretty(&optconfig)?)?;

        // re-read default and return it
        Config::read_from_default()
    }

    fn prompt(prompt: &str, default: Option<&str>) -> Result<String> {
        println!("> {}", prompt);
        if let Some(default) = default {
            println!("(default: {default})");
        }
        let mut data = String::new();
        let stdin = io::stdin();
        stdin
            .read_line(&mut data)
            .context("Failed to read user input")?;
        Ok(data.trim().to_owned())
    }

    pub fn list_path(&self, name: &str) -> PathBuf {
        let mut list_path = self.main_dir.clone();
        list_path.push(format!("{}.md", name));
        list_path
    }
}
