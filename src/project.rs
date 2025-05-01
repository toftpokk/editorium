use std::{fmt, fs, io, path::PathBuf};

use iced::{
    Element,
    widget::{Column, button, text},
};

use crate::{Message, button_style};

#[derive(PartialEq, Clone)]
pub struct Project {
    pub name: String,
    pub path: PathBuf,
}

impl Project {
    pub fn new(file_path: PathBuf) -> Self {
        let path = match fs::canonicalize(&file_path) {
            Ok(ok) => ok,
            Err(err) => {
                log::error!("could not canonicalize path {:?}: {}", file_path, err);
                file_path
            }
        };

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
                .map(|node| match node {
                    Node::File { name, path } => button(text(name))
                        .on_press(Message::OpenFile(path.to_owned()))
                        .style(button_style)
                        .into(),
                    Node::Directory {
                        name,
                        path,
                        children,
                        open,
                    } => button(text(name))
                        .on_press(Message::OpenFile(path.to_owned()))
                        .style(button_style)
                        .into(),
                })
                .collect();
            return Column::from_vec(nodes).into();
        }
        return items.into();
    }

    pub fn open_project(&mut self, path: PathBuf) -> Result<(), io::Error> {
        let mut node = Node::new(path);
        if let Node::Directory {
            name: _,
            path: _,
            children: _,
            open: _,
        } = node
        {
            let c = node.load_children()?;
            node.set_children(c)?;
        }
        self.tree = Some(node);

        Ok(())
    }
}

enum Node {
    File {
        name: String,
        path: PathBuf,
    },
    Directory {
        name: String,
        path: PathBuf,
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
            Self::Directory {
                name: name,
                path: path,
                children: None,
                open: false,
            }
        } else {
            Self::File {
                name: name,
                path: path,
            }
        };
    }

    fn flatten(&self) -> Vec<&Node> {
        match self {
            Node::File { name, path } => {
                vec![self]
            }
            Node::Directory {
                name,
                path,
                children,
                open,
            } => {
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

    fn set_children(&mut self, children: Vec<Node>) -> Result<(), io::Error> {
        match self {
            Node::File { name, path } => panic!("not a directory"),
            Node::Directory {
                name,
                path,
                children: c,
                open: o,
            } => {
                *c = Some(children);
                *o = true;
                Ok(())
            }
        }
    }

    fn load_children(&self) -> Result<Vec<Node>, io::Error> {
        match self {
            Node::File { name: _, path: _ } => panic!("not a directory"),
            Node::Directory {
                name: _,
                path,
                children,
                open,
            } => {
                let read_dir = fs::read_dir(&path)?;
                let mut nodes = Vec::new();
                for dir_entry in read_dir {
                    let entry = dir_entry?;
                    nodes.push(Node::new(entry.path()));
                }
                Ok(nodes)
            }
        }
    }
}
