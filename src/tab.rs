use std::path::PathBuf;
use std::sync::RwLock;
use std::{fs, io};

use cosmic_text::{Attrs, Buffer, Edit, Metrics, SyntaxEditor, SyntaxSystem};
use iced::advanced::widget::operate;
use iced::widget::{self, Column, Scrollable, scrollable, text_input};
use iced::{Element, Length, Task, advanced};
use iced_aw::TabBar;

use crate::{FONT_SYSTEM, Message, SYNTAX_SYSTEM, text_box, theme};

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

    pub fn view(&self) -> Element<Message, theme::MyTheme> {
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
            .width(Length::Shrink)
            .tab_width(Length::Shrink);

        if let Some(active) = self.active {
            tab_bar = tab_bar.set_active_tab(&active);
        }

        Column::new()
            .push(
                Scrollable::new(tab_bar)
                    .width(Length::Fill)
                    .height(Length::Shrink)
                    .direction(scrollable::Direction::Horizontal(
                        scrollable::Scrollbar::default().scroller_width(0),
                    )),
            )
            .push(main)
            .into()
    }
}

pub struct Search {
    id: text_input::Id,
    text: String,
}

pub struct Tab {
    pub file_path: Option<PathBuf>,

    editor: RwLock<SyntaxEditor<'static, 'static>>, // RwLock allows writing during draw
    attrs: Attrs<'static>,
    metrics: Metrics,
    text_box_id: iced::advanced::widget::Id,
    search: Search,
    search_open: bool,
}

impl Tab {
    fn new() -> Self {
        let metrics = Metrics::new(14.0, 20.0);
        let buffer = Buffer::new_empty(metrics);
        let attrs = Attrs::new().family(cosmic_text::Family::Monospace);
        let syntax_system: &SyntaxSystem = SYNTAX_SYSTEM.get().unwrap();
        let editor = SyntaxEditor::new(buffer, &syntax_system, "base16-eighties.dark").unwrap();

        let mut tab = Self {
            file_path: None,
            editor: RwLock::new(editor),
            attrs,
            metrics,
            search: Search {
                id: text_input::Id::unique(),
                text: "".to_string(),
            },
            search_open: false,
            text_box_id: advanced::widget::Id::unique(),
        };
        tab.set_config();

        tab
    }

    pub fn open_file(&mut self, file_path: PathBuf) -> io::Result<()> {
        let mut font_system = FONT_SYSTEM.get().unwrap().write().unwrap();
        let mut editor = self.editor.write().unwrap();
        let mut editor = editor.borrow_with(&mut font_system);

        // shape_until_scroll loads *all* text when height_opt = None
        // this skips shaping entirely
        editor.with_buffer_mut(|buffer| {
            buffer.set_size(Some(0.0), Some(0.0));
        });

        editor.load_text(file_path.clone(), self.attrs.clone())?;
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

    pub fn search_open(&mut self, text: Option<String>) -> Task<Message> {
        if let Some(text) = text {
            self.search.text = text;
        }
        // note: text seach is a good example of how events flow
        // also: editor is a good example of how leaf nodes work (widgets)
        self.search_open = true;
        widget::text_input::focus(self.search.id.clone())
    }

    pub fn search_close(&mut self) -> Task<Message> {
        self.search_open = false;
        // lifesaver: https://jl710.github.io/iced-guide/widget_api/operations.html
        operate(advanced::widget::operation::focusable::focus(
            self.text_box_id.clone(),
        ))
    }

    pub fn scroll(&mut self, scroll: f32) {
        let mut editor = self.editor.write().unwrap();
        editor.with_buffer_mut(|buffer| {
            let mut current_scroll = buffer.scroll();
            current_scroll.vertical += scroll;
            buffer.set_scroll(current_scroll);
        });
    }

    pub fn view(&self) -> Column<Message, theme::MyTheme> {
        let mut col = Column::new();
        if self.search_open {
            col = col.push(
                text_input("Find Something...", &self.search.text)
                    .on_input(Message::TabSearch)
                    .id(self.search.id.clone()),
            )
        }

        // TODO: halloy's combo_box
        col.push(text_box::text_box(&self.editor, self.metrics).id(self.text_box_id.clone()))
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

    fn set_config(&mut self) {
        let mut editor = self.editor.write().unwrap();
        let mut font_system = FONT_SYSTEM.get().unwrap().write().unwrap();

        let mut editor = editor.borrow_with(&mut font_system);
        editor.set_tab_width(4);
        editor.set_auto_indent(true);
        editor.with_buffer_mut(|buffer| {
            buffer.set_wrap(cosmic_text::Wrap::None);
        });
    }
}
