use lsp_types::request::Request;
use serde_json::json;
use smol::{
    Task, channel,
    io::{self, AsyncReadExt, AsyncWriteExt},
    process, spawn,
};
use std::{self, path, str::FromStr};

use crate::lsp::jsonrpc;

pub struct Client {
    next_req_id: u64,
    transport: Transport,
    initialized: bool,
}

impl Client {
    pub fn connect() -> Client {
        let process = process::Command::new("rust-analyzer")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .kill_on_drop(true)
            .spawn();

        let mut process = match process {
            Ok(process) => process,
            Err(e) => {
                panic!("{:?}", e)
            }
        };

        let (writer, recv) = channel::unbounded::<String>();
        // spawn does await automatically, apparently
        let writer_worker = spawn(writer_worker(
            recv,
            io::BufWriter::new(process.stdin.take().expect("Failed to open stdin")),
        ));

        let (send, reader) = channel::unbounded::<String>();
        let reader_worker = spawn(reader_worker(
            send,
            io::BufReader::new(process.stdout.take().expect("Failed to open stdout")),
            io::BufReader::new(process.stderr.take().expect("Failed to open stderr")),
        ));

        Self {
            initialized: false,
            next_req_id: 0,
            transport: Transport {
                _process: process,
                writer,
                reader,
                _writer_worker: writer_worker,
                _reader_worker: reader_worker,
            },
        }
    }

    pub fn build_init_message(
        &mut self,
        workspace: &path::PathBuf,
        workspace_name: String,
    ) -> String {
        let params = Self::init_params(workspace, workspace_name);

        let client_capabilities = serde_json::to_value(params).unwrap();

        let req = jsonrpc::Request::new(
            self.next_req_id,
            lsp_types::request::Initialize::METHOD,
            Some(client_capabilities),
        );
        self.next_req_id += 1;

        Self::build_request(req)
    }

    pub fn new_writer(&self) -> Writer {
        Writer {
            writer: self.transport.writer.clone(),
        }
    }

    pub fn new_reader(&self) -> Reader {
        Reader {
            reader: self.transport.reader.clone(),
        }
    }

    fn build_request(req: jsonrpc::Request) -> String {
        let req = serde_json::to_string(&req).unwrap();

        return format!("Content-Length: {}\r\n\r\n{}", req.len(), req);
    }

    fn init_params(
        workspace: &path::PathBuf,
        workspace_name: String,
    ) -> lsp_types::InitializeParams {
        let process_id = std::process::id();

        // TODO handle parse url error
        let workspace_uri = lsp_types::Uri::from_str(workspace.to_str().unwrap()).unwrap();
        let workspace_folder = lsp_types::WorkspaceFolder {
            uri: workspace_uri,
            name: workspace_name,
        };

        // TODO options for each lsp
        let options = json!({});

        lsp_types::InitializeParams {
            process_id: Some(process_id),
            workspace_folders: Some(vec![workspace_folder]),
            initialization_options: Some(options),
            capabilities: lsp_types::ClientCapabilities {
                workspace: Some(lsp_types::WorkspaceClientCapabilities {
                    apply_edit: Some(true), // modify resource
                    workspace_edit: Some(lsp_types::WorkspaceEditClientCapabilities {
                        document_changes: Some(true), // versioned document changes
                        resource_operations: Some(vec![
                            lsp_types::ResourceOperationKind::Create,
                            lsp_types::ResourceOperationKind::Delete,
                            lsp_types::ResourceOperationKind::Rename,
                        ]),
                        failure_handling: Some(lsp_types::FailureHandlingKind::Abort), // change is aborted if failed
                        normalizes_line_endings: Some(false),
                        change_annotation_support: None,
                    }),
                    did_change_configuration: Some(
                        lsp_types::DynamicRegistrationClientCapabilities {
                            dynamic_registration: Some(false),
                        },
                    ),
                    ..Default::default() // did_change_watch?
                }),
                ..Default::default() // text_document: (),
                                     // notebook_document: (),
                                     // window: (),
                                     // general: (),
                                     // experimental: (),
            },
            // trace: todo!(),
            // workspace_folders: todo!(),
            // client_info: todo!(),
            // locale: todo!(),
            // work_done_progress_params: todo!(),
            // deprecated
            root_path: None,
            root_uri: None,

            ..Default::default()
        }
    }
}

#[derive(Debug)]
pub enum Error {
    URIParseError(std::string::ParseError),
    Io(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Io(value)
    }
}

// so that Task::perform can clone only Writer, not whole client
pub struct Writer {
    writer: channel::Sender<String>,
}

impl Writer {
    pub async fn write(&self, msg: String) -> Result<(), channel::SendError<String>> {
        self.writer.send(msg).await
    }
}

pub struct Reader {
    reader: channel::Receiver<String>,
}

impl Reader {
    pub async fn read(&self) -> Result<String, channel::RecvError> {
        self.reader.recv().await
    }
}

struct Transport {
    // if removed, Child removed from scope
    _process: process::Child,

    // single instance
    _writer_worker: Task<()>,
    _reader_worker: Task<()>,

    // templates for cloning
    reader: channel::Receiver<String>,
    writer: channel::Sender<String>,
}

// reads from channel and writes to stdin
async fn writer_worker(
    chan_reader: channel::Receiver<String>,
    stdin: io::BufWriter<process::ChildStdin>,
) {
    let mut stdin = stdin;
    loop {
        // waits for message from channel
        let msg = chan_reader.recv().await;
        if let Ok(msg) = msg {
            stdin.write(msg.as_bytes()).await.unwrap();
            stdin.flush().await.unwrap();
        }
    }
}

// reads from stdin and stdout and writes to channel
async fn reader_worker(
    chan_writer: channel::Sender<String>,
    mut stdout: io::BufReader<process::ChildStdout>,
    mut stderr: io::BufReader<process::ChildStderr>,
) {
    smol::future::race(
        async {
            let mut buf = vec![0; 1024];
            loop {
                let cnt = stdout.read(&mut buf).await.unwrap();
                if cnt > 0 {
                    let msg = String::from_utf8(buf[..cnt].to_vec()).unwrap();
                    chan_writer.send(msg).await.unwrap()
                }
            }
        },
        async {
            let mut buf = vec![0; 1024];
            loop {
                let cnt = stderr.read(&mut buf).await.unwrap();
                if cnt > 0 {
                    let msg = String::from_utf8(buf[..cnt].to_vec()).unwrap();
                    chan_writer.send(msg).await.unwrap()
                }
            }
        },
    )
    .await;
}
