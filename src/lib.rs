use std::{
    fmt::{Debug, Display},
    fs, io,
    path::Path,
    str::FromStr,
};

use owo_colors::{colors, OwoColorize};
use thiserror::Error;

pub struct TodoList {
    pub name: String,
    list: Vec<TodoListFileItem>,
}

impl TodoList {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            list: vec![],
        }
    }

    pub fn from_file(path: &Path) -> Result<Self, TodoError> {
        let name = path.file_name().unwrap();
        let file_contents = fs::read_to_string(path)?;
        let list = TodoList::list_from_str(&file_contents)?;
        Ok(Self {
            name: name.to_string_lossy().to_string(),
            list,
        })
    }

    fn list_from_str(s: &str) -> Result<Vec<TodoListFileItem>, TodoError> {
        // todo: maybe try nom or smn
        let lines = s.lines();
        let mut list: Vec<TodoListFileItem> = vec![];
        for line in lines {
            if line.starts_with("- [") {
                let item: Result<TodoItem, _> = line.parse();
                if let Ok(item) = item {
                    list.push(TodoListFileItem::TodoItem(item));
                } else {
                    list.push(TodoListFileItem::String(line.to_string()));
                }
            } else {
                list.push(TodoListFileItem::String(line.to_string()));
            }
        }
        Ok(list)
    }

    pub fn display_with_numbers<P>(&self, predicate: P) -> String
    where
        P: Fn(&(usize, &TodoItem)) -> bool,
    {
        self.list
            .iter()
            .enumerate()
            .filter(|(i, list_file_item)| {
                if let TodoListFileItem::TodoItem(todo_item) = list_file_item {
                    let a = (*i, todo_item);
                    predicate(&a)
                } else {
                    true
                }
            })
            .map(|(i, item)| match item {
                TodoListFileItem::TodoItem(item) => format!("{: >3} {item}", i + 1),
                TodoListFileItem::String(s) => s.to_string(),
            }) // padding will be good till 3
            // digits - todo: check how we can remove this limit
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub fn as_markdown(&self) -> String {
        self.list
            .iter()
            .map(|i| match i {
                TodoListFileItem::TodoItem(i) => {
                    format!("- [{}] {}", i.state.as_markdown(), i.name)
                }
                TodoListFileItem::String(s) => s.to_string(),
            })
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub fn get_item_mut(&mut self, item_number: usize) -> Result<&mut TodoItem, TodoError> {
        self.list
            .get_mut(item_number - 1)
            .ok_or(TodoError::InvalidItemNumber(item_number))
            .and_then(|item| {
                if let TodoListFileItem::TodoItem(todo_item) = item {
                    Ok(todo_item)
                } else {
                    Err(TodoError::InvalidItemNumber(item_number))
                }
            })
    }
    pub fn get_item(&self, item_number: usize) -> Result<&TodoItem, TodoError> {
        self.list
            .get(item_number - 1)
            .ok_or(TodoError::InvalidItemNumber(item_number))
            .and_then(|item| {
                if let TodoListFileItem::TodoItem(todo_item) = item {
                    Ok(todo_item)
                } else {
                    Err(TodoError::InvalidItemNumber(item_number))
                }
            })
    }

    pub fn mark_item_done(&mut self, item_number: usize) -> Result<&TodoItem, TodoError> {
        let item = self.get_item_mut(item_number)?;
        item.mark_done();
        Ok(item)
    }

    pub fn add_item(&mut self, item_title: &str) {
        let item = TodoItem {
            name: item_title.to_string(),
            description: None,
            state: TodoItemState::Initial,
        };
        self.list.push(TodoListFileItem::TodoItem(item));
    }

    pub fn delete_items(&mut self, item_numbers: &[usize]) -> Result<Vec<TodoItem>, TodoError> {
        let mut sorted_indices = item_numbers.to_vec();
        sorted_indices.sort_unstable_by(|a, b| b.cmp(a));

        let mut removed = vec![];
        for &index in &sorted_indices {
            if index > 0 && index <= self.list.len() {
                let todo_item = self.get_item(index)?.clone();
                removed.push(todo_item);
                self.list.remove(index - 1);
            } else {
                return Err(TodoError::InvalidItemNumber(index));
            }
        }
        Ok(removed)
    }

    pub fn add_items(&mut self, items: Vec<TodoItem>) {
        let converted_items: Vec<TodoListFileItem> =
            items.into_iter().map(TodoListFileItem::TodoItem).collect();

        self.list.extend(converted_items);
    }

    pub fn write(&self, path: &Path) -> Result<(), TodoError> {
        Ok(fs::write(path, self.as_markdown())?)
    }
}

#[derive(PartialEq, Clone)]
pub enum TodoItemState {
    Done,
    Initial,
}

impl TodoItemState {
    pub fn as_markdown(&self) -> String {
        match self {
            TodoItemState::Done => "x".to_string(),
            TodoItemState::Initial => " ".to_string(),
        }
    }
}

pub enum TodoListFileItem {
    TodoItem(TodoItem),
    /// I put anything random as a string in this.
    /// Probably in the future I will also parse headings separately
    /// giving the users ability to add an item to a specific heading
    String(String),
}

#[derive(Clone)]
pub struct TodoItem {
    pub name: String,
    pub description: Option<String>,
    pub state: TodoItemState,
}

impl TodoItem {
    pub fn mark_done(&mut self) {
        self.state = TodoItemState::Done;
    }

    pub fn is_done(&self) -> bool {
        self.state == TodoItemState::Done
    }
}

impl Display for TodoList {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{}",
            self.list
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<String>>()
                .join("\n")
        )
    }
}

