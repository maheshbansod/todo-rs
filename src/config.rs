use std::{
    fmt::Display,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use getset::Getters;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};

const APP_NAME: &str = env!("CARGO_PKG_NAME");

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ListMetadata {
    name: String,
    path: PathBuf,
}

#[derive(Clone, Debug, Getters, Serialize, Deserialize)]
pub struct Config {
    /// all lists live in the main dir
    #[getset(get)]
    main_dir: PathBuf,
    #[getset(get)]
    /// Custom lists from around the OS
    #[serde(default = "Config::default_lists")]
    lists: Vec<ListMetadata>,
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
        dirs::config_dir()
            .expect("OS config directory not found")
            .join(APP_NAME)
    }

    fn default_list_directory_path() -> PathBuf {
        dirs::data_dir()
            .expect("OS shared data directory not found")
            .join(APP_NAME)
    }

    pub fn read_from(path: &Path) -> Result<Self> {
        let config_file = fs::read_to_string(path)
            .with_context(|| format!("Couldn't read the config at '{}'", &path.display()))?;

        serde_json::from_str(&config_file).context("Invalid config file")
    }

    fn default_lists() -> Vec<ListMetadata> {
        vec![]
    }
    fn default_general_list_name() -> String {
        "general".to_string()
    }

    /// Write's the config with all the default settings
    /// And prints information about it.
    pub fn write_default() -> Result<Self> {
        println!("Welcome to {} by @maheshbansod!", "todo".green());
        println!();
        println!(
            "Setting some defaults to your config at {:?}",
            Config::default_config_path()
        );
        let main_dir = Config::default_list_directory_path();
        println!("Setting the main_dir to {:?}. This is where any new lists you manually make will be stored.", main_dir);
        let general_list = Config::default_general_list_name();
        println!();
        println!("Setting the general list name to {}. This is like a default list. This list will be used for commands when there's no list in the current directory and no list is manually specified.", general_list);

        let optconfig = OptionalConfig {
            main_dir: PathBuf::from(main_dir),
            general_list: (!general_list.is_empty()).then_some(general_list.to_string()),
        };

        // write to the default config path
        let config_dir = Config::default_config_dir_path();
        fs::create_dir_all(config_dir).context("Creating config directory")?;
        let config_path = Config::default_config_path();
        fs::write(config_path, serde_json::to_string_pretty(&optconfig)?)?;

        println!();
        println!("All done!");
        println!();
        // re-read default and return it
        Config::read_from_default()
    }

    pub fn list_path(&self, name: &str) -> PathBuf {
        if let Some(l) = self.lists.iter().find(|i| i.name == name) {
            l.path.clone()
        } else {
            let mut list_path = self.main_dir.clone();
            list_path.push(format!("{}.md", name));
            list_path
        }
    }

    /// Lists existing lists
    pub fn existing_lists(&self) -> Result<Vec<String>> {
        let mut results = vec![];
        for l in self.lists.iter() {
            results.push(l.name.clone());
        }
        for entry in fs::read_dir(self.main_dir())? {
            let entry = entry?;
            results.push(entry.file_name().to_string_lossy().to_string());
        }
        Ok(results)
    }

    pub fn existing_lists_meta(&self) -> Result<Vec<ListMetadata>> {
        let mut results = vec![];
        for l in self.lists.iter() {
            results.push(l.clone());
        }
        for entry in fs::read_dir(self.main_dir())? {
            let entry = entry?;
            let list_name = entry.file_name().to_string_lossy().to_string();
            let list_name = &list_name[..list_name.len() - 3];
            results.push(ListMetadata {
                name: list_name.to_string(),
                path: PathBuf::from(format!(
                    "{}/{}.md",
                    self.main_dir().to_string_lossy(),
                    list_name
                )),
            });
        }
        Ok(results)
    }

    pub fn outside_list_exists(&self, list_name: &str) -> bool {
        self.lists.iter().any(|i| i.name == list_name)
    }

    /// Add a list
    pub fn add_list(&mut self, list_name: &str, list_path: &PathBuf) -> Result<()> {
        // let mut c = self.clone();
        self.lists.push(ListMetadata {
            name: list_name.to_string(),
            path: PathBuf::from(&list_path),
        });
        let config_dir = Config::default_config_path();
        fs::write(config_dir, serde_json::to_string_pretty(&self)?)?;
        Ok(())
    }
}

impl Display for ListMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.path.to_string_lossy())
    }
}
