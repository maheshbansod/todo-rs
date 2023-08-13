use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use config::Config;
use todo::{TodoError, TodoList};

mod config;

#[derive(Parser, Debug)]
#[command(author,version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

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
    /// Add an item
    #[command(alias="a")]
    Add {
        title: String,
    },
    /// List items
    #[command(alias="ls")]
    List,
    /// Mark items done
    #[command(alias="d")]
    Done {
        /// Item numbers to mark
        #[arg(short, long, num_args(1..))]
        item_numbers: Vec<usize>,
    },
    /// Delete items
    #[command(alias="rm")]
    Delete {
        /// Item numbers to delete
        #[arg(short, long, num_args(1..))]
        item_numbers: Vec<usize>,
    },
    /// move items to another list
    #[command(alias="mv")]
    Move {
        /// Item numbers to move
        #[arg(short, long, num_args(1..))]
        item_numbers: Vec<usize>,
        /// Destination list
        #[arg(short, long)]
        to_list: String,
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

    // list is the default command
    let command = cli.command.unwrap_or(Commands::List);

    // perform operation on this list
    let list_name = cli.list.unwrap_or(config.general_list().clone());
    let list_path = config.list_path(&list_name);

    match command {
        Commands::Add { title } => {
            let mut list = match TodoList::from_file(&list_path) {
                Ok(list) => list,
                Err(TodoError::FileIOError(_)) => TodoList::new(&list_name),
                Err(e) => return Err(e.into()),
            };
            list.add_item(&title);
            list.write(&list_path)
                .with_context(|| "Couldn't write the list")?;
        }
        Commands::List => {
            let list = TodoList::from_file(&list_path)?;
            println!("{}", list.display_with_numbers());
        }
        Commands::Done { item_numbers } => {
            let done_items = {
                let mut list = TodoList::from_file(&list_path)?;
                let done_items = item_numbers
                    .iter()
                    .map(|item_number| list.mark_item_done(*item_number).map(|i| i.clone()))
                    .collect::<Result<Vec<_>, _>>()?;
                list.write(&config.list_path(&list_name))
                    .with_context(|| "Something went wrong. Couldn't write to the list.")?;
                done_items
            };

            println!(
                "Marked item(s) done.\n{}",
                done_items
                    .iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<String>>()
                    .join("\n")
            );
        }
        Commands::Delete { item_numbers } => {
            let mut list = TodoList::from_file(&list_path)?;
            let removed_items = list.delete_items(item_numbers)?;

            list.write(&list_path)
                .with_context(|| "Couldn't write to the list")?;

            println!(
                "Deleted todo item(s)\n{}",
                removed_items
                    .iter()
                    .map(|i| i.to_string())
                    .collect::<Vec<_>>()
                    .join("\n")
            );
        }
        Commands::Move {
            item_numbers,
            to_list,
        } => {
            let mut from_list = TodoList::from_file(&list_path)?;
            let to_list_path = config.list_path(&to_list);
            let mut to_list = TodoList::from_file(&to_list_path)?;
            let removed_items = from_list.delete_items(item_numbers)?;
            to_list.add_items(removed_items);

            to_list.write(&to_list_path).with_context(|| {
                "Couldn't write to destination list. Items not added or removed"
            })?;
            from_list.write(&list_path).with_context(|| "Couldn't write to source list. Items not removed from source list but written to destination list.")?;
        }
    }
    Ok(())
}
