use std::{fmt, path::PathBuf, str::FromStr};

use iced::{
    Element, Task, Theme, highlighter,
    widget::{button, column, pick_list, row, scrollable, text_editor},
};
use rfd::FileDialog;

mod tab;

#[derive(Debug, Clone)]
enum Message {
    Increment,
    ThemeSelected(highlighter::Theme),
    Edit(text_editor::Action),
    OpenFileSelector,
    OpenDirectorySelector,
    SelectProject(Project),
    TabSelected(usize),
}

fn main() -> Result<(), iced::Error> {
    iced::application("Editorium", App::update, App::view)
        .theme(App::theme)
        .run_with(App::new)
}

struct App {
    value: u64,
    current_project: Option<Project>,
    tabs: tab::TabView,
}

impl App {
    fn new() -> (App, Task<Message>) {
        (
            App {
                value: 1,
                current_project: None,
                tabs: tab::TabView::new(),
            },
            Task::none(),
        )
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Increment => self.value += 1,
            Message::ThemeSelected(theme) => {
                println!("{}", theme)
            }
            Message::Edit(action) => match action {
                text_editor::Action::Scroll { .. } => (),
                _ => self.tabs.perform(action),
            },
            Message::OpenFileSelector => {
                if let Some(file_path) =
                    select_file(&self.current_project.as_ref().map(|p| p.dir_path.clone()))
                {
                    let mut tab = tab::Tab::new();
                    tab.open(&file_path);
                    let idx = self.tabs.push(tab);
                    self.tabs.select(idx);
                }
            }
            Message::OpenDirectorySelector => {
                if let Some(dir_path) =
                    select_dir(&self.current_project.as_ref().map(|p| p.dir_path.clone()))
                {
                    self.current_project = Some(Project::new(dir_path))
                }
            }
            Message::TabSelected(tab) => self.tabs.select(tab),
            Message::SelectProject(project) => self.change_project(project),
            _ => {
                todo!()
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<Message> {
        let cwd = PathBuf::from_str("/home").expect("could not get cwd");

        let proj = Project::new(cwd);

        let recent_projects = vec![proj];
        let nav_bar = row![
            pick_list(
                recent_projects,
                self.current_project.clone(),
                Message::SelectProject
            )
            .placeholder("Choose a Project"),
            button("Open File").on_press(Message::OpenFileSelector),
            button("Open Dir").on_press(Message::OpenDirectorySelector) //     // current_project
                                                                        //     // current git branch
                                                                        //     // run
        ];

        column![nav_bar, self.tabs.view()].into()
    }

    fn theme(&self) -> Theme {
        Theme::CatppuccinFrappe
    }

    fn change_project(&mut self, project: Project) {
        self.current_project = Some(project)
    }
}

#[derive(PartialEq, Clone, Debug)]
struct Project {
    name: String,
    dir_path: PathBuf,
}

impl Project {
    fn new(dir_path: PathBuf) -> Self {
        let name = dir_path
            .file_name()
            .expect("invalid dir name")
            .to_str()
            .expect("invalid dir name string");
        Self {
            name: name.to_string(),
            dir_path: dir_path,
        }
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
