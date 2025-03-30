use std::path::PathBuf;

use iced::{
    Element, Task, Theme, highlighter,
    widget::{button, column, row, scrollable, text_editor},
};
use rfd::FileDialog;

mod tab;

#[derive(Debug, Clone)]
enum Message {
    Increment,
    ThemeSelected(highlighter::Theme),
    Edit(text_editor::Action),
    SelectFile,
    TabSelected(usize),
}

fn main() -> Result<(), iced::Error> {
    iced::application("Editorium", App::update, App::view)
        .theme(App::theme)
        .run_with(App::new)
}

struct App {
    value: u64,
    working_dir: Option<PathBuf>,
    tabs: tab::TabView,
}

impl App {
    fn new() -> (App, Task<Message>) {
        (
            App {
                value: 1,
                working_dir: None,
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
            Message::SelectFile => {
                let mut tab = tab::Tab::new();

                if let Some(file_path) = select_file(&self.working_dir) {
                    tab.open(&file_path);
                }
                let idx = self.tabs.push(tab);
                self.tabs.select(idx);
            }
            Message::TabSelected(tab) => self.tabs.select(tab),
            _ => {
                todo!()
            }
        }

        Task::none()
    }

    fn view(&self) -> Element<Message> {
        // let file_selection = row![]
        let nav_bar = row![
            button("Open File").on_press(Message::SelectFile) //     // current_project
                                                              //     // current git branch
                                                              //     // run
        ];
        // column![
        // container().width(Length::Fill)
        // nav_bar
        // pick_list(highlighter::Theme::ALL, Some(highlighter::Theme::SolarizedDark), Message::ThemeSelected)
        // ].into()

        column![nav_bar, self.tabs.view()].into()
    }

    fn theme(&self) -> Theme {
        Theme::CatppuccinFrappe
    }
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
