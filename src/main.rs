use std::path::PathBuf;

use iced::{
    Element, Task, Theme, highlighter,
    widget::{button, column, row, text_editor},
};
use rfd::FileDialog;

mod tab;

#[derive(Debug, Clone)]
enum Message {
    Increment,
    ThemeSelected(highlighter::Theme),
    Edit(text_editor::Action),
    SelectFile,
    ChangeTab(usize),
}

fn main() -> Result<(), iced::Error> {
    iced::application("Editorium", App::update, App::view)
        .theme(App::theme)
        .run_with(App::new)
}

struct App {
    value: u64,
    text_content: text_editor::Content,
    working_dir: Option<PathBuf>,
    tabs: tab::TabView,
}

impl App {
    fn new() -> (App, Task<Message>) {
        (
            App {
                value: 1,
                text_content: text_editor::Content::with_text(
                    "1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n11\n12\n13\n14\n15\n16\n17\n18\n19\n20\n21\n22\n23\n24\n25\n26\n27\n28\n29\n30\n31\n32\n33\n34\n35\n36\n37\n38\n39\n40\n41\n42\n43\n44\n45\n46\n47\n48\n49\n50\n51\n52\n53\n54\n55\n56\n57\n58\n59\n60\n61\n62\n63\n64\n65\n66\n67\n68\n69\n70\n71\n72\n73\n74\n75\n76\n77\n78\n79\n80\n81\n82\n83\n84\n85\n86\n87\n88\n89\n90\n9",
                ),
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
                _ => self.text_content.perform(action),
            },
            Message::SelectFile => {
                let mut tab = tab::Tab::new();

                if let Some(file_path) = select_file(&self.working_dir) {
                    tab.open(&file_path);
                }
                self.tabs.push(tab);
            }
            Message::ChangeTab(tab_index) => {
                self.tabs.change_tab(tab_index);
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
