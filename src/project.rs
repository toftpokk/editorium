use std::{fmt, fs, io, path::PathBuf};

use iced::{
    Element,
    widget::{Column, button, text},
};

use crate::{Message, button_style};

#[derive()]
pub enum Error {
    NotADirectory,
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // write!(f, "invalid first item to double")
        match self {
            Error::NotADirectory => write!(f, "not a directory"),
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
    pub fn new(file_path: PathBuf) -> Self {
        let path = to_canonical(file_path);

        let name = path
            .file_name()
            .expect("invalid dir name")
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
    tree: Option<Node>,
}

impl ProjectTree {
    pub fn new() -> Self {
        Self { tree: None }
    }

    pub fn view(&self) -> Element<Message> {
        let items = Column::new();
        if let Some(tree) = &self.tree {
            let nodes = tree
                .flatten()
                .iter()
                .map(|&node| match &node.kind {
                    NodeKind::File => button(text(&node.name))
                        .on_press(Message::OpenFile(node.path.to_owned()))
                        .style(button_style)
                        .into(),
                    NodeKind::Directory { children, open } => button(text(&node.name))
                        .on_press(Message::ProjectTreeToggleDirectory(node.path.to_owned()))
                        .style(button_style)
                        .into(),
                })
                .collect();
            return Column::from_vec(nodes).into();
        }
        return items.into();
    }

    pub fn open_project(&mut self, path: PathBuf) -> Result<(), Error> {
        let path = to_canonical(path);

        let mut node = Node::new(path);
        if let NodeKind::Directory {
            children: _,
            open: _,
        } = node.kind
        {
            let c = node.load_children()?;
            node.set_children(c)?;
        }
        self.tree = Some(node);

        Ok(())
    }
}

struct Node {
    name: String,
    path: PathBuf,
    kind: NodeKind,
}

// https://stackoverflow.com/questions/49186751/sharing-a-common-value-in-all-enum-values
enum NodeKind {
    File,
    Directory {
        // for preloading
        children: Option<Vec<Node>>,
        // for ui
        open: bool,
    },
}

impl Node {
    fn new(path: PathBuf) -> Self {
        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        return if path.is_dir() {
            Self {
                name: name,
                path: path,
                kind: NodeKind::Directory {
                    children: None,
                    open: false,
                },
            }
        } else {
            Self {
                name: name,
                path: path,
                kind: NodeKind::File,
            }
        };
    }

    fn flatten(&self) -> Vec<&Node> {
        match &self.kind {
            NodeKind::File => {
                vec![self]
            }
            NodeKind::Directory { children, open } => {
                let result = vec![self];
                if !open {
                    return result;
                }
                match children {
                    None => panic!("children not loaded"),
                    Some(nodes) => nodes.iter().fold(result, |mut acc, child| {
                        acc.append(&mut child.flatten());
                        acc
                    }),
                }
            }
        }
    }

    fn set_children(&mut self, children: Vec<Node>) -> Result<(), Error> {
        if let NodeKind::Directory {
            children: ref mut c,
            open: ref mut o,
        } = self.kind
        {
            *c = Some(children);
            *o = true;
            return Ok(());
        }
        Err(Error::NotADirectory)
    }

    fn load_children(&self) -> Result<Vec<Node>, Error> {
        if let NodeKind::Directory {
            children: _,
            open: _,
        } = &self.kind
        {
            let read_dir = fs::read_dir(&self.path)?;
            let mut nodes = Vec::new();
            for dir_entry in read_dir {
                let entry = dir_entry?;
                nodes.push(Node::new(entry.path()));
            }
            return Ok(nodes);
        }
        Err(Error::NotADirectory)
    }
}

fn to_canonical(path: PathBuf) -> PathBuf {
    fs::canonicalize(&path).expect("could not canonicalize")
}
