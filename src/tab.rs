use std::{ffi, fs, path};

use iced::widget::scrollable::RelativeOffset;
use iced::widget::{
    Row, Scrollable, button, column, container, row, scrollable, text, text_editor,
};
use iced::{Alignment, Element, Font, Length, Padding, Pixels, Task, highlighter};

use crate::Message;

pub struct TabView {
    active_tab: usize,
    current_tab: Option<usize>,
    tabs: Vec<Tab>,
}

impl TabView {
    pub fn new() -> Self {
        Self {
            active_tab: usize::default(),
            current_tab: None,
            tabs: Vec::new(),
        }
    }

    pub fn push(&mut self, tab: Tab) {
        self.tabs.push(tab);
        self.current_tab = Some(self.tabs.len() - 1);
    }

    pub fn change_tab(&mut self, tab_index: usize) -> Task<Message> {
        if tab_index < self.tabs.len() {
            self.current_tab = Some(tab_index);
            // let tab = &self.tabs[tab_index];
            // return scrollable::snap_to(tab.scroll_id.clone(), tab.scroll_offset);
        }
        Task::none()
    }

    pub fn select(&mut self, selected: usize) {
        self.active_tab = selected
    }

    pub fn view(&self) -> Element<Message> {
        let main = if let Some(tab_index) = self.current_tab {
            let tab = &self.tabs[tab_index];
            tab.view()
        } else {
            scrollable(Row::new())
        };

        // let tab_list = self
        //     .tabs
        //     .iter()
        //     .enumerate()
        //     .fold(Row::new(), |row, (index, tab)| {
        //         let file_name = if let Some(path) = &tab.file_path {
        //             path.file_name()
        //                 .expect("invalid file")
        //                 .to_str()
        //                 .expect("invalid file string")
        //         } else {
        //             ""
        //         };
        //         row.push(button(file_name).on_press(Message::ChangeTab(index)))
        //     });

        column![iced_aw::TabBar::new(Message::TabSelected), main].into()
    }

    pub fn perform(&mut self, action: text_editor::Action) {
        if let Some(tab_index) = self.current_tab {
            self.tabs[tab_index].content.perform(action);
        }
    }

    https://github.com/iced-rs/iced_aw/blob/main/examples/tab_bar.rs

    pub fn set_offset(&mut self, id: scrollable::Id, x: f32, y: f32) {
        // for tab in &mut self.tabs {
        //     if tab.scroll_id == id {
        //         tab.scroll_offset.x = x;
        //         tab.scroll_offset.y = y;
        //     }
        // }
    }
}

pub struct Tab {
    file_path: Option<path::PathBuf>,
    content: text_editor::Content,
    scroll_id: scrollable::Id,
    scroll_offset: scrollable::RelativeOffset,
}

impl Tab {
    pub fn new() -> Self {
        Self {
            file_path: None,
            content: text_editor::Content::new(),
            scroll_id: scrollable::Id::unique(),
            scroll_offset: scrollable::RelativeOffset::default(),
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
        .on_scroll(|viewport| {
            let RelativeOffset { x, y } = viewport.relative_offset();
            Message::TabScroll(self.scroll_id.clone(), x, y)
        })
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
