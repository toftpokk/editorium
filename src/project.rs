use std::{
    fmt::{self, write},
    fs, io,
    path::PathBuf,
};

use iced::{
    Element,
    mouse::Button,
    widget::{Column, button, text},
};

use crate::{Message, button_style};

#[derive()]
pub enum Error {
    NodeNotADirectory,
    IDNotFound,
    Io(io::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // write!(f, "invalid first item to double")
        match self {
            Error::NodeNotADirectory => write!(f, "not a directory"),
            Error::IDNotFound => write!(f, "node not found for id"),
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
    tree: Vec<Node>,
    next_id: usize,
}

impl ProjectTree {
    pub fn new(path: PathBuf) -> Result<Self, Error> {
        let path = to_canonical(path);

        let root = Node::new(path, 0, None);
        let mut next_id = 1;
        let mut tree = Vec::new();
        if let NodeKind::Directory { open: _ } = root.kind {
            let (mut children, id) = root.load_children(next_id)?;
            next_id = id;
            tree.push(root);
            tree.append(&mut children);
        } else {
            tree.push(root);
        }

        Ok(Self {
            tree: tree,
            next_id: next_id,
        })
    }

    pub fn view(&self) -> Column<Message> {
        let nodes: Vec<Element<Message>> = self
            .tree
            .iter()
            .map(|node| match &node.kind {
                NodeKind::File => button(text(&node.name))
                    .on_press(Message::OpenFile(node.path.to_owned()))
                    .style(button_style)
                    .into(),
                NodeKind::Directory { open } => button(text(&node.name))
                    .on_press(Message::ProjectTreeToggleDirectory(node.id))
                    .style(button_style)
                    .into(),
            })
            .collect();
        Column::from_vec(nodes)
    }

    pub fn toggle_dir(&mut self, id: usize) -> Result<(), Error> {
        if let Some(node) = self.tree.iter_mut().find(|x| x.id == id) {
            node.edit(|n| {
                if let NodeKind::Directory { open } = &mut n.kind {
                    *open = !*open;
                    Ok(())
                } else {
                    Err(Error::NodeNotADirectory)
                }
            })
        } else {
            Err(Error::IDNotFound)
        }
    }
}

struct Node {
    id: usize,
    name: String,
    path: PathBuf,
    kind: NodeKind,
    parent: Option<usize>,
}

// https://stackoverflow.com/questions/49186751/sharing-a-common-value-in-all-enum-values
enum NodeKind {
    File,
    Directory { open: bool },
}

impl Node {
    fn new(path: PathBuf, id: usize, parent: Option<usize>) -> Self {
        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        return if path.is_dir() {
            Self {
                id: id,
                name: name,
                path: path,
                parent: parent,
                kind: NodeKind::Directory { open: false },
            }
        } else {
            Self {
                id: id,
                name: name,
                path: path,
                parent: parent,
                kind: NodeKind::File,
            }
        };
    }

    // fn flatten(&self) -> Vec<&Node> {
    //     match &self.kind {
    //         NodeKind::File => {
    //             vec![self]
    //         }
    //         NodeKind::Directory { open } => {
    //             let result = vec![self];
    //             if !open {
    //                 return result;
    //             }
    //             match children {
    //                 None => panic!("children not loaded"),
    //                 Some(nodes) => nodes.iter().fold(result, |mut acc, child| {
    //                     acc.append(&mut child.flatten());
    //                     acc
    //                 }),
    //             }
    //         }
    //     }
    // }

    // fn set_children(&mut self, children: Vec<Node>) -> Result<(), Error> {
    //     if let NodeKind::Directory {
    //         children: ref mut c,
    //         open: ref mut o,
    //     } = self.kind
    //     {
    //         *c = Some(children);
    //         *o = true;
    //         return Ok(());
    //     }
    //     Err(Error::NotADirectory)
    // }

    fn load_children(&self, mut next_id: usize) -> Result<(Vec<Node>, usize), Error> {
        if let NodeKind::Directory { open: _ } = &self.kind {
            let read_dir = fs::read_dir(&self.path)?;
            let mut nodes = Vec::new();
            for dir_entry in read_dir {
                let entry = dir_entry?;
                nodes.push(Node::new(entry.path(), next_id, Some(self.id)));
                next_id += 1;
            }
            return Ok((nodes, next_id));
        }
        Err(Error::NodeNotADirectory)
    }

    fn edit<T>(&mut self, edit_fn: fn(&mut Self) -> T) -> T {
        edit_fn(self)
    }
}

fn to_canonical(path: PathBuf) -> PathBuf {
    fs::canonicalize(&path).expect("could not canonicalize")
}
