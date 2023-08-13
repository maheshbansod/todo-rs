use std::{fmt::{Display, Debug}, str::FromStr, path::Path, fs, io};

use thiserror::Error;

pub struct TodoList {
    list: Vec<TodoItem>,
}

impl TodoList {
    pub fn display_with_numbers(&self) -> String {
        format!(
            "{}",
            self.list
                .iter()
                .enumerate()
                .map(|(i, item)| format!("{: >3} {item}", i+1)) // padding will be good till 3
                // digits - todo: check how we can remove this limit
                .collect::<Vec<String>>()
                .join("\n")
        )
    }

    pub fn as_markdown(&self) -> String {
        self.list.iter().map(|i| format!("- [{}] {}", i.state.as_markdown(), i.name)).collect::<Vec<String>>().join("\n")
    }

    pub fn get_item_mut(&mut self, item_number: usize) -> Result<&mut TodoItem, TodoError> {
        self.list.get_mut(item_number - 1).ok_or_else(|| TodoError::InvalidItemNumber)
    }

    pub fn mark_item_done(&mut self, item_number: usize) -> Result<&TodoItem, TodoError> {
        let item = self.get_item_mut(item_number)?;
        item.mark_done();
        Ok(item)
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
            TodoItemState::Initial => " ".to_string()
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
            if let Some(desc) = &self.description { format!("\n{desc}") } else { "".to_string() }
        )
    }
}

impl Display for TodoItemState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TodoItemState::Done => write!(f, "✅"),
            TodoItemState::Initial => write!(f, "⬜")
        }
    }
}

impl FromStr for TodoList {
    type Err = TodoError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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
        Ok(Self { list })
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
    #[error("Invalid item number. This item doesn't exist in the list")]
    InvalidItemNumber,
    #[error("Error writing. {0}")]
    FileWriteError (#[from] io::Error)
}
