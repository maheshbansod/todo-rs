# Todo: Your Life, Organized in Plain Text

Ditch bloated, proprietary todo apps! `todo` is a simple, powerful, and flexible command-line todo list manager that puts *you* in control.

## Why `todo`?

`todo` is built for developers and power users who value simplicity and control. Here's why it's different:

*   **Plain Text Markdown:**  No lock-in. Edit anywhere, backup easily, and track changes with Git.
*   **Command-Line Efficiency:**  Manage tasks lightning-fast, without leaving your terminal.
*   **Multiple Lists:** Organize by project, context, or anything else.
*   **Tagging:** Categorize with `#tags` that are highlighted in the CLI.
*   **Move Items:** Reorganize tasks by moving them between lists.
*   **Context-Aware:** Automatically detects `TODO.md` in your current directory. In fact, this is how I manage TODO items for this repository (and others)!
*   **Configurable:** Customize list locations and default list names.

## Install

```
cargo install --git https://github.com/maheshbansod/todo-rs.git
```

## Usage

```
todo add "Buy groceries #errands"
todo ls
todo done --item-numbers 1
todo mv --item-numbers 2 --to-list "projectx"
todo lists
```

I have aliased it to `t` simply and most subcommands have an alias, so to add an item it's just
```
t a "replace moment with datefns #chore"
```

## Configuration

`todo`'s configuration is in `~/.config/todo/config.json`. Customize `main_dir`, `general_list`, and custom `lists`. Configure interactively by running without a config file.

## Contributing

Contributions are welcome! Submit bug reports and feature requests as issues.

## License

[MIT](./LICENSE)
