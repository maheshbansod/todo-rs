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

### Sample workflow

I have aliased it to `t` simply and most subcommands have an alias, so to add an item it's just
```
t a "replace moment with datefns #chore"
```
List down all incomplete tasks:
```
t
```
it outputs:
```
  9  ⬜ add summary command to list a summary of existing list and some undone tasks
 11  ⬜ ability add tags to each todo item
 12  ⬜ a way to set time to finish
```
Mark a few tasks as done:
```
t d -i 9 11
```
It outputs:
```
Marked item(s) done.
 ✅ add summary command to list a summary of existing list and some undone tasks
 ✅ ability add tags to each todo item
```

List all the lists tracked by `todo` so far along with the location of the list:
```
t lists --show-paths
```
It'll output:
```
todo: /home/light/projects/todo/TODO.md
ai.nvim: /home/light/projects/ai.nvim/TODO.md
general: /home/light/vaults/Main/todos/general.md
```
List all tasks including done tasks for a specific list
```
t -l ai.nvim ls --all
```
Outputs:
```
#### Chat stuff

 18  ✅ Chat window for continous chats
 19  ⬜ Add context with @<some code symbol>.
 20  ⬜ RAG on the current file (if the file is big enough)
 21  ⬜ Allow adding current file as context.
 22  ⬜ Different colors for AI message and user message maybe.
 23  ⬜ Auto scroll down to user prompt when AI response appears probably?
 24  ⬜ Stream AI response + autoscroll while streaming.
    - I want to do this in a non-invasive way
 26  ⬜ Apply code block
    - without context - search in the file where to put the code first.
    - with context it can directly select what parts of the context to replace.

#### Quick suggestion

 32  ⬜ Better support for adding to the current line
    - more context like language / framework / locals / etc
 34  ⬜ Highlight generated text for maybe 1 second.
```
(output trimmed -> check out [ai.nvim](https://github.com/maheshbansod/ai.nvim), your AI within Neovim!)


## Configuration

`todo`'s configuration is in `~/.config/todo/config.json`. Customize `main_dir`, `general_list`, and custom `lists`.

Running it for the first time will auto-generate your config with defaults.

## Contributing

Contributions are welcome! Submit bug reports and feature requests as issues.

## License

[MIT](./LICENSE)