fn render_tag(s: &str) -> String {
    format!(" #{} ", s)
        .bg::<colors::Yellow>()
        .fg::<colors::Black>()
        .to_string()
}

/// Looks for words that start with # and colors them differently - maybe i will make this
/// configurable
fn color_tags(s: &str) -> String {
    let mut split = s.split("#");
    let first = split.next().expect("one elem should be present");
    let rest = split
        .map(|s| {
            if s.starts_with(|c: char| c.is_whitespace()) {
                s.to_string()
            } else if let Some((first_word, rest)) = s.split_once(|c: char| c.is_whitespace()) {
                // todo: seems i'm converting any first whitespace to space character -> it should probably
                // preserve whitespace
                let first_word = render_tag(first_word);
                format!("{} {}", first_word, rest)
            } else {
                render_tag(s).to_string()
            }
        })
        .collect::<String>();
    if rest.is_empty() {
        first.to_string()
    } else {
        format!("{first}{rest}")
    }
}

impl Display for TodoListFileItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(s) => write!(f, "{s}"),
            Self::TodoItem(t) => write!(f, "{t}"),
        }
    }
}

impl Display for TodoItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            " {} {}{}",
            self.state,
            if self.is_done() {
                format!("{}", color_tags(&self.name).strikethrough())
            } else {
                color_tags(&self.name).to_string()
            },
            if let Some(desc) = &self.description {
                format!("\n{}", color_tags(desc))
            } else {
                "".to_string()
            }
        )
    }
}

impl Display for TodoItemState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TodoItemState::Done => write!(f, "{}", "✅".green()),
            TodoItemState::Initial => write!(f, "{}", "⬜".yellow()),
        }
    }
}

impl FromStr for TodoItem {
    type Err = TodoError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("- [") {
            return Err(TodoError::ParseError(format!(
                "Item should start with the check box\nFound: '{s}'"
            )));
        };
        let mut chars = s.chars().skip(3); // skip first 3 characters ("- [")
        let mark = chars
            .next()
            .ok_or_else(|| {
                TodoError::ParseError(format!(
                    "Item should start with the check box. No mark.\nFound: '{s}'"
                ))
            })?
            .to_string();
        chars
            .next()
            .ok_or_else(|| TodoError::ParseError(format!(
                "Item should start with the check box. No closing bracket.\nFound: '{s}'"
            )))?
            .eq(&']')
            .then(|| chars.next())
            .ok_or_else(|| TodoError::ParseError(format!(
                "Item should start with the check box. Expected ']'\nFound: '{s}'"
            )))?
            .ok_or_else(|| TodoError::ParseError(format!(
                "Item should start with the check box. Item ended unexpectedly. Space expected after ']'.\nFound: '{s}'"
            )))?
            .eq(&' ')
            .then_some(())
            .ok_or_else(|| TodoError::ParseError(format!(
                "Item should start with the check box. Space expected after ']'.\nFound: '{s}'"
            )))?;
        let name = chars.collect::<String>();
        (!name.is_empty()).then_some(()).ok_or_else(|| {
            TodoError::ParseError(format!("Item name can't be empty.\nFound: '{s}'"))
        })?;

        Ok(Self {
            name,
            state: mark.parse()?,
            description: None,
        })
    }
}

impl FromStr for TodoItemState {
    type Err = TodoError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "x" => Ok(TodoItemState::Done),
            " " => Ok(TodoItemState::Initial),
            _ => Err(TodoError::ParseError(format!(
                "This state of a todo item is not supported.\nFound: '{s}'"
            ))),
        }
    }
}

#[derive(Error, Debug)]
pub enum TodoError {
    #[error("Parsing error. {0}")]
    ParseError(String),
    #[error("Invalid item number. The item number {0} doesn't exist in the list")]
    InvalidItemNumber(usize),
    #[error("IO Error. {0}")]
    FileIOError(#[from] io::Error),
}
