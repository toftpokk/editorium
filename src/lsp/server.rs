use lsp_types::{notification::Notification, request::Request};
use serde_json::json;
use smol::{
    Task, channel,
    io::{self, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt},
    process, spawn,
};
use std::{self, str::FromStr};

use crate::{Message, lsp::jsonrpc};

pub struct Server {
    pub initialized: bool,
    server_options: Option<lsp_types::InitializeResult>,

    // if removed, Child removed from scope
    _process: process::Child,
    next_req_id: u64,
    transport: Transport,
    last_message: Option<String>,
}

impl Server {
    pub fn connect(program: String) -> Server {
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

    pub fn on_message(&mut self, raw_message: jsonrpc::Message) -> iced::Task<Message> {
        if let Some(response) = raw_message.clone().as_response() {
            // TODO buffer request. Assuming response to last request. Ignoring message ID
            if let Some(method) = &self.last_message {
                if response.error.is_some() {
                    log::error!("lsp returned an error to {}: {:?}", method, response)
                }

                match method.as_str() {
                    lsp_types::request::Initialize::METHOD => {
                        self.initialized = true;
                        let response: lsp_types::InitializeResult =
                            serde_json::from_value(response.result.unwrap()).unwrap();
                        let log_string = if let Some(info) = &response.server_info {
                            if let Some(version) = &info.version {
                                format!("{} {}", info.name, version)
                            } else {
                                format!("{}", info.name)
                            }
                        } else {
                            "no server info".to_string()
                        };
                        self.server_options = Some(response);
                        log::info!("Connected: {}", log_string);

                        let req = Self::initialized();
                        let writer = self.new_writer();
                        return iced::Task::perform(
                            async move { writer.write(req.as_message()).await },
                            |x| {
                                match x {
                                    Ok(..) => {}
                                    Err(err) => {
                                        log::error!("{:?}", err)
                                    }
                                }
                                Message::None
                            },
                        );
                    }
                    _ => log::warn!(
                        "response unknown previous method {}: {:?}",
                        method,
                        response
                    ),
                }
            } else {
                log::warn!("response to unknown request: {:?}", response)
            }
        } else {
            log::warn!("response to rpc message: {:?}", raw_message)
        }
        iced::Task::none()
    }

    pub fn initialize(&mut self) -> jsonrpc::Request {
        let params = Self::init_params();

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

    pub fn initialized() -> jsonrpc::Notification {
        jsonrpc::Notification::new(lsp_types::notification::Initialized::METHOD, None)
    }

    pub fn text_document_did_open(
        file: url::Url,
        language_id: String,
        version: i32,
        text: &str,
    ) -> jsonrpc::Notification {
        let params = lsp_types::DidOpenTextDocumentParams {
            text_document: lsp_types::TextDocumentItem {
                uri: lsp_types::Uri::from_str(file.as_str()).unwrap(),
                language_id: language_id,
                version: version,
                text: text.to_string(),
            },
        };

        let req = jsonrpc::Notification::new(
            lsp_types::notification::DidOpenTextDocument::METHOD,
            Some(serde_json::to_value(params).unwrap()),
        );

        req
    }

    pub fn new_writer(&self) -> Writer {
        Writer {
            writer: self.transport.writer.clone(),
        }
    }

    pub fn new_reader(&self) -> Reader<jsonrpc::Message> {
        Reader {
            reader: self.transport.reader.clone(),
        }
    }

    pub fn new_reader_error(&self) -> Reader<String> {
        Reader {
            reader: self.transport.error.clone(),
        }
    }

    fn init_params() -> lsp_types::InitializeParams {
        let process_id = std::process::id();

        // TODO later
        // let workspace_uri = lsp_types::Uri::from_str(workspace.as_str()).unwrap();
        // let workspace_folder = lsp_types::WorkspaceFolder {
        //     uri: workspace_uri,
        //     name: workspace_name,
        // };

        // TODO options for each lsp
        let options = json!({});

        lsp_types::InitializeParams {
            process_id: Some(process_id),
            workspace_folders: None,
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
    ChannelError(channel::SendError<jsonrpc::Message>),
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Error::Io(value)
    }
}

impl From<channel::SendError<jsonrpc::Message>> for Error {
    fn from(value: channel::SendError<jsonrpc::Message>) -> Self {
        Error::ChannelError(value)
    }
}

// so that Task::perform can clone only Writer, not whole client
pub struct Writer {
    writer: channel::Sender<jsonrpc::Message>,
}

impl Writer {
    pub async fn write(&self, msg: jsonrpc::Message) -> Result<(), Error> {
        match self.writer.send(msg).await {
            Ok(ok) => Ok(ok),
            Err(err) => Err(err.into()),
        }
    }
}

pub struct Reader<T> {
    reader: channel::Receiver<T>,
}

impl<T> Reader<T> {
    pub async fn read(&self) -> Result<T, channel::RecvError> {
        self.reader.recv().await
    }
}

// a set of message channels for reading & writing to buffer
// instead of directly r/w to buffer
struct Transport {
    // single instance
    _writer_worker: Task<()>,
    _reader_worker: Task<()>,
    _error_worker: Task<()>,

    // templates for cloning
    writer: channel::Sender<jsonrpc::Message>,
    reader: channel::Receiver<jsonrpc::Message>,
    error: channel::Receiver<String>,
}

impl Transport {
    fn new(
        stdin: io::BufWriter<process::ChildStdin>,
        stdout: io::BufReader<process::ChildStdout>,
        stderr: io::BufReader<process::ChildStderr>,
    ) -> Self {
        // spawn does await automatically, apparently
        // to lsp server
        let (writer, recv) = channel::unbounded::<jsonrpc::Message>();
        let writer_worker = spawn(Self::writer_worker(recv, stdin));

        // from lsp server
        let (send, reader) = channel::unbounded::<jsonrpc::Message>();
        let reader_worker = spawn(Self::reader_worker(send, stdout));

        let (send, error) = channel::unbounded::<String>();
        let error_worker = spawn(Self::error_worker(send, stderr));

        Transport {
            _writer_worker: writer_worker,
            _reader_worker: reader_worker,
            _error_worker: error_worker,
            writer,
            reader,
            error,
        }
    }
    // reads from channel and writes to stdin
    async fn writer_worker(
        chan_reader: channel::Receiver<jsonrpc::Message>,
        stdin: io::BufWriter<process::ChildStdin>,
    ) {
        let mut stdin = stdin;
        loop {
            // waits for message from channel
            let msg = chan_reader.recv().await;
            if let Ok(msg) = msg {
                Self::send(msg, &mut stdin).await;
            }
        }
    }

    async fn error_worker(
        chan_writer: channel::Sender<String>,
        mut stderr: io::BufReader<process::ChildStderr>,
    ) {
        loop {
            let mut buf = String::new();
            stderr.read_line(&mut buf).await.unwrap();
            // let result = Self::recv(&mut stderr).await.unwrap();
            if buf.len() > 0 {
                chan_writer.send(buf).await.unwrap();
            }
        }
    }

    // reads from stdin and writes to channel
    // recieve jsonrpc payload
    // note: AsyncRead polls, does not wait
    // maybe optimize?
    async fn reader_worker(
        chan_writer: channel::Sender<jsonrpc::Message>,
        mut stdout: io::BufReader<process::ChildStdout>,
    ) {
        loop {
            let result = Self::recv(&mut stdout).await.unwrap();
            if let Some(result) = result {
                chan_writer.send(result).await.unwrap();
            }
        }
    }

    async fn send<T>(message: jsonrpc::Message, writer: &mut io::BufWriter<T>)
    where
        T: io::AsyncWrite + Unpin, // need to be able to read async
    {
        let request = message.to_string();

        let all = format!(
            "Content-Length: {}\r\n\r\n{}",
            request.len(),
            request.to_string()
        );
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
