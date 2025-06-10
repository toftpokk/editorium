use lsp_types::request::Request;
use serde_json::json;
use smol::{
    Task, channel,
    io::{self, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt},
    process, spawn,
};
use std::{self, path, str::FromStr};

use crate::lsp::jsonrpc;

pub struct Client {
    // if removed, Child removed from scope
    _process: process::Child,
    next_req_id: u64,
    transport: Transport,
    initialized: bool,
    server_options: Option<lsp_types::InitializeResult>,
    last_message: Option<String>,
}

impl Client {
    pub fn connect(program: String) -> Client {
        let process = process::Command::new(program)
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

        let transport = Transport::new(
            io::BufWriter::new(process.stdin.take().expect("Failed to open stdin")),
            io::BufReader::new(process.stdout.take().expect("Failed to open stdout")),
            io::BufReader::new(process.stderr.take().expect("Failed to open stderr")),
        );

        Self {
            transport,
            _process: process,

            initialized: false,
            next_req_id: 0,
            last_message: None,
            server_options: None,
        }
    }

    pub fn on_message(&mut self, raw_message: jsonrpc::Message) {
        if raw_message.is_response() {
            let message = raw_message.as_response();
            // TODO buffer request. Assuming response to last request. Ignoring message ID
            if let Some(method) = &self.last_message {
                if message.error.is_some() {
                    log::error!("lsp returned an error to {}: {:?}", method, message)
                }

                match method.as_str() {
                    lsp_types::request::Initialize::METHOD => {
                        self.initialized = true;
                        let response: lsp_types::InitializeResult =
                            serde_json::from_value(message.result.unwrap()).unwrap();
                        let log_string = if let Some(info) = &response.server_info {
                            if let Some(version) = &info.version {
                                format!("{} {}", info.name, version)
                            } else {
                                format!("{}", info.name)
                            }
                        } else {
                            "no server info".to_string()
                        };
                        log::info!("Connected: {}", log_string);
                        self.server_options = Some(response);
                    }
                    _ => log::warn!("response unknown previous method {}: {:?}", method, message),
                }
            } else {
                log::warn!("response to unknown request: {:?}", message)
            }
        } else {
            log::warn!("response to rpc message: {:?}", raw_message)
        }
    }

    pub fn build_init_message(
        &mut self,
        workspace: url::Url,
        workspace_name: String,
    ) -> jsonrpc::Request {
        let params = Self::init_params(workspace, workspace_name);

        let client_capabilities = serde_json::to_value(params).unwrap();

        let req = jsonrpc::Request::new(
            self.next_req_id,
            lsp_types::request::Initialize::METHOD,
            Some(client_capabilities),
        );
        self.last_message = Some(lsp_types::request::Initialize::METHOD.to_string());
        self.next_req_id += 1;

        req
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

    fn init_params(workspace: url::Url, workspace_name: String) -> lsp_types::InitializeParams {
        let process_id = std::process::id();

        // TODO handle parse url error
        let workspace_uri = lsp_types::Uri::from_str(workspace.as_str()).unwrap();
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
    ChannelError(channel::SendError<jsonrpc::Request>),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Io(value)
    }
}

impl From<channel::SendError<jsonrpc::Request>> for Error {
    fn from(value: channel::SendError<jsonrpc::Request>) -> Self {
        Error::ChannelError(value)
    }
}

// so that Task::perform can clone only Writer, not whole client
pub struct Writer {
    writer: channel::Sender<jsonrpc::Request>,
}

impl Writer {
    pub async fn write(&self, msg: jsonrpc::Request) -> Result<(), Error> {
        match self.writer.send(msg).await {
            Ok(ok) => Ok(ok),
            Err(err) => Err(err.into()),
        }
    }
}

pub struct Reader {
    reader: channel::Receiver<jsonrpc::Message>,
}

impl Reader {
    pub async fn read(&self) -> Result<jsonrpc::Message, channel::RecvError> {
        self.reader.recv().await
    }
}

// a set of message channels for reading & writing to buffer
// instead of directly r/w to buffer
struct Transport {
    // single instance
    _writer_worker: Task<()>,
    _reader_worker: Task<()>,

