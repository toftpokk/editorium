use std::{collections::HashMap, fmt, fs, path::PathBuf, str::FromStr};

use iced::{
    Element, Length, Subscription, Task, Theme,
    advanced::graphics::{core::keyboard, text::cosmic_text::ttf_parser::kern},
    event, highlighter,
    keyboard::{Modifiers, key},
    widget::{Button, Column, Text, button, column, pick_list, row, scrollable, text_editor},
};
use rfd::FileDialog;

mod key_binds;
mod project;
mod tab;

#[derive(Debug, Clone)]
enum Message {
    KeyPressed(keyboard::Modifiers, keyboard::Key),
    ThemeSelected(highlighter::Theme),
    Edit(text_editor::Action),
    OpenFileSelector,
    OpenDirectorySelector,
    OpenFile(PathBuf),
    OpenProject(PathBuf),
    TabSelected(usize),
    TabClose,
    SaveFile,
}

fn main() -> Result<(), iced::Error> {
    env_logger::init();
    iced::application("Editorium", App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .run_with(App::new)
}

struct App {
    tabs: tab::TabView,
    project_tree: ProjectTree,
    key_binds: HashMap<key_binds::KeyBind, Message>,
}

impl App {
    fn new() -> (Self, Task<Message>) {
        let mut app = Self {
            tabs: tab::TabView::new(),
            project_tree: ProjectTree::new(),
            key_binds: key_binds::default(),
        };
        (app, Task::none())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::ThemeSelected(theme) => {
                println!("{}", theme)
            }
            Message::Edit(action) => match action {
                text_editor::Action::Scroll { .. } => (),
                _ => {
                    if let Some(t) = self.tabs.get_current() {
                        t.action(action);
                    }
                }
            },
            Message::OpenFileSelector => {
                if let Some(file_path) =
                    select_file(&self.project_tree.current.as_ref().map(|p| p.path.clone()))
                {
                    self.open_file(file_path)
                }
            }
            Message::OpenDirectorySelector => {
                if let Some(dir_path) =
                    select_dir(&self.project_tree.current.as_ref().map(|p| p.path.clone()))
                {
                    self.open_project(dir_path)
                }
            }
            Message::TabSelected(tab) => self.tabs.select(tab),
            Message::OpenProject(project) => self.open_project(project),
            Message::OpenFile(file_path) => self.open_file(file_path),
            Message::SaveFile => {
                if let Some(t) = self.tabs.get_current() {
                    t.save()
                }
            }
            Message::KeyPressed(modifier, key) => {
                if let Some(value) = self.key_binds.get(&key_binds::KeyBind {
                    modifiers: modifier,
                    key: key,
                }) {
                    return self.update(value.clone());
                }
            }
            Message::TabClose => self.tabs.close(self.tabs.get_active_pos()),
            _ => {
                todo!()
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let cwd = PathBuf::from_str("./").expect("could not get cwd");

        let recent_projects = vec![Project::new(cwd)];
        let nav_bar = row![
            pick_list(
                recent_projects,
                self.project_tree.current.clone(),
                |project: Project| Message::OpenProject(project.path),
            )
            .placeholder("Choose a Project"),
            button("Open File").on_press(Message::OpenFileSelector),
            button("Open Dir").on_press(Message::OpenDirectorySelector) //     // current_project
                                                                        //     // current git branch
                                                                        //     // run
        ];

        let file_tree = self
            .project_tree
            .nodes
            .iter()
            .map(|node| match node {
                project::Node::File { name, path } => button(Text::new(name))
                    .width(Length::Fill)
                    .on_press(Message::OpenFile(path.to_owned()))
                    .into(),
                project::Node::Directory { name, .. } => {
                    button(Text::new(name)).width(Length::Fill).into()
                }
            })
            .collect();

        column![
            nav_bar,
            row![Column::from_vec(file_tree).width(100.0), self.tabs.view()]
        ]
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        let subscriptions = vec![event::listen_with(|event, status, window_id| match event {
            event::Event::Keyboard(keyboard::Event::KeyPressed { modifiers, key, .. }) => {
                match status {
                    event::Status::Captured => None,
                    event::Status::Ignored => Some(Message::KeyPressed(modifiers, key)),
                }
            }
            _ => None,
        })];
        Subscription::batch(subscriptions)
    }

    fn theme(&self) -> Theme {
        Theme::CatppuccinFrappe
    }

    fn open_project(&mut self, dir_path: PathBuf) {
        let read_dir = match fs::read_dir(&dir_path) {
            Ok(ok) => ok,
            Err(err) => {
                log::error!("invalid directory {:?}: {}", dir_path, err);
                return;
            }
        };
        let mut nodes = Vec::new();
        for dir_entry in read_dir {
            let entry = match dir_entry {
                Ok(ok) => ok,
                Err(err) => {
                    log::error!("invalid directory {:?}: {}", dir_path, err);
                    return;
                }
            };
            let entry_path = entry.path();
            nodes.push(project::Node::new(entry_path));
        }
        self.project_tree.with_nodes(nodes);
    }

    fn open_file(&mut self, file_path: PathBuf) {
        if let Some(pos) = self.tabs.get_pos(&file_path) {
            self.tabs.select(pos);
            return;
        }
        let mut tab = tab::Tab::new();
        tab.open(&file_path);
        let idx = self.tabs.push(tab);
        self.tabs.select(idx);
    }
}

#[derive(PartialEq, Clone)]
struct Project {
    name: String,
    path: PathBuf,
}

impl Project {
    fn new(file_path: PathBuf) -> Self {
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

pub struct ProjectTree {
    current: Option<Project>,
    nodes: Vec<project::Node>,
}

impl ProjectTree {
    fn new() -> Self {
        Self {
            current: None,
            nodes: Vec::new(),
        }
    }
    fn with_nodes(&mut self, nodes: Vec<project::Node>) -> &mut Self {
        self.nodes = nodes;
        self
    }
}

impl fmt::Display for Project {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

fn select_dir(working_dir: &Option<PathBuf>) -> Option<PathBuf> {
    let mut dialog = FileDialog::new().set_title("Open a directory...");

    if let Some(working_dir) = working_dir {
        dialog = dialog.set_directory(working_dir)
    }

    if let Some(dir) = dialog.pick_folder() {
        return Some(dir);
    }
    return None;
}

fn select_file(working_dir: &Option<PathBuf>) -> Option<PathBuf> {
    let mut dialog = FileDialog::new().set_title("Open a file...");

    if let Some(working_dir) = working_dir {
        dialog = dialog.set_directory(working_dir)
    }

    if let Some(file) = dialog.pick_file() {
        return Some(file);
    }
    return None;
}
