use std::{
    collections::HashMap,
    fmt::Display,
    path::PathBuf,
    str::FromStr,
    sync::atomic::{AtomicU64, Ordering},
};

use iced::Task;
use lsp_types::Uri;

// TODO move Message back to main, should not be here
use crate::{
    Message, buffer,
    lsp::{self, jsonrpc},
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
    Starting(lsp::Server, u64, Option<Task<Message>>),
    Running(lsp::Server),
}

impl ServerState {
    fn set_state_running(self) -> Self {
        match self {
            ServerState::Starting(server, _, _) => return Self::Running(server),
            ServerState::Running(server) => Self::Running(server),
        }
    }
}

pub struct BufferSnapshot {
    version: i32,
    file: url::Url,
    language: String,
    text: String,
}

impl BufferSnapshot {
    // TODO text as weak reference to buffer text
    pub fn new(version: i32, file: url::Url, language: String, text: String) -> Self {
        Self {
            version: version,
            file: file,
            language: language,
            text: text,
        }
    }
}

pub struct Store {
    workspace: Option<PathBuf>,

    snapshots: HashMap<buffer::Id, BufferSnapshot>,
    registered_buffers: Vec<buffer::Id>,
    pub servers: HashMap<Id, ServerState>,
    languages: HashMap<String, Id>,
}

impl Store {
    pub fn new() -> Self {
        Self {
            workspace: None,
            servers: HashMap::new(),
            languages: HashMap::new(),
            registered_buffers: Vec::new(),
            snapshots: HashMap::new(),
        }
    }

    pub fn use_workspace(&mut self, workspace: PathBuf) {
        self.workspace = Some(workspace)
    }

    pub fn on_message(&mut self, id: Id, message: lsp::Message) -> Task<Message> {
        let entry = match self.servers.get_mut(&id) {
            Some(entry) => entry,
            None => {
                log::warn!("message unknown server: {} {:?}", id, message);
                return Task::none();
            }
        };

        match entry {
            ServerState::Starting(server, init_req_id, _) => {
                let mut task = server.on_message(message.clone());
                if let Some(response) = message.as_response() {
                    if server.initialized && response.id.as_u64().unwrap() == *init_req_id {
                        let writer = server.new_writer();
                        // notify open buffers
                        let requests: Vec<jsonrpc::Notification> = self
                            .registered_buffers
                            .iter()
                            .map(|buf_id| {
                                let snapshot = self.snapshots.get(buf_id).unwrap();
                                lsp::Server::text_document_did_open(
                                    snapshot.file.clone(),
                                    snapshot.language.clone(),
                                    snapshot.version.into(),
                                    &snapshot.text,
                                )
                            })
                            .collect();

                        let t = Task::perform(
                            async move {
                                for req in requests {
                                    match writer.write(req.as_message()).await {
                                        Ok(_) => {}
                                        Err(err) => log::error!("{:?}", err),
                                    }
                                }
                            },
                            |_| Message::None,
                        );
                        task = task.chain(t);

                        // FIXME feels hacky
                        let (id, server_state) = self.servers.remove_entry(&id).unwrap();
                        self.servers.insert(id, server_state.set_state_running());
                    }
                }
                task
            }
            ServerState::Running(server) => server.on_message(message),
        }
    }

    pub fn register_buffer(&mut self, id: buffer::Id, snapshot: BufferSnapshot) {
        self.registered_buffers.push(id);
        self.snapshots.insert(id, snapshot);
    }

    pub fn get_or_init_lsp(&mut self, lang: String) -> Id {
        if let Some(id) = self.languages.get(&lang) {
            return id.clone();
        }
        let mut server = lsp::Server::connect("rust-analyzer".to_string());
        let req = server.initialize();
        let req_id = req.id.as_u64().unwrap();
        let writer = server.new_writer();
        let fut = async move { writer.write(req.as_message()).await };

        let id = Id(ID_COUNTER.fetch_add(1, Ordering::SeqCst));
        self.servers.insert(
            id,
            lsp::store::ServerState::Starting(
                server,
                req_id,
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
