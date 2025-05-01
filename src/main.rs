use std::{collections::HashMap, fmt, fs, path::PathBuf, str::FromStr};

use iced::{
    Color, Element, Length, Subscription, Task, Theme,
    advanced::graphics::core::keyboard,
    event,
    widget::{
        Column, Container, PaneGrid, button, column, container, pane_grid, pick_list, row,
        scrollable, text, text_editor,
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
    ProjectTreeToggleDirectory(usize),
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
    project_tree: Option<project::ProjectTree>,
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
        (
            Self {
                tabs: tab::TabView::new(),
                project_tree: None,
                current_project: None,
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
            Message::ProjectTreeToggleDirectory(id) => {
                if let Some(tree) = &mut self.project_tree {
                    if let Err(err) = tree.toggle_dir(id) {
                        log::error!("could not toggle tree: {}", err)
                    }
                }
            }
            _ => {
                todo!()
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let cwd = PathBuf::from_str("./").expect("could not get cwd");

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
                let file_tree = if let Some(tree) = &self.project_tree {
                    tree.view()
                } else {
                    Column::new()
                };

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

    fn open_project(&mut self, dir_path: PathBuf) {
        self.current_project = Some(project::Project::new(dir_path.clone()));
        match project::ProjectTree::new(dir_path.clone()) {
            Ok(tree) => self.project_tree = Some(tree),
            Err(err) => log::error!("could not open project {:?}: {}", dir_path, err),
        }
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
