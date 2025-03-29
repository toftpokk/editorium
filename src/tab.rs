use std::{ffi, fs, path};

use iced::widget::{Row, button, column, container, row, scrollable, text, text_editor};
use iced::{Alignment, Element, Font, Length, Padding, Pixels, highlighter};

use crate::Message;

pub struct TabView {
    current_tab: Option<usize>,
    tabs: Vec<Tab>,
}

impl TabView {
    pub fn new() -> Self {
        Self {
            current_tab: None,
            tabs: Vec::new(),
        }
    }

    pub fn push(&mut self, tab: Tab) {
        self.tabs.push(tab);
        self.current_tab = Some(self.tabs.len() - 1);
    }

    pub fn change_tab(&mut self, tab_index: usize) {
        if tab_index < self.tabs.len() {
            self.current_tab = Some(tab_index);
        }
    }

    pub fn view(&self) -> Element<Message> {
        let font_size = 15.0;
        let line_height = 1.1;
        let syntax_theme = highlighter::Theme::SolarizedDark;

        let main = if let Some(tab_index) = self.current_tab {
            let tab = &self.tabs[tab_index];
            row![
                line_number(tab.content.line_count(), font_size, line_height,),
                text_editor(&tab.content)
                    .font(Font::MONOSPACE)
                    .size(font_size)
                    .line_height(line_height)
                    .padding(Padding {
                        top: 0.0,
                        bottom: 0.0,
                        left: 5.0,
                        right: 0.0,
                    })
                    .highlight(
                        tab.file_path
                            .as_deref()
                            .and_then(path::Path::extension)
                            .and_then(ffi::OsStr::to_str)
                            .unwrap_or(""),
                        syntax_theme,
                    )
                    .on_action(Message::Edit)
            ]
        } else {
            Row::new()
        };

        let tab_list = self
            .tabs
            .iter()
            .enumerate()
            .fold(Row::new(), |row, (index, tab)| {
                let file_name = if let Some(path) = &tab.file_path {
                    path.file_name()
                        .expect("invalid file")
                        .to_str()
                        .expect("invalid file string")
                } else {
                    ""
                };
                row.push(button(file_name).on_press(Message::ChangeTab(index)))
            });

        column![
            tab_list,
            scrollable(main)
                .height(Length::Fill)
                .direction(scrollable::Direction::Vertical(
                    scrollable::Scrollbar::default().scroller_width(0).width(0),
                )),
        ]
        .into()
    }
}

pub struct Tab {
    file_path: Option<path::PathBuf>,
    content: text_editor::Content,
}

impl Tab {
    pub fn new() -> Self {
        Self {
            file_path: None,
            content: text_editor::Content::new(),
        }
    }

    pub fn open(&mut self, file_path: &path::PathBuf) {
        let file_path = match fs::canonicalize(file_path) {
            Ok(ok) => ok,
            Err(err) => {
                log::error!("could not canonicalize file {:?}: {}", file_path, err);
                return;
            }
        };
        match fs::read_to_string(&file_path) {
            Ok(content) => {
                self.file_path = Some(file_path);
                self.content = text_editor::Content::with_text(&content)
            }
            Err(err) => {
                log::error!("Could not load file {:?}: {}", file_path, err)
            }
        }
    }
}

fn line_number(line_count: usize, font_size: f32, line_height: f32) -> Element<'static, Message> {
    let mut lines: Vec<Element<Message>> = Vec::new();
    let box_height = text::LineHeight::from(line_height).to_absolute(Pixels(font_size));

    for i in 1..line_count.saturating_add(1) {
        let container = container(
            text(i)
                .font(Font::MONOSPACE)
                .size(font_size)
                .line_height(line_height),
        )
        .width(Length::Fixed(40.0))
        .align_x(Alignment::End)
        .align_y(Alignment::End)
        .height(box_height);
        lines.push(container.into());
    }

    column(lines)
        .padding(Padding {
            top: 0.0,
            bottom: 0.0,
            left: 0.0,
            right: 15.0,
        })
        .into()
}
