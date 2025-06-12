use std::path::PathBuf;
use std::sync::RwLock;
use std::{fs, io};

use cosmic_text::{Attrs, Edit, Metrics, SyntaxEditor, SyntaxSystem};
use iced::advanced::widget::operate;
use iced::widget::{self, Column, Scrollable, scrollable, text_input};
use iced::{Element, Length, Task, advanced};
use iced_aw::TabBar;

use crate::{FONT_SYSTEM, Message, SYNTAX_SYSTEM, lsp, text_box, theme};

// TODO: use iced editor as an example for content RwLock
// TODO: use viewer(model) instead of model.view()

pub struct Store {
    active: Option<usize>,
    buffers: Vec<Buffer>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            active: None,
            buffers: Vec::new(),
        }
    }

    pub fn insert(&mut self, path: Option<PathBuf>) -> io::Result<usize> {
        let mut buf = Buffer::new();
        if let Some(path) = path {
            buf.open_file(path)?;
        }
        self.buffers.push(buf);
        Ok(self.buffers.len() - 1)
    }

    pub fn remove(&mut self, index: usize) {
        // check buf exists
        if let Some(buf) = self.buffers.get(index) {
            buf
        } else {
            return;
        };

        self.buffers.remove(index);

        // check shift left
        let last_active = if let Some(active) = self.active {
            if active >= index {
                active
            } else {
                return;
            }
        } else {
            // no active buffer
            return;
        };

        if last_active > 0 {
            self.active = Some(last_active - 1);
        } else {
            if self.buffers.len() > 0 {
                self.active = Some(0)
            } else {
                self.active = None
            }
        }
    }

    pub fn activate(&mut self, index: usize) {
        if let Some(_) = self.buffers.get(index) {
            self.active = Some(index)
        }
    }

    pub fn activate_with_lsp(&mut self, index: usize, lsp: lsp::Id) {
        if let Some(buf) = self.buffers.get_mut(index) {
            buf.register_lsp(lsp);
            self.active = Some(index)
        }
    }

    pub fn active(&self) -> Option<usize> {
        self.active
    }

    pub fn buf_mut(&mut self, index: usize) -> Option<&mut Buffer> {
        self.buffers.get_mut(index)
    }

    pub fn position(&self, path: PathBuf) -> Option<usize> {
        self.buffers.iter().position(|x| {
            if let Some(x_path) = &x.file_path {
                x_path == &path
            } else {
                false
            }
        })
    }

    pub fn view(&self) -> Element<Message, theme::MyTheme> {
        let main = if let Some(active) = self.active {
            let buf = self.buffers.get(active).unwrap();
            buf.view()
        } else {
            // scrollable(Row::new())
            Column::new()
        };

        let mut tab_bar = self
            .buffers
            .iter()
            .fold(TabBar::new(Message::BufferSelected), |tab_bar, tab| {
                let idx = tab_bar.size();
                tab_bar.push(idx, iced_aw::TabLabel::Text(tab.get_name().to_owned()))
            })
            .on_close(Message::BufferClose)
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

pub struct Buffer {
    pub file_path: Option<PathBuf>,

    editor: RwLock<SyntaxEditor<'static, 'static>>, // RwLock allows writing during draw
    attrs: Attrs<'static>,
    metrics: Metrics,
    text_box_id: iced::advanced::widget::Id,
    search: Search,
    search_open: bool,
    lsp: Option<lsp::Id>,
}

impl Buffer {
    fn new() -> Self {
        let metrics = Metrics::new(14.0, 20.0);
        let buffer_inner = cosmic_text::Buffer::new_empty(metrics);
        let attrs = Attrs::new().family(cosmic_text::Family::Monospace);
        let syntax_system: &SyntaxSystem = SYNTAX_SYSTEM.get().unwrap();
        let editor =
            SyntaxEditor::new(buffer_inner, &syntax_system, "base16-eighties.dark").unwrap();

        let mut buf = Self {
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
            lsp: None,
        };
        buf.set_config();

        buf
    }

    pub fn register_lsp(&mut self, id: lsp::Id) {
        self.lsp = Some(id)
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
                    .on_input(Message::BufferSearch)
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
