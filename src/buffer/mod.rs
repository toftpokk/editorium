use std::collections::HashMap;
use std::fmt::Display;
use std::path::PathBuf;
use std::sync::RwLock;
use std::{
    fs, io,
    sync::atomic::{AtomicU64, Ordering},
};

use cosmic_text::{Attrs, Edit, Metrics, SyntaxEditor, SyntaxSystem};
use iced::advanced::widget::operate;
use iced::widget::text_input;
use iced::{Task, advanced};

use crate::{FONT_SYSTEM, Message, SYNTAX_SYSTEM, lsp};

// TODO: use iced editor as an example for content RwLock
// TODO: use viewer(model) instead of model.view()

static ID_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Id(u64);

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub struct Store {
    buffers: HashMap<Id, Buffer>,

    active: Option<usize>,
    buffer_list: Vec<Id>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            active: None,
            buffers: HashMap::new(),
            buffer_list: Vec::new(),
        }
    }

    pub fn insert(&mut self, path: Option<PathBuf>) -> io::Result<Id> {
        let mut buf = Buffer::new();
        if let Some(path) = path {
            buf.open_file(path)?;
        }

        let id = Id(ID_COUNTER.fetch_add(1, Ordering::SeqCst));
        self.buffers.insert(id, buf);
        self.buffer_list.push(id);
        Ok(id)
    }

    pub fn buf_unlist(&mut self, target: Id) -> Option<Id> {
        let target_idx = self.buffer_list.iter().position(|x| *x == target)?;
        self.buffer_list.remove(target_idx);

        let Some(active_idx) = self.active else {
            return Some(target);
        };
        if active_idx < target_idx {
            return Some(target);
        };

        if active_idx > 0 {
            self.active = Some(active_idx - 1)
        } else {
            if self.buffers.len() > 0 {
                self.active = Some(0)
            } else {
                self.active = None
            }
        }

        Some(target)
    }

    pub fn buf_remove(&mut self, id: Id) -> Option<()> {
        let Some(_) = self.buffers.remove(&id) else {
            return None;
        };
        self.buf_unlist(id);
        return Some(());
    }

    pub fn activate(&mut self, id: Id) -> Option<usize> {
        let idx = self.position(id)?;
        self.active = Some(idx);
        Some(idx)
    }

    pub fn activate_with_lsp(&mut self, id: Id, lsp: lsp::Id) -> Option<usize> {
        let buf = self.buffers.get_mut(&id)?;
        buf.register_lsp(lsp);
        Some(self.activate(id).unwrap())
    }

    pub fn active_idx(&self) -> Option<usize> {
        self.active
    }

    pub fn active(&self) -> Option<Id> {
        let idx = self.active?;
        Some(self.id(idx).unwrap().clone())
    }

    pub fn buf_mut(&mut self, id: Id) -> Option<&mut Buffer> {
        self.buffers.get_mut(&id)
    }

    pub fn buf(&self, id: Id) -> Option<&Buffer> {
        self.buffers.get(&id)
    }

    pub fn buffer_list(&self) -> &Vec<Id> {
        &self.buffer_list
    }

    pub fn id_from_path(&self, target: &PathBuf) -> Option<Id> {
        self.buffers
            .iter()
            .find(|(_, buf)| {
                let Some(path) = &buf.file_path else {
                    return false;
                };
                path == target
            })
            .map(|(id, _)| *id)
    }

    pub fn position(&self, id: Id) -> Option<usize> {
        self.buffer_list.iter().position(|x| *x == id)
    }
    pub fn id(&self, pos: usize) -> Option<&Id> {
        self.buffer_list.get(pos)
    }
}

pub struct Search {
    pub id: text_input::Id,
    pub text: String,
}

pub struct Buffer {
    pub file_path: Option<PathBuf>,

    pub editor: RwLock<SyntaxEditor<'static, 'static>>, // RwLock allows writing during draw
    attrs: Attrs<'static>,
    pub metrics: Metrics,
    pub text_box_id: iced::advanced::widget::Id,
    pub search: Search,
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

    pub fn is_search_open(&self) -> bool {
        self.search_open
    }

    pub fn search_open(&mut self, text: Option<String>) -> Task<Message> {
        if let Some(text) = text {
            self.search.text = text;
        }
        // note: text seach is a good example of how events flow
        // also: editor is a good example of how leaf nodes work (widgets)
        self.search_open = true;
        text_input::focus(self.search.id.clone())
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

    pub fn redraw(&self) {
        self.editor.write().unwrap().set_redraw(true);
    }

    pub fn get_name(&self) -> Option<String> {
        let Some(path) = &self.file_path else {
            return None;
        };
        Some(
            path.file_name()
                .expect("invalid file name")
                .to_str()
                .expect("could not parse to string")
                .to_owned(),
        )
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
