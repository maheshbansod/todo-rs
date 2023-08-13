use std::io::Write;
use std::{
    fs::{self, OpenOptions},
    path::PathBuf,
};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use config::Config;
use todo::TodoList;

mod config;

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Optionally specify path to a configuration file.
    #[arg(short, long)]
    config: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Add { title: String },
    List { name: Option<String> },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    let config = if let Some(config_path) = cli.config {
        Config::read_from(&config_path)?
    } else if let Ok(config) = Config::read_from_default() {
        config
    } else {
        println!(
            "Looked for the config file at '{}'",
            Config::default_config_path().display()
        );
        println!("It either does not exist or is invalid.");
        println!("You can stop the application now or you can respond to the following questions to create a new config file.");
        Config::read_interactive()?
    };

    match cli.command {
        Commands::Add { title } => {
            let list_path = config.general_list_path();

            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&list_path)
                .with_context(|| {
                    format!(
                        "Couldn't open file '{}' to write to the list",
                        &list_path.display()
                    )
                })?;

            writeln!(file, "- [ ] {}", title).with_context(|| {
                format!("Couldn't write to the file '{}'", &list_path.display())
            })?;
        }
        Commands::List { name } => {
            let name = name.unwrap_or(config.general_list().clone());

            let file = fs::read_to_string(config.list_path(&name))
                .expect("Can't read the list. Are you sure it exists?");

            let list: TodoList = file.trim().parse()?;
            println!("{}", list.display_with_numbers());
        }
    }
    Ok(())
}
