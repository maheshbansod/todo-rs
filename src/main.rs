use std::io::Write;
use std::{fs::OpenOptions, path::PathBuf};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use config::Config;
use todo::TodoList;

mod config;

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Perform actions on this list - general list is used if unspecified
    #[arg(short, long)]
    list: Option<String>, // TODO: implement some way to store list path in config so lists can be
    // refered by name here
    /// Optionally specify path to a configuration file.
    #[arg(short, long)]
    config: Option<PathBuf>,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Add {
        title: String,
    },
    List,
    /// mark an item done
    Done {
        #[arg(short, long)]
        item_number: usize,
    },
    /// delete an item
    Delete {
        #[arg(short, long)]
        item_number: usize,
    },
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

    // perform operation on this list
    let list_name = cli.list.unwrap_or(config.general_list().clone());
    let list_path = config.list_path(&list_name);

    match cli.command {
        Commands::Add { title } => {
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

            writeln!(file, "\n- [ ] {}", title).with_context(|| {
                format!("Couldn't write to the file '{}'", &list_path.display())
            })?;
        }
        Commands::List => {
            let list = TodoList::from_file(&list_path)?;
            println!("{}", list.display_with_numbers());
        }
        Commands::Done { item_number } => {
            let mut list = TodoList::from_file(&list_path)?;
            let item = list.mark_item_done(item_number)?.clone();

            list.write(&config.list_path(&list_name))
                .with_context(|| "Something went wrong. Couldn't write to the list.")?;

            println!("Marked item done.\n{item}");
        }
        Commands::Delete { item_number } => {
            let mut list = TodoList::from_file(&list_path)?;
            let item = list.delete_item(item_number)?;

            list.write(&list_path)
                .with_context(|| "Couldn't write to the list")?;

            println!("Deleted todo item\n{item}");
        }
    }
    Ok(())
}
