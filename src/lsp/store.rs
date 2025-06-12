use std::{
    collections::HashMap,
    fmt::Display,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
};

use iced::Task;

// TODO move Message back to main, should not be here
use crate::{
    Message,
    lsp::{self},
};

static ID_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct Id(u64);

impl Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

// Copied from zed
pub enum ServerState {
    Starting(lsp::Server, Option<Task<Message>>),
    Running(lsp::Server),
}

pub struct Store {
    workspace: Option<PathBuf>,
    pub servers: HashMap<Id, ServerState>,
    languages: HashMap<String, Id>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            workspace: None,
            servers: HashMap::new(),
            languages: HashMap::new(),
        }
    }

    pub fn use_workspace(&mut self, workspace: PathBuf) {
        self.workspace = Some(workspace)
    }

    pub fn process(&mut self, id: Id, message: lsp::Message) {
        let Some(entry) = self.servers.get(&id) else {
            log::warn!("message unknown server: {} {:?}", id, message);
            return;
        };
        log::info!("{} {:?}", id, message)
    }

    pub fn get_or_init_lsp(&mut self, lang: String) -> Id {
        if let Some(id) = self.languages.get(&lang) {
            return id.clone();
        }
        let mut server = lsp::Server::connect("rust-analyzer".to_string());
        let req = server.initialize();
        let writer = server.new_writer();
        let fut = async move { writer.write(req.as_message()).await };

        let id = Id(ID_COUNTER.fetch_add(1, Ordering::SeqCst));
        self.servers.insert(
            id,
            lsp::store::ServerState::Starting(
                server,
                Some(Task::perform(fut, |x| {
                    match x {
                        Ok(..) => {}
                        Err(err) => {
                            log::error!("{:?}", err)
                        }
                    }
                    Message::None
                })),
            ),
        );
        self.languages.insert(lang, id);

        id
    }
}
