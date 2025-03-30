use std::{ffi, fs, path};

use iced::widget::{
    Column, Row, Scrollable, button, column, container, row, scrollable, text, text_editor,
};
use iced::{Alignment, Element, Font, Length, Padding, Pixels, Task, highlighter};
use iced_aw::TabBar;

use crate::Message;

pub struct TabView {
    active_tab: usize,
    tabs: Vec<Tab>,
}

impl TabView {
    pub fn new() -> Self {
        Self {
            active_tab: usize::default(),
            tabs: Vec::new(),
        }
    }

    pub fn tabs(&self) -> &Vec<Tab> {
        &self.tabs
    }

    pub fn push(&mut self, tab: Tab) -> usize {
        self.tabs.push(tab);
        self.tabs.len() - 1
    }

    pub fn select(&mut self, selected: usize) {
        self.active_tab = selected
    }

    pub fn view(&self) -> Element<Message> {
        let main = if let Some(tab) = self.tabs.get(self.active_tab) {
            tab.view()
        } else {
            scrollable(Row::new())
        };

        Column::new()
            .push(
                self.tabs
                    .iter()
                    .fold(TabBar::new(Message::TabSelected), |tab_bar, tab| {
                        let idx = tab_bar.size();
                        tab_bar.push(idx, iced_aw::TabLabel::Text(tab.name.to_owned()))
                    })
                    .tab_width(Length::Shrink)
                    .set_active_tab(&self.active_tab),
            )
            .push(main)
            .into()
    }

    pub fn get_current(&mut self) -> Option<&mut Tab> {
        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
            Some(tab)
        } else {
            None
        }
    }
}

pub struct Tab {
    pub name: String,
    pub file_path: Option<path::PathBuf>,
    pub content: text_editor::Content,
}

impl Tab {
    pub fn new() -> Self {
        Self {
            name: "New Tab".to_owned(),
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
                self.content = text_editor::Content::with_text(&content);
                self.name = file_path
                    .file_name()
                    .expect("invalid file")
                    .to_str()
                    .expect("invalid file name")
                    .to_owned();
                self.file_path = Some(file_path);
            }
            Err(err) => {
                log::error!("Could not load file {:?}: {}", file_path, err)
            }
        }
    }

    pub fn save(&self) {
        if let Some(path) = &self.file_path {
            match fs::write(path, self.content.text()) {
                Ok(()) => {}
                Err(err) => {
                    log::error!("{}", err)
                }
            }
        }
    }

    pub fn view(&self) -> Scrollable<Message> {
        let font_size = 15.0;
        let line_height = 1.1;
        let syntax_theme = highlighter::Theme::SolarizedDark;

        Scrollable::new(row![
            line_number(self.content.line_count(), font_size, line_height,),
            text_editor(&self.content)
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
                    self.file_path
                        .as_deref()
                        .and_then(path::Path::extension)
                        .and_then(ffi::OsStr::to_str)
                        .unwrap_or(""),
                    syntax_theme,
                )
                .on_action(Message::Edit)
        ])
        .height(Length::Fill)
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::default().scroller_width(0).width(0),
        ))
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
