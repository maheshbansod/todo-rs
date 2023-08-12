use std::{fmt::{Display, Debug}, str::FromStr};

use thiserror::Error;

pub struct TodoList {
    list: Vec<TodoItem>,
}

#[derive(PartialEq, Clone)]
pub enum TodoItemState {
    Done,
    Initial,
}

#[derive(Clone)]
pub struct TodoItem {
    pub name: String,
    pub description: Option<String>,
    pub state: TodoItemState,
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
            "- [{}] {}{}",
            if self.state == TodoItemState::Done { 'x' } else { ' ' },
            self.name,
            if let Some(desc) = &self.description { format!("\n{desc}") } else { "".to_string() }
        )
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
}
