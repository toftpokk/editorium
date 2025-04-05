use std::{collections::HashMap, fmt, fs, path::PathBuf, str::FromStr};

use iced::{
    Color, Element, Length, Subscription, Task, Theme,
    advanced::graphics::core::keyboard,
    event,
    widget::{
        Container, PaneGrid, button, column, pane_grid, pick_list, row, scrollable, text,
        text_editor,
    },
};
use iced_aw::iced_fonts;
use rfd::FileDialog;

mod key_binds;
mod project;
mod tab;

#[derive(Debug, Clone)]
enum Message {
    KeyPressed(keyboard::Modifiers, keyboard::Key),
    Edit(text_editor::Action),
    OpenFileSelector,
    OpenDirectorySelector,
    OpenFile(PathBuf),
    OpenProject(PathBuf),
    TabSelected(usize),
    TabClose(usize),
    TabCloseCurrent,
    PaneResized(pane_grid::ResizeEvent),
    SaveFile,
}

fn main() -> Result<(), iced::Error> {
    env_logger::init();
    iced::application("Editorium", App::update, App::view)
        .subscription(App::subscription)
        .theme(App::theme)
        .font(iced_fonts::REQUIRED_FONT_BYTES)
        .run_with(App::new)
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

struct Pane {
    pane_type: PaneType,
}

#[derive(PartialEq)]
enum PaneType {
    FileTree,
    Editor,
}

impl Pane {
    fn new(pane_type: PaneType) -> Self {
        Self {
            pane_type: pane_type,
        }
    }
}

struct App {
    tabs: tab::TabView,
    project_tree: ProjectTree,
    key_binds: HashMap<key_binds::KeyBind, Message>,
    panes: pane_grid::State<Pane>,
}

fn create_pane() -> pane_grid::State<Pane> {
    let (mut pane_grid_state, pane) = pane_grid::State::new(Pane::new(PaneType::FileTree));

    let (_, split) = pane_grid_state
        .split(pane_grid::Axis::Vertical, pane, Pane::new(PaneType::Editor))
        .unwrap();

    pane_grid_state.resize(split, 0.2);

    pane_grid_state
}

impl App {
    fn new() -> (Self, Task<Message>) {
        (
            Self {
                tabs: tab::TabView::new(),
                project_tree: ProjectTree::new(),
                key_binds: key_binds::default(),
                panes: create_pane(),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
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
            Message::TabCloseCurrent => self.tabs.close(self.tabs.get_active_pos()),
            Message::TabClose(tab) => self.tabs.close(tab),
            Message::KeyPressed(modifier, key) => {
                if let Some(value) = self.key_binds.get(&key_binds::KeyBind {
                    modifiers: modifier,
                    key: key,
                }) {
                    return self.update(value.clone());
                }
            }
            Message::PaneResized(pane_grid::ResizeEvent { split, ratio }) => {
                self.panes.resize(split, ratio);
            }
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

        let pane_grid = PaneGrid::new(&self.panes, |_, state, _| {
            if state.pane_type == PaneType::Editor {
                pane_grid::Content::new(self.tabs.view())
            } else {
                let file_tree: Vec<Element<Message>> = self
                    .project_tree
                    .nodes
                    .iter()
                    .map(|node| match node {
                        project::Node::File { name, path } => button(text(name))
                            .width(Length::Fill)
                            .on_press(Message::OpenFile(path.to_owned()))
                            .style(button_style)
                            .into(),
                        project::Node::Directory { name, path } => button(text(name))
                            .on_press(Message::OpenFile(path.to_owned()))
                            .width(Length::Fill)
                            .style(button_style)
                            .into(),
                    })
                    .collect();
                // pane_grid::Content::new(Column::from_vec(file_tree))
                // let s = Limits::new(Length::Fill./, max)
                pane_grid::Content::new(
                    scrollable(
                        Container::new(
                            column![
                                button("hello").width(Length::Fill).style(|_, _| {
                                    button::Style {
                                        background: Some(iced::Background::Color(
                                            Color::from_rgb8(0xFF, 0xFF, 0xFF),
                                        )),
                                        text_color: Color::from_rgb8(0xFF, 0xFF, 0xFF),
                                        ..Default::default()
                                    }
                                }),
                                button("helaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaalo")
                                    .width(Length::Fill)
                                    .style(button_style),
                                button("hello").width(Length::Fill).style(button_style),
                            ]
                            .extend(file_tree),
                        )
                        .width(Length::Shrink), // .max_width(Pixels::from(100)),
                    )
                    .width(Length::Fill)
                    .height(Length::Fill) // scrollable(Column::from_vec(file_tree).width(Length::Fill))
                    //     .width(Length::Fill)
                    //     .height(Length::Fill)
                    .direction(scrollable::Direction::Both {
                        vertical: scrollable::Scrollbar::default().scroller_width(0).width(0),
                        horizontal: scrollable::Scrollbar::default().scroller_width(0).width(0),
                    }),
                )
            }
        })
        .width(Length::Fill)
        .height(Length::Fill)
        .spacing(10)
        .on_resize(10, Message::PaneResized);

        let content: Element<Message> = column![nav_bar, pane_grid].into();

        // k.explain(iced::Color::WHITE)
        content
    }

    fn subscription(&self) -> Subscription<Message> {
        let subscriptions = vec![event::listen_with(|event, status, _| match event {
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

        self.project_tree.with_project(nodes, dir_path);
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
    fn with_project(&mut self, nodes: Vec<project::Node>, path: PathBuf) -> &mut Self {
        self.nodes = nodes;
        self.current = Some(Project::new(path));
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
