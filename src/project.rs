use std::{
    cmp::min,
    collections::HashMap,
    fmt, fs, io,
    iter::Map,
    ops::{Index, Mul},
    path::PathBuf,
};

use iced::{
    Color, Element, Padding, Theme,
    mouse::Button,
    widget::{Column, button, text},
};

use crate::Message;

#[derive()]
pub enum Error {
    Io(io::Error),
}

// Inspired by cosmic-edit

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Io(error) => write!(f, "io error: {}", error),
        }
    }
}

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        return Error::Io(value);
    }
}

#[derive(PartialEq, Clone)]
pub struct Project {
    pub name: String,
    pub path: PathBuf,
}

impl Project {
    pub fn new(path: PathBuf) -> Self {
        let name = path
            .file_name()
            .expect(format!("invalid directory name {:?}", path).as_str())
            .to_str()
            .expect("invalid dir name string");
        Self {
            name: name.to_string(),
            path: path,
        }
    }
}

impl fmt::Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

pub struct ProjectTree {
    order: Vec<usize>,
    items: HashMap<usize, Node>,
    next_id: usize,
}

impl ProjectTree {
    pub fn new() -> Self {
        Self {
            order: Vec::new(),
            items: HashMap::new(),
            next_id: 0,
        }
    }

    pub fn clear(&mut self) {
        self.order.clear();
        self.items.clear();
        self.next_id = 0;
    }

    pub fn insert(&mut self, path: PathBuf, pos: usize, indent: usize) {
        let pos = min(pos, self.order.len());
        self.order.insert(pos, self.next_id);
        self.items
            .insert(self.next_id, Node::new(path, self.next_id, indent));
        self.next_id += 1;
    }

    pub fn remove(&mut self, id: usize) {
        self.items.remove(&id);
        if let Some(pos) = self.position(id) {
            self.order.remove(pos);
        }
    }

    pub fn node_mut(&mut self, id: usize) -> Option<&mut Node> {
        self.items.get_mut(&id)
    }

    pub fn node_at(&self, pos: usize) -> Option<&Node> {
        let id = self.order.get(pos)?;
        self.items.get(id)
    }
    pub fn position(&self, id: usize) -> Option<usize> {
        return self.order.iter().position(|x| *x == id);
    }

    pub fn view(&self) -> Column<Message> {
        let nodes: Vec<Element<Message>> = self
            .order
            .iter()
            .map(|id| {
                let node = self.items.get(id).unwrap();
                match &node.kind {
                    NodeKind::File => button(text(&node.name))
                        .on_press(Message::OpenFile(node.path.to_owned()))
                        .padding(Padding {
                            top: 0.0,
                            right: 0.0,
                            bottom: 0.0,
                            left: node.indent as f32 * 10.0,
                        })
                        .style(button_style)
                        .into(),
                    NodeKind::Directory { open } => button(text(&node.name))
                        .on_press(Message::ProjectTreeSelect(node.id))
                        .padding(Padding {
                            top: 0.0,
                            right: 0.0,
                            bottom: 0.0,
                            left: node.indent as f32 * 10.0,
                        })
                        .style(button_style)
                        .into(),
                }
            })
            .collect();
        Column::from_vec(nodes)
    }
}

#[derive(Clone)]
pub struct Node {
    pub id: usize,
    name: String,
    pub indent: usize,
    pub path: PathBuf,
    pub kind: NodeKind,
}

// https://stackoverflow.com/questions/49186751/sharing-a-common-value-in-all-enum-values
#[derive(Clone, Copy)]
pub enum NodeKind {
    File,
    Directory { open: bool },
}

impl Node {
    fn new(path: PathBuf, id: usize, indent: usize) -> Self {
        let name = path.file_name().unwrap().to_str().unwrap().to_string();

        return if path.is_dir() {
            Self {
                id: id,
                name: name,
                path: path,
                indent: indent,
                kind: NodeKind::Directory { open: false },
            }
        } else {
            Self {
                id: id,
                name: name,
                path: path,
                indent: indent,
                kind: NodeKind::File,
            }
        };
    }
}

fn button_style(_theme: &Theme, status: button::Status) -> button::Style
where
    Theme: button::Catalog,
{
    match status {
        button::Status::Active => button::Style {
            background: Some(iced::Background::Color(Color::from_rgb8(0x2B, 0x2D, 0x30))),
            text_color: Color::from_rgb8(0xDF, 0xE1, 0xE5),
            ..Default::default()
        },
        button::Status::Disabled => button::Style {
            background: Some(iced::Background::Color(Color::from_rgb8(0x2B, 0x2D, 0x30))),
            text_color: Color::from_rgb8(0xDF, 0xE1, 0xE5),
            ..Default::default()
        },
        button::Status::Hovered => button::Style {
            background: Some(iced::Background::Color(Color::from_rgb8(0x2B, 0x2D, 0x30))),
            text_color: Color::from_rgb8(0xDF, 0xE1, 0xE5),
            ..Default::default()
        },
        button::Status::Pressed => button::Style {
            background: Some(iced::Background::Color(Color::from_rgb8(0x2D, 0x43, 0x6E))),
            text_color: Color::from_rgb8(0xDF, 0xE1, 0xE5),
            ..Default::default()
        },
    }
}