    // templates for cloning
    reader: channel::Receiver<jsonrpc::Message>,
    writer: channel::Sender<jsonrpc::Request>,
}

impl Transport {
    fn new(
        stdin: io::BufWriter<process::ChildStdin>,
        stdout: io::BufReader<process::ChildStdout>,
        stderr: io::BufReader<process::ChildStderr>,
    ) -> Self {
        // from lsp server
        let (writer, recv) = channel::unbounded::<jsonrpc::Request>();
        // spawn does await automatically, apparently
        let writer_worker = spawn(Self::writer_worker(recv, stdin));

        // to lsp server
        let (send, reader) = channel::unbounded::<jsonrpc::Message>();
        let reader_worker = spawn(Self::reader_worker(send, stdout, stderr));

        Transport {
            _writer_worker: writer_worker,
            _reader_worker: reader_worker,
            reader: reader,
            writer: writer,
        }
    }
    // reads from channel and writes to stdin
    async fn writer_worker(
        chan_reader: channel::Receiver<jsonrpc::Request>,
        stdin: io::BufWriter<process::ChildStdin>,
    ) {
        let mut stdin = stdin;
        loop {
            // waits for message from channel
            let msg = chan_reader.recv().await;
            if let Ok(msg) = msg {
                Self::send(
                    msg.id.as_u64().unwrap(),
                    &msg.method,
                    msg.params,
                    &mut stdin,
                )
                .await;
            }
        }
    }

    // reads from stdin and stdout and writes to channel
    // recieve jsonrpc payload
    // note: AsyncRead polls, does not wait
    // maybe optimize?
    async fn reader_worker(
        chan_writer: channel::Sender<jsonrpc::Message>,
        mut stdout: io::BufReader<process::ChildStdout>,
        mut stderr: io::BufReader<process::ChildStderr>,
    ) {
        loop {
            smol::future::race(
                Self::read_from_buf(&chan_writer, &mut stdout),
                Self::read_from_buf(&chan_writer, &mut stderr),
            )
            .await;
        }
    }

    async fn read_from_buf<T>(
        chan_writer: &channel::Sender<jsonrpc::Message>,
        buf: &mut io::BufReader<T>,
    ) where
        T: io::AsyncRead + Unpin, // need to be able to read async
    {
        let result = Self::recv(buf).await.unwrap();
        if let Some(result) = result {
            chan_writer.send(result).await.unwrap();
        }
    }

    async fn send<T>(
        request_id: u64,
        method: &str,
        params: Option<serde_json::Value>,
        writer: &mut io::BufWriter<T>,
    ) where
        T: io::AsyncWrite + Unpin, // need to be able to read async
    {
        let request = jsonrpc::request(request_id, method, params);

        let all = format!("Content-Length: {}\r\n\r\n{}", request.len(), request);
        writer.write_all(all.as_bytes()).await.unwrap();
        writer.flush().await.unwrap();
    }

    // recieve jsonrpc payload
    // note: AsyncRead polls, does not wait
    // maybe optimize?
    async fn recv<T>(reader: &mut io::BufReader<T>) -> Result<Option<jsonrpc::Message>, Error>
    where
        T: io::AsyncRead + Unpin, // need to be able to read async
    {
        let mut content_length = 0;
        let mut content_type = "application/vscode-jsonrpc; charset=utf-8".to_string();
        loop {
            let mut header_line = String::new();
            reader.read_line(&mut header_line).await?;
            // read_line will return empty string while polling
            if header_line.len() < 1 {
                return Ok(None);
            }

            let header_line = header_line.as_str().trim();
            if header_line.len() == 0 {
                // end of headers
                break;
            }

            let parts = header_line.split_once(":").unwrap();
            let header = parts.0.trim();
            let value = parts.1.trim();
            match header {
                "Content-Length" => {
                    content_length = value.parse().unwrap();
                }
                "Content-Type" => {
                    content_type = value.to_string();
                }
                header => {
                    log::warn!("Unknown header: {}", header)
                }
            }
        }

        let mut buf = vec![0; content_length];
        reader.read_exact(&mut buf).await?;

        if content_type != "application/vscode-jsonrpc; charset=utf-8" {
            panic!("Unknown content type: {}", content_type);
        }

        let payload = String::from_utf8(buf).unwrap();

        Ok(Some(jsonrpc::Message::from(payload)))
    }
}
// TODO use treesitter https://github.com/nvim-treesitter/nvim-treesitter?tab=readme-ov-file#supported-languages
