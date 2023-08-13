use std::{
    fmt::{Debug, Display},
    fs, io,
    path::Path,
    str::FromStr,
};

use thiserror::Error;

pub struct TodoList {
    pub name: String,
    list: Vec<TodoItem>,
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

    fn list_from_str(s: &str) -> Result<Vec<TodoItem>, TodoError> {
        // todo: maybe try nom or smn
        let lines = s.lines();
        let mut list: Vec<TodoItem> = vec![];
        for line in lines {
            let item: Result<TodoItem, _> = line.parse();
            if let Err(err) = item {
                // concat to last's desciption if invalid todo item
                if let Some(last) = list.last_mut() {
                    if let Some(desc) = &last.description {
                        last.description = Some(format!("{}\n{}", desc, line));
                    } else {
                        last.description = Some(line.to_string());
                    }
                } else {
                    return Err(err);
                }
            } else {
                list.push(item.unwrap());
            }
        }
        Ok(list)
    }

    pub fn display_with_numbers(&self) -> String {
        self.list
            .iter()
            .enumerate()
            .map(|(i, item)| format!("{: >3} {item}", i + 1)) // padding will be good till 3
            // digits - todo: check how we can remove this limit
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub fn as_markdown(&self) -> String {
        self.list
            .iter()
            .map(|i| format!("- [{}] {}", i.state.as_markdown(), i.name))
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub fn get_item_mut(&mut self, item_number: usize) -> Result<&mut TodoItem, TodoError> {
        self.list
            .get_mut(item_number - 1)
            .ok_or_else(|| TodoError::InvalidItemNumber(item_number))
    }
    pub fn get_item(&self, item_number: usize) -> Result<&TodoItem, TodoError> {
        self.list
            .get(item_number - 1)
            .ok_or_else(|| TodoError::InvalidItemNumber(item_number))
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
        self.list.push(item);
    }

    pub fn delete_items(&mut self, item_numbers: Vec<usize>) -> Result<Vec<TodoItem>, TodoError> {
        let items_to_remove = item_numbers
            .iter()
            .map(|&i| self.get_item(i).cloned())
            .collect::<Result<Vec<_>, _>>()?;
        // this implementation will remove items with the same name - is a fix to this needed?
        self.list
            .retain(|i| items_to_remove.iter().find(|r| r.name == i.name).is_none());
        Ok(items_to_remove)
    }

    pub fn add_items(&mut self, mut items: Vec<TodoItem>) {
        self.list.append(&mut items);
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

impl Display for TodoItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            " {} {}{}",
            self.state,
            self.name,
            if let Some(desc) = &self.description {
                format!("\n{desc}")
            } else {
                "".to_string()
            }
        )
    }
}

impl Display for TodoItemState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TodoItemState::Done => write!(f, "✅"),
            TodoItemState::Initial => write!(f, "⬜"),
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
