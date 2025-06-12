use std::{
    collections::HashMap,
    collections::hash_map::Entry,
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

impl ServerState {
    fn set_state_running(self) -> Self {
        match self {
            ServerState::Starting(server, _) => return Self::Running(server),
            ServerState::Running(server) => Self::Running(server),
        }
    }
}

pub struct Store {
    workspace: Option<PathBuf>,

    pub language_servers: HashMap<Id, ServerState>,
    languages: HashMap<String, Id>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            workspace: None,
            language_servers: HashMap::new(),
            languages: HashMap::new(),
        }
    }

    pub fn use_workspace(&mut self, workspace: PathBuf) {
        self.workspace = Some(workspace)
    }

    pub fn on_message(&mut self, id: Id, message: lsp::Message) {
        let entry = match self.language_servers.get_mut(&id) {
            Some(entry) => entry,
            None => {
                log::warn!("message unknown server: {} {:?}", id, message);
                return;
            }
        };

        match entry {
            ServerState::Starting(server, _) => {
                server.on_message(message);
                if server.initialized {
                    // FIXME feels hacky
                    let (id, server_state) = self.language_servers.remove_entry(&id).unwrap();
                    self.language_servers
                        .insert(id, server_state.set_state_running());
                }
            }
            ServerState::Running(server) => {
                server.on_message(message);
            }
        }
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
        self.language_servers.insert(
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
