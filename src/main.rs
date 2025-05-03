use std::{
    collections::HashMap,
    fs,
    path::PathBuf,
    str::FromStr,
    sync::{LazyLock, Once, OnceLock, RwLock},
};

use clap::Parser;
use iced::{
    Color, Element, Length, Subscription, Task, Theme,
    advanced::graphics::core::keyboard,
    event,
    widget::{
        Container, PaneGrid, button, column, container, pane_grid, pick_list, row, scrollable,
        text_editor,
    },
};
use iced_aw::iced_fonts;
use rfd::FileDialog;

mod cli;
mod key_binds;
mod project;
mod tab;

static FONT_SYSTEM: OnceLock<RwLock<cosmic_text::FontSystem>> = OnceLock::new();
// static SYNTAX_SYSTEM: LazyLock<RwLock<cosmic_text::SyntaxSystem>> =
//     LazyLock::new(|| RwLock::new(cosmic_text::SyntaxSystem::new()));
static SYNTAX_SYSTEM: OnceLock<cosmic_text::SyntaxSystem> = OnceLock::new();
// cosmic_text::SyntaxSystem::new();

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
    ProjectTreeSelect(usize),
    SaveFile,
}

fn main() -> Result<(), iced::Error> {
    // SYNTAX_SYSTEM.get_or_init(f)
    // FONT_SYSTEM = ;
    FONT_SYSTEM.get_or_init(|| RwLock::new(cosmic_text::FontSystem::new()));
    SYNTAX_SYSTEM.get_or_init(|| cosmic_text::SyntaxSystem::new());

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
    project_tree: project::ProjectTree,
    current_project: Option<project::Project>,
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
        let cli = cli::Cli::parse();

        let mut app = Self {
            tabs: tab::TabView::new(),
            project_tree: project::ProjectTree::new(),
            current_project: None,
            key_binds: key_binds::default(),
            panes: create_pane(),
        };

        if let Some(path) = cli.path {
            app.open_file(path);
        }

        (app, Task::none())
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            // Message::Edit(action) => match action {
            //     text_editor::Action::Scroll { .. } => (),
            //     _ => {
            //         if let Some(t) = self.tabs.get_current() {
            //             t.action(action);
            //         }
            //     }
            // },
            Message::OpenFileSelector => {
                if let Some(file_path) =
                    select_file(&self.current_project.as_ref().map(|p| p.path.clone()))
                {
                    self.open_file(file_path)
                }
            }
            Message::OpenDirectorySelector => {
                if let Some(dir_path) =
                    select_dir(&self.current_project.as_ref().map(|p| p.path.clone()))
                {
                    self.open_project(dir_path)
                }
            }
            Message::TabSelected(tab) => self.tabs.activate(tab),
            Message::OpenProject(project) => self.open_project(project),
            Message::OpenFile(file_path) => self.open_file(file_path),
            Message::SaveFile => {
                if let Some(active) = self.tabs.active() {
                    let tab = self.tabs.tab_mut(active).unwrap();
                    tab.save();
                };
            }
            Message::TabCloseCurrent => {
                let active = self.tabs.active().unwrap();
                self.tabs.remove(active);
            }
            Message::TabClose(tab) => {
                self.tabs.remove(tab);
            }
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
            Message::ProjectTreeSelect(id) => {
                let node = if let Some(node) = self.project_tree.node_mut(id) {
                    if let project::NodeKind::Directory { open } = &mut node.kind {
                        *open = !*open;
                    }
                    Some(node.clone())
                } else {
                    None
                };

                let pos = self.project_tree.position(id).unwrap();

                if let Some(node) = node {
                    match node.kind {
                        project::NodeKind::Directory { open } => {
                            if open {
                                self.tree_open_dir(node.path, pos, node.indent + 1);
                            } else {
                                while let Some(child) = self.project_tree.node_at(pos + 1) {
                                    if child.indent > node.indent {
                                        self.project_tree.remove(child.id);
                                    } else {
                                        break;
                                    }
                                }
                            }
                        }
                        project::NodeKind::File => {
                            self.open_file(node.path);
                        }
                    }
                }
            }
            #[allow(unreachable_patterns)]
            _ => {
                todo!()
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let cwd = PathBuf::from_str("./").expect("could not get cwd");
        let cwd = match fs::canonicalize(cwd) {
            Ok(ok) => ok,
            Err(err) => {
                panic!("could not open directory: {}", err);
            }
        };

        let recent_projects = vec![project::Project::new(cwd)];
        let nav_bar = row![
            pick_list(
                recent_projects,
                self.current_project.clone(),
                |project: project::Project| Message::OpenProject(project.path),
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
                let file_tree = self.project_tree.view();

                pane_grid::Content::new(
                    scrollable(Container::new(file_tree))
                        .width(Length::Fill)
                        .height(Length::Fill)
                        .direction(scrollable::Direction::Both {
                            vertical: scrollable::Scrollbar::default().scroller_width(0).width(0),
                            horizontal: scrollable::Scrollbar::default().scroller_width(0).width(0),
                        }),
                )
                .style(|_| container::Style {
                    background: Some(iced::Background::Color(Color::from_rgb8(0x2B, 0x2D, 0x30))),
                    ..Default::default()
                })
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

    fn open_project(&mut self, path: PathBuf) {
        let path = fs::canonicalize(&path).expect("could not canonicalize");
        self.current_project = Some(project::Project::new(path.clone()));
        self.project_tree.clear();
        self.project_tree.insert(path, 0, 0);
    }

    fn open_file(&mut self, file_path: PathBuf) {
        if let Some(pos) = self.tabs.position(file_path.clone()) {
            self.tabs.activate(pos);
            return;
        }
        let index = self.tabs.insert(Some(file_path));
        self.tabs.activate(index);
    }

    fn tree_open_dir(&mut self, path: PathBuf, pos: usize, indent: usize) {
        let path = fs::canonicalize(&path).expect("could not canonicalize");

        let read_dir = match fs::read_dir(path) {
            Ok(ok) => ok,
            Err(err) => {
                log::error!("could not open directory: {}", err);
                return;
            }
        };
        let mut nodes = Vec::new();
        for dir_entry in read_dir {
            match dir_entry {
                Ok(ok) => {
                    nodes.push(ok.path());
                }
                Err(err) => {
                    log::error!("could not read directory entry: {}", err);
                    return;
                }
            }
        }
        nodes.sort();

        let mut position = pos + 1;
        for node in nodes {
            self.project_tree.insert(node, position, indent);
            position += 1;
        }
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
