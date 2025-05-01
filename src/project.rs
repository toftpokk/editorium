use std::{fmt, fs, io, path::PathBuf};

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
        let mut res = Self {
            tree: vec![root],
            next_id: 1,
        };

        if let NodeKind::Directory { open: _ } = res.tree[0].kind {
            let (children, id) = res.tree[0].query_children(res.next_id)?;
            res.add_children(children, id);
        }
        Ok(res)
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

    // TODO implement b+ tree
    pub fn toggle_dir(&mut self, id: usize) -> Result<(), Error> {
        let node = self.node_find(id)?;

        let children_new = node.query_children()?;

        let children_old: Vec<&Node> = self
            .tree
            .iter()
            .filter(|n| {
                if let Some(parent_id) = n.parent {
                    parent_id == id
                } else {
                    false
                }
            })
            .collect();

        // // loop through children

        // things in children_old, but not in children_new
        let to_remove = children_old.iter().filter(|entry| {
            children_new
                .iter()
                .find(|x| x.path() == entry.path)
                .is_none()
        });

        // things in children_old, but not in children_new
        let to_add = children_new.iter().filter(|entry| {
            children_old
                .iter()
                .find(|x| x.path == entry.path())
                .is_none()
        });

        //     node.edit(|n| {
        //         if let NodeKind::Directory { open } = &mut n.kind {
        //             *open = !*open;
        //             Ok(())
        //         } else {
        //             Err(Error::NodeNotADirectory)
        //         }
        //     })?;
        //     // self.add_children(children, next_id);
        Ok(())
        // } else {
        //     Err(Error::IDNotFound)
        // }
    }

    fn add_children(&mut self, mut children: Vec<Node>, next_id: usize) {
        self.tree.append(&mut children);

        self.next_id = next_id;
    }

    fn node_find(&self, target_id: usize) -> Result<&Node, Error> {
        if let Some(target) = self.tree.iter().find(|x| x.id == target_id) {
            Ok(target)
        } else {
            return Err(Error::IDNotFound);
        }
    }

    fn node_edit<T>(&mut self, edit_fn: fn(&mut Node) -> T, target_id: usize) -> Result<T, Error> {
        if let Some(target) = self.tree.iter_mut().find(|x| x.id == target_id) {
            Ok(edit_fn(target))
        } else {
            return Err(Error::IDNotFound);
        }
    }
}

#[derive(Clone)]
struct Node {
    id: usize,
    name: String,
    path: PathBuf,
    kind: NodeKind,
    parent: Option<usize>,
}

// https://stackoverflow.com/questions/49186751/sharing-a-common-value-in-all-enum-values
#[derive(Clone, Copy)]
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

    fn query_children(&self) -> Result<(Vec<fs::DirEntry>), Error> {
        if let NodeKind::Directory { open: _ } = &self.kind {
            let read_dir = fs::read_dir(&self.path)?;
            let mut nodes = Vec::new();
            for dir_entry in read_dir {
                let entry = dir_entry?;
                nodes.push(entry);
            }
            return Ok(nodes);
        }
        Err(Error::NodeNotADirectory)
    }
}

fn to_canonical(path: PathBuf) -> PathBuf {
    fs::canonicalize(&path).expect("could not canonicalize")
}
