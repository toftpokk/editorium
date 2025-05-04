use std::path::PathBuf;
use std::sync::RwLock;
use std::{fs, io};

use cosmic_text::{Attrs, Buffer, Edit, Metrics, SyntaxEditor, SyntaxSystem};
use iced::advanced::graphics::text::{editor, font_system};
use iced::widget::{Column, Row, Scrollable, column, container, row, scrollable, text};
use iced::{Alignment, Element, Font, Length, Padding, Pixels};
use iced_aw::TabBar;
use tab_widget::tab_widget;

use crate::{FONT_SYSTEM, Message, SYNTAX_SYSTEM};

mod tab_widget;

// TODO: use iced editor as an example for content RwLock
// TODO: use viewer(model) instead of model.view()

pub struct TabView {
    active: Option<usize>,
    tabs: Vec<Tab>,
}

impl TabView {
    pub fn new() -> Self {
        Self {
            active: None,
            tabs: Vec::new(),
        }
    }

    pub fn insert(&mut self, path: Option<PathBuf>) -> io::Result<usize> {
        let mut tab = Tab::new();
        if let Some(path) = path {
            tab.open_file(path)?;
        }
        self.tabs.push(tab);
        Ok(self.tabs.len() - 1)
    }

    pub fn remove(&mut self, index: usize) {
        // check tab exists
        if let Some(tab) = self.tabs.get(index) {
            tab
        } else {
            return;
        };

        self.tabs.remove(index);

        // check shift left
        let last_active = if let Some(active) = self.active {
            if active >= index {
                active
            } else {
                return;
            }
        } else {
            // no active tab
            return;
        };

        if last_active > 0 {
            self.active = Some(last_active - 1);
        } else {
            if self.tabs.len() > 0 {
                self.active = Some(0)
            } else {
                self.active = None
            }
        }
    }

    pub fn activate(&mut self, index: usize) {
        if let Some(_) = self.tabs.get(index) {
            self.active = Some(index)
        }
    }

    pub fn active(&self) -> Option<usize> {
        self.active
    }

    pub fn tab(&self, index: usize) -> Option<&Tab> {
        self.tabs.get(index)
    }

    pub fn tab_mut(&mut self, index: usize) -> Option<&mut Tab> {
        self.tabs.get_mut(index)
    }

    pub fn position(&self, path: PathBuf) -> Option<usize> {
        self.tabs.iter().position(|x| {
            if let Some(x_path) = &x.file_path {
                x_path == &path
            } else {
                false
            }
        })
    }

    pub fn view(&self) -> Element<Message> {
        let main = if let Some(active) = self.active {
            let tab = self.tabs.get(active).unwrap();
            tab.view()
        } else {
            // scrollable(Row::new())
            Column::new()
        };

        let mut tab_bar = self
            .tabs
            .iter()
            .fold(TabBar::new(Message::TabSelected), |tab_bar, tab| {
                let idx = tab_bar.size();
                tab_bar.push(idx, iced_aw::TabLabel::Text(tab.get_name().to_owned()))
            })
            .on_close(Message::TabClose)
            .tab_width(Length::Shrink);

        if let Some(active) = self.active {
            tab_bar = tab_bar.set_active_tab(&active);
        }

        Column::new().push(tab_bar).push(main).into()
    }
}

pub struct Tab {
    pub file_path: Option<PathBuf>,

    editor: RwLock<SyntaxEditor<'static, 'static>>, // RwLock allows writing during draw
    attrs: Attrs<'static>,
    metrics: Metrics,

    padding: Padding,
    line_numbers: bool,
}

impl Tab {
    fn new() -> Self {
        let metrics = Metrics::new(14.0, 20.0);
        let buffer = Buffer::new_empty(metrics);
        let attrs = Attrs::new().family(cosmic_text::Family::Monospace);
        let syntax_system: &SyntaxSystem = SYNTAX_SYSTEM.get().unwrap();
        let editor = SyntaxEditor::new(buffer, &syntax_system, "base16-mocha.dark").unwrap();
        Self {
            file_path: None,
            editor: RwLock::new(editor),
            attrs,
            padding: Padding {
                top: 0.0,
                right: 0.0,
                bottom: 0.0,
                left: 0.0,
            },
            metrics,
            line_numbers: true,
        }
    }

    pub fn open_file(&mut self, file_path: PathBuf) -> io::Result<()> {
        let mut font_system = FONT_SYSTEM.get().unwrap().write().unwrap();
        self.editor
            .write()
            .unwrap()
            .borrow_with(&mut font_system)
            .load_text(file_path.clone(), self.attrs.clone())?;
        self.file_path = Some(file_path);
        Ok(())
    }

    pub fn save(&mut self) -> io::Result<()> {
        if let Some(path) = &self.file_path {
            let mut text = String::new();
            self.editor.write().unwrap().with_buffer(|buf| {
                for line in buf.lines.iter() {
                    text.push_str(line.text());
                    text.push_str(line.ending().as_str());
                }
            });

            return fs::write(path, text);
        }
        Ok(())
    }

    pub fn scroll(&mut self, scroll: f32) {
        let mut editor = self.editor.write().unwrap();
        editor.with_buffer_mut(|buffer| {
            let mut current_scroll = buffer.scroll();
            current_scroll.vertical += scroll;
            buffer.set_scroll(current_scroll);
        });
    }

    pub fn view(&self) -> Column<Message> {
        let w = tab_widget(&self.editor, self.metrics);
        Column::new().push(w)
    }

    pub fn redraw(&self) {
        self.editor.write().unwrap().set_redraw(true);
    }

    fn get_name(&self) -> String {
        if let Some(path) = &self.file_path {
            path.file_name()
                .expect("invalid file name")
                .to_str()
                .expect("could not parse to string")
                .to_owned()
        } else {
            "New Tab".into()
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
