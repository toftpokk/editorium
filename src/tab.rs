use std::{ffi, fs, path};

use iced::{Alignment, Element, Font, Length, Padding, Pixels, highlighter, widget};

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
        self.current_tab = Some(self.tabs.len());
    }

    pub fn view(&self) -> Element<Message> {
        let font_size = 15.0;
        let line_height = 1.1;
        let syntax_theme = highlighter::Theme::SolarizedDark;

        if let Some(tab_index) = self.current_tab {
            let tab = &self.tabs[tab_index - 1];
            widget::scrollable(widget::row![
                line_number(tab.content.line_count(), font_size, line_height,),
                widget::text_editor(&tab.content)
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
            ])
            .height(Length::Fill)
            .direction(widget::scrollable::Direction::Vertical(
                widget::scrollable::Scrollbar::default()
                    .scroller_width(0)
                    .width(0),
            ))
            .into()
        } else {
            widget::Column::new().into()
        }
    }
}

pub struct Tab {
    file_path: Option<path::PathBuf>,
    content: widget::text_editor::Content,
}

impl Tab {
    pub fn new() -> Self {
        Self {
            file_path: None,
            content: widget::text_editor::Content::new(),
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
                self.content = widget::text_editor::Content::with_text(&content)
            }
            Err(err) => {
                log::error!("Could not load file {:?}: {}", file_path, err)
            }
        }
    }
}

fn line_number(line_count: usize, font_size: f32, line_height: f32) -> Element<'static, Message> {
    let mut lines: Vec<Element<Message>> = Vec::new();
    let box_height = widget::text::LineHeight::from(line_height).to_absolute(Pixels(font_size));

    for i in 1..line_count.saturating_add(1) {
        let container = widget::container(
            widget::text(i)
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

    widget::column(lines)
        .padding(Padding {
            top: 0.0,
            bottom: 0.0,
            left: 0.0,
            right: 15.0,
        })
        .into()
}
